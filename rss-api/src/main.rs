use axum::{
    extract::{Json, State, Path},
    routing::{get, post},
    Router,
    debug_handler,
};
use serde::Deserialize;
use rss_api::database::DatabaseConnection;

#[derive(Clone)]
struct Appstate { dbconn: DatabaseConnection }

#[tokio::main]
async fn main() {
    let dbconn = DatabaseConnection::new();

    let app = Router::new()
        .route("/test", get(|| async {"Hello World!"}))
        .route("/feed", get(feed))
        .route("/sub", post(sub))
        .route("/read", get(read))
        .with_state(Appstate { dbconn });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn read(){

}

#[derive(Deserialize)]
struct Subscription {
    user_id: u32,
    publisher: String,
}

#[derive(Deserialize)]
struct Publisher {
    pid: u32,
    url: Option<u32>
}
// a struct taking in the trait subscription
// it deserializes from the post body automatically 
async fn sub(
    State(state): State<Appstate>,
    Json(payload): Json<Subscription>
) -> Result<(),()>
{
    println!("Hello {}, who wants to subscribe to {}", payload.user_id, payload.publisher);
    // TODO: ERROR HANDLING SO THAT THE WHOLE DAMN THING DOESN'T CRASH
    state.dbconn.subscribe(payload.user_id, payload.publisher).await;
    Ok(())
}

async fn feed(
    State(state): State<Appstate>,
    Path(uid): Path<u32>
){
    // get the list of all publications user is subscribed to
    let urls = state.dbconn.get_subbed(uid).await;
    
    // using all the rss links, get all the content and insert into database

    // return the posts in the database, limit 30?
}