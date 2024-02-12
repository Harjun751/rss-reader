use axum::{
    debug_handler,
    extract::{Json, Path, Query, State},
    routing::{get, post},
    Router,
};
use rss_api::rss_parser;
use rss_api::{database::DatabaseConnection, Post};
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
        .route("/test", get(|| async { "Hello World!" }))
        .route("/feed/:uid", get(feed))
        .route("/sub", post(sub))
        .route("/read", get(read))
        .with_state(Appstate { dbconn });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[debug_handler]
async fn read(
    State(state): State<Appstate>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Post>, String> {
    let post_id = params.get("id").unwrap().parse::<i64>().unwrap();
    let _to_scrape = params.get("scrape").unwrap();

    let res = state.dbconn.get_post(post_id).await;
    match res {
        Ok(val) => Ok(Json(val)),
        Err(e) => Err(e.to_string()),
    }
}

#[derive(Deserialize)]
struct Subscription {
    user_id: u32,
    publisher: String,
}

#[derive(Deserialize)]
struct Publisher {
    pid: u32,
    url: Option<u32>,
}
// a struct taking in the trait subscription
// it deserializes from the post body automatically
async fn sub(
    State(state): State<Appstate>,
    Json(payload): Json<Subscription>,
) -> Result<(), String> {
    println!(
        "Hello {}, who wants to subscribe to {}",
        payload.user_id, payload.publisher
    );
    // TODO: ERROR HANDLING SO THAT THE WHOLE DAMN THING DOESN'T CRASH
    let res = state
        .dbconn
        .subscribe(payload.user_id, payload.publisher)
        .await;
    match res {
        Ok(()) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

#[debug_handler]
async fn feed(State(state): State<Appstate>, Path(uid): Path<u32>) -> Json<Vec<Post>> {
    // get the list of all publications user is subscribed to
    let urls = state.dbconn.get_subbed(uid).await.unwrap();

    // using all the rss links, get all the posts from the xml feeds
    let data = rss_parser::get_whole_feed(urls).await;

    // insert into database - this is for scrolling later?
    let res = state.dbconn.insert_posts(&data).await;
    if let Err(e) = res {
        println!("DEBUG: ERROR INSERTING {}", e.to_string())
    }

    Json(data)
}

// TODO:
// THINK ABOUT HOW MUCH DATA TO RETURN ON THE FEED PAGE. YOU DON'T NEED ALL OF THAT.
// CREATE THE "ALL" PAGE - PAGINATED, ORDERED BY DATE
// WEB SCRAPER
