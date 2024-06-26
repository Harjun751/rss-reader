use rss_api::{
    database::DatabaseConnection,
    logger::{self, DetailedError},
    rss_parser, web_scraper, Channel, Post, Subscription,
};

use axum::{
    debug_handler,
    extract::{Json, Query, State},
    http::StatusCode,
    routing::{get, post},
    Router,
};
use axum_server::tls_rustls::RustlsConfig;
use http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderValue, Method,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;
use tokio_cron_scheduler::{Job, JobScheduler};
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, services::ServeFile, trace::TraceLayer};
use tracing::{event, Level};
use tracing_subscriber::{filter, layer::Layer, prelude::*};

#[derive(Clone)]
struct Appstate {
    dbconn: DatabaseConnection,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            logger::FileLogger.with_filter(filter::LevelFilter::from_level(tracing::Level::ERROR)),
        )
        .init();
    let dbconn = DatabaseConnection::new();

    let origin = match env::var("IS_DOCKER_COMPOSED") {
        Ok(val) => val,
        Err(_) => "http://localhost:5173".to_string(),
    };
    let origin = origin.parse::<HeaderValue>().unwrap();

    let cors = CorsLayer::new()
        // allow requests from any origin
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
        .allow_origin(origin)
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let app = Router::new()
        .route("/all", get(all_posts))
        .route("/feed", get(feed))
        .route("/sub", get(get_subs).post(sub).delete(unsub))
        .route("/read", post(read))
        .route(
            "/channel",
            get(get_channels).post(post_channel).delete(delete_channel),
        )
        .route_service("/logs", ServeFile::new("error_log.xml"))
        .with_state(Appstate { dbconn })
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
        .layer(cors);

    let sched = JobScheduler::new().await.unwrap();
    sched
        .add(
            // every 30 mins
            Job::new_async("0 0,30 * * * *", |_, _| {
                Box::pin(async {
                    update_feed_task().await;
                })
            })
            .unwrap(),
        )
        .await
        .unwrap();

    sched.start().await.unwrap();

    match env::var("IS_DOCKER_COMPOSED") {
        Ok(_) => {
            // docker environment: run on https
            let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
            let config = RustlsConfig::from_pem_file("ssl/cert.pem", "ssl/key.pem")
                .await
                .unwrap();
            axum_server::bind_rustls(addr, config)
                .serve(app.into_make_service())
                .await
                .unwrap();
        }
        Err(_) => {
            // dev environment: run on http
            let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
            axum::serve(listener, app).await.unwrap();
        }
    };
}

#[debug_handler]
async fn all_posts(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<Appstate>,
) -> Result<Json<Vec<Post>>, (StatusCode, String)> {
    let uid = params.get("uid").map(|x| x.parse::<u64>());
    let uid = match uid {
        Some(Ok(val)) => val,
        _ => return Err((StatusCode::BAD_REQUEST, "Invalid id passed!".to_string())),
    };
    let offset = params.get("offset").map(|x| x.parse::<u64>());
    let offset = match offset {
        Some(Ok(val)) => val,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                "Invalid offset passed!".to_string(),
            ))
        }
    };

    let res = state.dbconn.get_post_list(uid, offset).await;

    match res {
        Ok(val) => Ok(Json(val)),
        Err(e) => {
            event!(
                Level::ERROR,
                backtrace = ?e,
                description = e.desc,
                uid,
                offset
            );
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "We had some issues with the request...".to_string(),
            ))
        }
    }
}

#[derive(Deserialize)]
struct ReadQuery {
    id: Option<u64>,
    url: String,
    scrape: bool,
}
#[debug_handler]
async fn read(
    State(state): State<Appstate>,
    Json(payload): Json<ReadQuery>,
) -> Result<Json<Post>, (StatusCode, String)> {
    let to_scrape = payload.scrape;

    // EXPLANATION FOR MY OWN LEARNING:
    // Was getting an issue regarding tokio -> that dyn Error does not implement send and thus cannot be
    // held across an await. I figured that the value would be free'd before the match statement, as it
    // was not used later, but that was not the case. The `res` variable lived past the below .await, and hence would error
    // To terminate the lifeline early and hence not cause this error, we execute get_post in its own scope and error handle there
    // At the end of the scope, `res` gets dropped and we can use the other await.
    let mut post = {
        let res = state
            .dbconn
            .get_post(payload.id, Some(payload.url.to_string()))
            .await;

        match res {
            Ok(val) => val,
            Err(e) => {
                event!(
                    Level::ERROR,
                    backtrace = ?e,
                    description = e.desc,
                    url = ?payload.url,
                    id = ?payload.id
                );
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Error resource doesn't exist".to_string(),
                ));
            }
        }
    };

    if to_scrape {
        let res = web_scraper::scrape(&mut post).await;
        match res {
            Ok(_) => Ok(Json(post)),
            Err(e) => {
                event!(
                    Level::ERROR,
                    backtrace = ?e,
                    description = e.to_string(),
                    url = ?post.link,
                );
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to scrape post!".to_string(),
                ))
            }
        }
    } else {
        Ok(Json(post))
    }
}

async fn sub(
    State(state): State<Appstate>,
    Json(payload): Json<Subscription>,
) -> Result<(), (StatusCode, String)> {
    let res = state
        .dbconn
        .subscribe(payload.cid, payload.url.to_string())
        .await;
    match res {
        Ok(()) => Ok(()),
        Err(e) => {
            event!(
                Level::ERROR,
                backtrace = ?e,
                description = e.desc,
                url = ?payload.url,
            );
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                e.friendly_desc.unwrap_or("".to_string()) + &e.desc,
            ))
        }
    }
}

