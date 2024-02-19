#![allow(non_snake_case)]

use std::fmt::Display;

use dioxus::{html::desc, prelude::*};
use dioxus_router::prelude::*;
use rss_frontend::{get_daily_feed, get_post_with_url, Post};

#[derive(Routable, PartialEq, Debug, Clone)]
pub enum Route{
    #[route("/")]
    DailyFeed {},

    #[route("/article?:article_params")]
    Article{
        article_params: ArticleParams,
    }
}

#[derive(Debug, Clone, PartialEq)]
struct ArticleParams {
    url: String,
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

fn main(){
    env_logger::init();
    dioxus_web::launch(App);
}

fn App(cx: Scope) -> Element {
    render!{
        Router::<Route>{}
    }
}

// "id": null,
// "title": "Ross Gelbspan, author who probed roots of climate change denial, dies at 84",
// "link": "https://www.washingtonpost.com",
// "date": "2024-02-18T00:06:16Z",
// "description": "Mr. Gelbspan, a longtime journalist, criticized the profession for giving a forum to those who sow doubts about global warming.",
// "content": null,
// "enclosure": null,
// "pid": 3

#[component]
fn DailyFeed(cx: Scope) -> Element{
    // TODO: HARDCODED CID VALUE
    let feed = use_future(cx, (), |_| get_daily_feed(1));
    match feed.value(){
        Some(Ok(list)) => {
            render!{
                for item in list {
                    FeedItem { post: item.clone() }
                }
            }
        }
        Some(Err(e)) => {
            render! {"An error occured when loading. {e}"}
        }
        None => {
            render! {"loading feed..."}
        }
    }
}

use chrono::{DateTime, Utc, Local};

#[component]
fn FeedItem(cx: Scope, post: Post) -> Element{
    let Post { id, title, link, date, description, content, enclosure, pid } = post;
    let date: DateTime<Local> = DateTime::from(date.clone());
    let date_formatted = date.format("%a, %b %d %Y");
    let time_formatted = date.format("%r");
    render!{
        Link{
            to: Route::Article { article_params: ArticleParams{ url: link.to_string() }},
            div {
                margin_bottom: "30px",
                div {
                    font_family: "\"Patua One\", serif",
                    font_size: "18px",
                    text_decoration: "underline",
                    "{title}"
                }
                div {
                    font_family: "\"Bitter\", serif",
                    font_size: "16px",
                    margin_top: "7px",
                    "{description}"
                }
                div {
                    font_family: "\"Bitter\", serif",
                    font_size: "14px",
                    margin_top: "7px",
                    color: "#808080",
                    "{date_formatted} • {time_formatted}"
                }
            }
        }
    }
}

#[component]
fn Article(cx: Scope, article_params: ArticleParams) -> Element {
    let mut scrape = false;
    let mut post = use_future(cx, &(scrape,), move |(scrape,)| get_post_with_url(article_params.url.clone(), scrape));
    let url = article_params.url.clone();
    match post.value(){
        Some(Ok(p)) => {
            let Post { id, title, link, date, description, content, enclosure, pid } = p;
            let date: DateTime<Local> = DateTime::from(date.clone());
            let date_formatted = date.format("%a, %b %d %Y");
            let time_formatted = date.format("%r");
            let mut content = use_state(cx, || match content{
                Some(val) => val.clone(),
                None => description.clone()
            });
            render!{
                div{ 
                    padding_bottom:"20px",
                    border_bottom: "3px dashed #808080",
                    div {
                        font_family: "\"Patua One\", serif",
                        font_size: "18px",
                        text_decoration: "underline",
                        "{title}"
                    }
                    div {
                        font_family: "\"Bitter\", serif",
                        font_size: "14px",
                        margin_top: "7px",
                        color: "#808080",
                        "{date_formatted} • {time_formatted}"
                    }
                    div {
                        font_family: "\"Bitter\", serif",
                        font_size: "16px",
                        margin_top: "7px",
                        "{content}"
                    }
                }
                div {
                    text_align: "center",
                    margin_top: "20px",
                    a {
                        href: "{link}",
                        font_family: "\"Patua One\", serif",
                        font_size: "14px",
                        color: "#808080",
                        text_decoration: "underline",
                        "Article Link"
                    }
                }
                // TODO: matching here to selectively show this button
                div {
                    div {
                        text_align: "center",
                        margin_top: "20px",
                        font_family: "\"Patua One\", serif",
                        font_size: "14px",
                        color: "#808080",
                        "Not the full Article?"
                    }
                    div {
                        button{
                            onclick: move |_| {
                                scrape = true;
                                post.restart();
                            },
                            "Engage Fallback"
                        }
                    }
                }
            }
        },
        Some(Err(e)) => render!{"{e}{url}"},
        None => render!{"Loading..."}
    }
}

// #[component]
// fn ArticleRender(cx: Scope, post: Post) -> Element{
    // let Post { id, title, link, date, description, content, enclosure, pid } = post;
    // let date: DateTime<Local> = DateTime::from(date.clone());
    // let date_formatted = date.format("%a, %b %d %Y");
    // let time_formatted = date.format("%r");
    // let mut content = match content{
    //     Some(val) => val,
    //     None => description
    // };
    // render!{
    //     div{ 
    //         padding_bottom:"20px",
    //         border_bottom: "3px dashed #808080",
    //         div {
    //             font_family: "\"Patua One\", serif",
    //             font_size: "18px",
    //             text_decoration: "underline",
    //             "{title}"
    //         }
    //         div {
    //             font_family: "\"Bitter\", serif",
    //             font_size: "14px",
    //             margin_top: "7px",
    //             color: "#808080",
    //             "{date_formatted} • {time_formatted}"
    //         }
    //         div {
    //             font_family: "\"Bitter\", serif",
    //             font_size: "16px",
    //             margin_top: "7px",
    //             "{content}"
    //         }
    //     }
    //     div {
    //         text_align: "center",
    //         margin_top: "20px",
    //         a {
    //             href: "{link}",
    //             font_family: "\"Patua One\", serif",
    //             font_size: "14px",
    //             color: "#808080",
    //             text_decoration: "underline",
    //             "Article Link"
    //         }
    //     }
    //     // TODO: matching here to selectively show this button
    //     div {
    //         div {
    //             text_align: "center",
    //             margin_top: "20px",
    //             font_family: "\"Patua One\", serif",
    //             font_size: "14px",
    //             color: "#808080",
    //             "Not the full Article?"
    //         }
    //         div {
    //             button{
    //                 onclick: move |ev| log::info!(ev),
    //                 "Engage Fallback"
    //             }
    //         }
    //     }
    // }
// }