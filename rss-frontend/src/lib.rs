use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

pub static API_URL: &str = "http://localhost:3000/";

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
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

#[derive(Deserialize)]
pub struct Channel {
    pub cid: u64,
    pub name: String,
}
pub async fn get_channels(id: u64) -> Result<Vec<Channel>, reqwest::Error>{
    let endpoint = format!("{}{}{}",API_URL , "channel?uid=", id);
    reqwest::get(&endpoint).await?.json().await
}

pub async fn get_channels_and_feed(id: u64) -> Result<(Vec<Channel>, Vec<Post>), String> {
    let channels = get_channels(id).await.map_err(
        |_| "Server Error!"
    )?;
    match channels.get(0) {
        Some(ch) => {
            let feed = get_daily_feed(ch.cid).await.map_err(
                |_| "Server Error!"
            )?;
            Ok((channels, feed))
        },
        None => {
            // return empty tuple because user has no channels yet
            Ok((vec![], vec![]))
        }
    }
}

#[derive(Deserialize)]
pub struct Subscription {
    pub cid: u64,
    pub pid: u64,
    pub url: String,
    pub name: String,
}
pub async fn get_subscription_for_channel(id: u64) -> Result<Vec<Subscription>,reqwest::Error>{
    let url = format!("{}{}{}",API_URL , "sub?cid=", id);
    reqwest::get(&url).await?.json().await
}

struct UnsubData{
    cid: u64,
    pid: u64,
}
pub async fn unsubscribe(cid: u64, pid: u64) -> Result<(), String> {
    let endpoint = format!("{}{}",API_URL , "sub");
    let cli = reqwest::Client::new();
    let resp = cli.delete(endpoint)
        .send().await;

    match resp {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string())
    }
}