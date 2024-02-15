use chrono::{DateTime, Utc};
use serde::Serialize;

pub mod database;
pub mod rss_parser;

#[derive(Debug, Serialize)]
pub struct Post {
    // ID is optional as we don't provide it when creating an object
    id: Option<u64>,
    title: String,
    pub link: String,
    date: Option<DateTime<Utc>>,
    description: Option<String>,
    content: Option<String>,
    enclosure: Option<String>,
    // Note pid is PUBLISHER ID!
    pid: u64,
}

#[derive(Debug, Serialize)]
pub struct Channel {
    cid: u64,
    name: String,
}

impl Post {
    pub fn new() {}

    // used primarily for testing
    pub fn new_link(link: String) -> Post {
        Post {
            id: None,
            title: "".to_string(),
            link: link,
            date: None,
            description: None,
            content: None,
            enclosure: None,
            pid: 10000,
        }
    }

    pub fn set_content(&mut self, new_content: String) {
        self.content = Some(new_content);
    }

    pub fn get_content(&self) -> String {
        let str_content = &self.content;
        let str_content = str_content.to_owned();
        str_content.unwrap_or("".to_string())
    }
}

pub struct URLObject {
    url: String,
    pid: u64,
}

#[cfg(test)]
mod integrated_tests {
    use super::*;
    use database::DatabaseConnection;
    use rss_parser;

    #[tokio::test]
    async fn test_get_feed_insert_data_to_db() {
        // this should be a urls pointing to .xml files online
        let obj = URLObject {
            url: "https://raw.githubusercontent.com/Harjun751/rss-reader/main/rss-api/atom.xml"
                .to_string(),
            pid: 6,
        };
        let urls: Vec<URLObject> = vec![obj];
        let posts = rss_parser::get_whole_feed(urls).await;

        let conn = DatabaseConnection::new();
        let res = conn.insert_posts(&posts).await;
        match res {
            Ok(_) => {}
            Err(err) => {
                println!("Error: {}", err.to_string());
                assert!(false)
            }
        }
    }
}
