use axum::{
    debug_handler,
    extract::{Json, Query, State},
    http::StatusCode,
    routing::{get, post},
    Router,
};
use rss_api::{database::DatabaseConnection, rss_parser, Post};
use serde::Deserialize;
use std::collections::HashMap;

mod scraper;

#[derive(Clone)]
struct Appstate {
    dbconn: DatabaseConnection,
}

#[tokio::main]
async fn main() {
    let dbconn = DatabaseConnection::new();

    let app = Router::new()
        .route("/all", post(all_posts))
        .route("/feed", get(feed))
        .route("/sub", post(sub))
        .route("/read", post(read))
        .with_state(Appstate { dbconn });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[debug_handler]
async fn all_posts(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<Appstate>,
) -> Result<Json<Vec<Post>>, (StatusCode, String)> {
    let uid = params.get("id").map(|x| x.parse::<u64>());
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
                    "Error resource don't exist".to_string(),
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
    uid: u32,
    publisher: String,
}
async fn sub(
    State(state): State<Appstate>,
    Json(payload): Json<Subscription>,
) -> Result<(), (StatusCode, String)> {
    let res = state.dbconn.subscribe(payload.uid, payload.publisher).await;
    match res {
        Ok(()) => Ok(()),
        Err(e) => {
            // LOG THIS!
            // It would be an error in mysql.
            println!("DEBUG: {e}");
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
    let res = params.get("id").map(|x| x.parse::<u64>());
    let uid = match res {
        Some(Ok(val)) => val,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                "Invalid ID value passed".to_string(),
            ))
        }
    };

    // get the list of all publications user is subscribed to
    let urls = state.dbconn.get_subbed(uid).await.unwrap();

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

// TODO:
// THINK ABOUT HOW MUCH DATA TO RETURN ON THE FEED PAGE. YOU DON'T NEED ALL OF THAT.

// STANDARDIZE QUERY STRING, URL PARAMS.

// PUBLISHER WITH SEPARATE FEEDS? PUBLICATIONS HAVE MULTIPLE RSS FEEDS: INCLUDE *NAME* -> IT'S IMPORTANT!!!
// -- Kind of already works, just need to make name field required
// -- question is, get name from atom/rss feed or provide?

// FURTHER IN THE FUTURE: AUTHENTICATION?
// CHANNELS FOR EACH USER?
//----- shouldn't be too hard of a fix, change user_pubs to channel_pubs
//----- each uid has 0* channels
//----- each channel has an id, and is linked to publications via channel pubs
// IMPLEMENT LOGGING
// FIREFOX READVIEW FALLBACK
