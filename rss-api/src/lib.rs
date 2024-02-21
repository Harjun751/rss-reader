use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub mod database;
pub mod logger;
pub mod rss_parser;

#[derive(Debug, Serialize)]
pub struct Post {
    id: u64,
    title: String,
    pub link: String,
    date: DateTime<Utc>,
    description: String,
    // Some feeds don't provide content
    content: Option<String>,
    // enclosure is the link to a resource
    enclosure: Option<String>,
    pid: u64,
}

#[derive(Debug, Serialize)]
pub struct Channel {
    cid: u64,
    name: String,
}

// Sub requires a post body that deserializes into the Subscription struct
#[derive(Deserialize, Serialize)]
pub struct Subscription {
    pub cid: u64,
    pub pid: Option<u64>,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub name: String,
}

impl Post {
    pub fn new() {}

    // used primarily for testing
    pub fn new_link(link: String) -> Post {
        Post {
            id: 0,
            title: "Test".to_string(),
            link: link,
            date: Utc::now(),
            description: "Test".to_string(),
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

#[cfg(test)]
mod integrated_tests {
    use super::*;
    use database::DatabaseConnection;
    use rss_parser;

    #[test]
    fn macro_test() {
        // let params = log_params!("unit", 42);
    }

    #[tokio::test]
    async fn test_get_feed_insert_data_to_db() {
        // this should be a urls pointing to .xml files online
        let obj = Subscription {
            url: "https://raw.githubusercontent.com/Harjun751/rss-reader/main/rss-api/test-files/atom.xml"
                .to_string(),
            pid: Some(5),
            cid: 1,
            name: "nil".to_string()
        };
        let urls: Vec<Subscription> = vec![obj];
        let posts = rss_parser::get_whole_feed(urls).await;
        assert!(posts.len() > 0);

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
