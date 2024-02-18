mod logger;
mod scraper;

use axum::{
    debug_handler,
    extract::{Json, Query, State},
    http::StatusCode,
    routing::{get, post},
    Router,
};
use http::Method;
use rss_api::{database::DatabaseConnection, rss_parser, Channel, Post, URLObject};
use serde::Deserialize;
use std::collections::HashMap;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::error;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{filter, layer::Layer};

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

    let cors = CorsLayer::new()
        // allow requests from any origin
        .allow_methods(Any)
        .allow_origin(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/all", get(all_posts))
        .route("/feed", get(feed))
        .route("/sub", get(get_subs).post(sub).delete(unsub))
        .route("/read", post(read))
        .route(
            "/channel",
            get(get_channels).post(post_channel).delete(delete_channel),
        )
        .with_state(Appstate { dbconn })
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[debug_handler]
async fn all_posts(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<Appstate>,
) -> Result<Json<Vec<Post>>, (StatusCode, String)> {
    let cid = params.get("cid").map(|x| x.parse::<u64>());
    let cid = match cid {
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

    let res = state.dbconn.get_post_list(cid, offset).await;

    match res {
        Ok(val) => Ok(Json(val)),
        Err(e) => {
            // LOG THIS!
            println!("Debug: {e}");
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
        let res = state.dbconn.get_post(payload.id, Some(payload.url)).await;

        match res {
            Ok(val) => val,
            Err(e) => {
                // LOG THIS!
                println!("Debug: {e}");
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Error resource doesn't exist".to_string(),
                ));
            }
        }
    };

    if to_scrape {
        let res = scraper::scrape(&mut post).await;
        match res {
            Ok(_) => Ok(Json(post)),
            // LOG THIS!
            Err(_) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to scrape post!".to_string(),
            )),
        }
    } else {
        Ok(Json(post))
    }
}

// Sub requires a post body that deserializes into the Subscription struct
#[derive(Deserialize)]
struct Subscription {
    cid: u64,
    pid: Option<u64>,
    publisher: Option<String>,
}
async fn sub(
    State(state): State<Appstate>,
    Json(payload): Json<Subscription>,
) -> Result<(), (StatusCode, String)> {
    let res = state
        .dbconn
        .subscribe(payload.cid, payload.publisher.unwrap())
        .await;
    match res {
        Ok(()) => Ok(()),
        Err(e) => {
            // LOG THIS!
            error!("{}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "We had some issues with the request...".to_string(),
            ))
        }
    }
}

async fn unsub(
    State(state): State<Appstate>,
    Json(payload): Json<Subscription>,
) -> Result<(), (StatusCode, String)> {
    // TODO: INVESTIGATE THIS BUG
    println!("pid: {}", payload.pid.unwrap());
    println!("cid: {}", payload.cid);
    let res = state
        .dbconn
        .unsubscribe(payload.cid, payload.pid.unwrap())
        .await;
    match res {
        Ok(()) => Ok(()),
        Err(e) => {
            // LOG THIS!
            error!("{}", e);
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
) -> Result<Json<Vec<URLObject>>, (StatusCode, String)> {
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

    let urls = state.dbconn.get_subbed(cid).await;

    match urls {
        Ok(vecs) => Ok(Json(vecs)),
        Err(e) => {
            // LOG THIS!
            error!("{}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "We had some issues with the request...".to_string(),
            ))
        }
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
        // LOG THIS!
        println!("DEBUG: ERROR INSERTING {}", e.to_string())
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
            // LOG THIS!
            println!("DEBUG: {e}");
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
            // LOG THIS!
            println!("DEBUG: {e}");
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
            // LOG THIS!
            println!("DEBUG: {e}");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "We had an issue with the request..".to_string(),
            ))
        }
    }
}

// TODO:
// STRIP HTML AND MAKE CUSTOM DESCRIPTION FIELD

// FURTHER IN THE FUTURE: AUTHENTICATION?

// FIND OUT WHY DELETE SUBSCRIPTION DOESN'T WORK!!!!!!!!

// FALLBACK:
// I can't implement readerview myself
// Thinking of some options:
// -- Run a separate container running readerview code, and pass in my data
// -- Run a separate container with dragnet, query it somehow.

// IMPLEMENT LOGGING: CT'D: MACRO TO COMPOSE, AND CTOR TAKING IN KWARGS
// macro for creating scrapers
// END TODO