async fn unsub(
    State(state): State<Appstate>,
    Json(payload): Json<Subscription>,
) -> Result<(), (StatusCode, String)> {
    let res = state
        .dbconn
        .unsubscribe(payload.pid.unwrap_or(0), payload.cid)
        .await;
    match res {
        Ok(()) => Ok(()),
        Err(e) => {
            event!(
                Level::ERROR,
                backtrace = ?e,
                description = e.to_string(),
                id = payload.pid,
            );
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "We had some issues with the request...".to_string(),
            ))
        }
    }
}

#[debug_handler]
async fn get_subs(
    State(state): State<Appstate>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<Subscription>>, (StatusCode, String)> {
    let res = params.get("cid").map(|x| x.parse::<u64>());
    match res {
        Some(Ok(cid)) => {
            let urls = state.dbconn.get_subbed(cid).await;

            match urls {
                Ok(vecs) => Ok(Json(vecs)),
                Err(e) => {
                    event!(
                        Level::ERROR,
                        backtrace = ?e,
                        description = e.to_string(),
                    );
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "We had some issues with the request...".to_string(),
                    ))
                }
            }
        }
        Some(Err(_)) => {
            return Err((
                StatusCode::BAD_REQUEST,
                "Invalid ID value passed".to_string(),
            ))
        }
        None => match params.get("uid").map(|x| x.parse::<u64>()) {
            Some(Ok(uid)) => {
                let urls = state.dbconn.get_subbed_for_user(uid).await;

                match urls {
                    Ok(vecs) => Ok(Json(vecs)),
                    Err(e) => {
                        event!(
                            Level::ERROR,
                            backtrace = ?e,
                            description = e.to_string(),
                            uid = uid,
                        );
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "We had some issues with the request...".to_string(),
                        ))
                    }
                }
            }
            Some(Err(_)) => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    "Invalid ID value passed".to_string(),
                ))
            }
            None => return Err((StatusCode::BAD_REQUEST, "Invalid params!".to_string())),
        },
    }
}

// Feed gets the front page aggregated posts from the user's rss feed
// EXPECTED QUERY PARAMS: uid
#[debug_handler]
async fn feed(
    State(state): State<Appstate>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<Post>>, (StatusCode, String)> {
    let res = params.get("cid").map(|x| x.parse::<u64>());
    let cid = match res {
        Some(Ok(val)) => val,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                "Invalid ID value passed".to_string(),
            ))
        }
    };

    // get the list of all publications user is subscribed to
    let urls = state.dbconn.get_subbed(cid).await.unwrap();

    // using all the rss links, get all the posts from the xml feeds
    let data = rss_parser::get_whole_feed(urls).await;

    // insert posts into database for the 'all' page
    let res = state.dbconn.insert_posts(&data).await;
    if let Err(e) = res {
        // We don't need to necessarily return an error to the consumer here
        event!(
            Level::ERROR,
            backtrace = ?e,
            description = e.to_string(),
            // want to log data here but have to implement display i think. lazy rn.
        );
    }

    Ok(Json(data))
}

#[debug_handler]
async fn get_channels(
    State(state): State<Appstate>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<Channel>>, (StatusCode, String)> {
    let uid = match params.get("uid").map(|x| x.parse::<u64>()) {
        Some(Ok(val)) => val,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                "Missing required `uid` field".to_string(),
            ))
        }
    };

    match state.dbconn.get_channels_for_user(uid).await {
        Ok(channels) => Ok(Json(channels)),
        Err(e) => {
            event!(
                Level::ERROR,
                backtrace = ?e,
                description = e.to_string(),
            );
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "We had an issue with the request..".to_string(),
            ))
        }
    }
}

#[derive(Deserialize)]
struct CreateChannel {
    uid: u64,
    name: String,
}
#[debug_handler]
async fn post_channel(
    State(state): State<Appstate>,
    Json(payload): Json<CreateChannel>,
) -> Result<(), (StatusCode, String)> {
    match state
        .dbconn
        .insert_channel_for_user(payload.uid, payload.name)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => {
            event!(
                Level::ERROR,
                backtrace = ?e,
                description = e.to_string(),
            );
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "We had an issue with the request..".to_string(),
            ))
        }
    }
}

async fn delete_channel(
    State(state): State<Appstate>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<(), (StatusCode, String)> {
    let uid = match params.get("uid").map(|x| x.parse::<u64>()) {
        Some(Ok(val)) => val,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                "Missing required `uid` field".to_string(),
            ))
        }
    };

    let cid = match params.get("cid").map(|x| x.parse::<u64>()) {
        Some(Ok(val)) => val,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                "Missing required `cid` field".to_string(),
            ))
        }
    };

    match state.dbconn.delete_channel_for_user(uid, cid).await {
        Ok(_) => Ok(()),
        Err(e) => {
            event!(
                Level::ERROR,
                backtrace = ?e,
                description = e.to_string(),
            );
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "We had an issue with the request..".to_string(),
            ))
        }
    }
}

async fn update_feed_task() {
    println!("Starting update feed task!");
    let dbconn = DatabaseConnection::new();
    let pubs = dbconn.get_all_publishers().await;
    match pubs {
        Ok(pubs) => {
            let data = rss_parser::get_whole_feed(pubs).await;
            let res = dbconn.insert_posts(&data).await;
            if let Err(e) = res {
                DetailedError::new_descriptive(Box::new(e), "Failed auto update script");
            }
        }
        Err(e) => {
            DetailedError::new_descriptive(Box::new(e), "Failed auto update script");
        }
    }
    println!("Finished update feed task!")
}

// TODO:
// script to refresh feeds
// END TODO
