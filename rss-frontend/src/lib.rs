use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

pub static API_URL: &str = "http://localhost:3000/";

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
// pub struct Post {
//     // ID is optional as we don't provide it when creating an object
//     pub id: Option<u64>,
//     pub title: String,
//     pub link: String,
//     pub date: Option<DateTime<Utc>>,
//     pub description: Option<String>,
//     pub content: Option<String>,
//     pub enclosure: Option<String>,
//     // Note pid is PUBLISHER ID!
//     pub pid: u64,
// }
pub struct Post {
    pub id: u64,
    pub title: String,
    pub link: String,
    pub date: DateTime<Utc>,
    pub description: String,
    // Some feeds don't provide content
    pub content: Option<String>,
    // enclosure is the link to a resource
    pub enclosure: Option<String>,
    pub pid: u64,
}

pub async fn get_daily_feed(cid: u64) -> Result<Vec<Post>, reqwest::Error>{
    let url = format!("{}{}{}",API_URL , "feed?cid=", cid);
    reqwest::get(&url).await?.json().await
}
#[derive(Serialize)]
struct GetPostData{
    id: Option<u64>,
    url: String, 
    scrape: bool,
}
pub async fn get_post_with_url(url: String, scrape: bool) -> Result<Post, reqwest::Error>{
    let endpoint = format!("{}{}",API_URL , "read");
    let data = GetPostData { id: None, url: url, scrape: scrape};
    let cli = reqwest::Client::new();
    cli.post(endpoint)
        .json(&data)
        .send().await?
        .json::<Post>().await
}