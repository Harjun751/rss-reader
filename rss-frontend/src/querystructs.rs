use std::fmt::Display;
use dioxus_router::prelude::FromQuery;
#[derive(Debug, Clone, PartialEq)]
pub struct ArticleParams {
    pub url: String,
}
/// The display impl needs to display the query in a way that can be parsed:
impl Display for ArticleParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "url={}", self.url)
    }
}

impl FromQuery for ArticleParams{
    fn from_query(query: &str) -> Self {
        let mut url = None;
        let pairs = form_urlencoded::parse(query.as_bytes());
        pairs.for_each(|(key, value)| {
            if key == "url" {
                url = Some(value.clone().into());
            }
        });
        Self {
            url: url.unwrap()
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChParams {
    pub cid: u64,
}
/// The display impl needs to display the query in a way that can be parsed:
impl Display for ChParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "id={}", self.cid)
    }
}

impl FromQuery for ChParams{
    fn from_query(query: &str) -> Self {
        let mut id = None;
        let pairs = form_urlencoded::parse(query.as_bytes());
        pairs.for_each(|(key, value)| {
            if key == "id" {
                id = Some(value.clone().parse::<u64>());
            }
        });
        Self {
            cid: id.unwrap().unwrap()
        }
    }
}

