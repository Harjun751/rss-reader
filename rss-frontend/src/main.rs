#![allow(non_snake_case)]
mod querystructs;
use log::LevelFilter;
use dioxus::prelude::*;
use dioxus_router::prelude::*;
use rss_frontend::{get_channels, get_daily_feed, get_post_with_url, get_subscription_for_channel, subscribe, unsubscribe, Channel, Post, Subscription};
use querystructs::*;

#[derive(Routable, PartialEq, Debug, Clone)]
pub enum Route{
    #[route("/")]
    DailyFeed {},

    #[route("/article?:article_params")]
    Article{
        article_params: ArticleParams,
    },

    #[route("/ch?:chparams")]
    ChannelSetting{
        chparams: ChParams,
    },

    #[route("/settings")]
    Settings {}
}



fn main(){
    dioxus_logger::init(LevelFilter::Info).expect("failed to init logger");
    dioxus_web::launch(App);
}

fn App(cx: Scope) -> Element {
    render!{
        Router::<Route>{}
    }
}

#[component]
fn DailyFeed(cx: Scope) -> Element{
    // TODO: HARDCODED UID VALUE
    let fut = use_future(cx, (), |_| get_channels(1));
    
    match fut.value() {
        Some(Ok(channels)) => {
            let ch1 = channels.get(0);
            match ch1{
                Some(ch) => {
                    let cid = use_state(cx, || ch.cid);
                    let post_fut = use_future(cx, cid, |cid| get_daily_feed(*cid));
                    cx.render(
                        rsx!{

                            div{
                                display: "flex",
                                align_items: "center",
                                margin_bottom: "40px",
                                div{
                                    h1{
                                        "Today's Feed:"
                                    }
                                }
                                div {
                                    class: "fancy-select",
                                    select{
                                        all: "unset",
                                        class: "fancy-select",
                                        margin_left: "20px",
                                        font_family: "\"Patua One\", serif",
                                        font_size: "20px",
                                        background: "#D9D9D9",
                                        padding: "5px 32px 5px 10px",
                                        onchange: |e| {
                                            let new_cid = &e.value.parse::<u64>().unwrap();
                                            log::error!("New cid: {new_cid}");
                                            cid.set(*new_cid);
                                        },
                                        for ch in channels {
                                            option{
                                                // font_family: "\"Patua One\", serif",
                                                value: ch.cid as i64,
                                                "{ch.name}"
                                            }
                                        }
                                    }
                                }
                                // reminder for blog lol: talk about how it was kinda annoying to do a simple thing like this\
                                div {
                                    margin_left: "auto",
                                    margin_right: "30px",
                                    Link{
                                        to: Route::Settings {},
                                        img{
                                            src:"/assets/cog.png",
                                            width: "50px",
                                        }
                                    }   
                                }
                            }
                            match post_fut.state(){
                                UseFutureState::Complete(value) => {
                                    match value {
                                        Ok(posts) => {
                                            if posts.len() == 0 {
                                                rsx!{"subscribe to some stuff."}
                                            } else {
                                                rsx!{
                                                    for item in posts {
                                                        FeedItem { post: item.clone() }
                                                    }
                                                }
                                            }
                                        },
                                        Err(e) => {
                                            rsx!{"failed to load, bro."}
                                        }
                                    }
                                },
                                _ => {
                                    rsx!{"loading.."}
                                }
                            }
                        }
                    )
                },
                None => {
                    render!("subscribe to some channels bro.")
                }
            }
        },
        Some(Err(e)) => {
            render!("failed loading...")
        }
        None => {
            render!("loading...")
        }
    }
}

use chrono::{DateTime, Local};

#[component]
fn FeedItem(cx: Scope, post: Post) -> Element{
    let Post { title, link, date, description, .. } = post;
    let date: DateTime<Local> = DateTime::from(date.clone());
    let date_formatted = date.format("%a, %b %d %Y");
    let time_formatted = date.format("%r");
    render!{
        Link{
            to: Route::Article { article_params: ArticleParams{ url: link.to_string() }},
            div {
                margin_bottom: "30px",
                div {
                    h2{
                        "{title}"
                    }
                }
                div {
                    p{
                        "{description}"
                    }
                }
                div {
                    small{
                        "{date_formatted} • {time_formatted}"
                    }
                }
            }
        }
    }
}

#[component]
fn Article(cx: Scope, article_params: ArticleParams) -> Element {
    let scrape = use_state(cx, || false);
    let post = use_future(cx, scrape, |scrape| get_post_with_url(article_params.url.clone(), *scrape) );
    let url = article_params.url.clone();
    
    cx.render(match post.state(){
        UseFutureState::Complete(value) => {
            match value {
                Ok(p) => {
                    let Post { id, title, link, date, description, content, enclosure, pid } = p;
                    let date: DateTime<Local> = DateTime::from(date.clone());
                    let date_formatted = date.format("%a, %b %d %Y");
                    let time_formatted = date.format("%r");
                    rsx!{
                        div{ 
                            padding_bottom:"20px",
                            border_bottom: "3px dashed #808080",
                            div {
                                h2{
                                    "{title}"
                                }
                            }
                            div {
                                small{
                                    "{date_formatted} • {time_formatted}"
                                }
                            }
                            div {
                                p{
                                    match content {
                                        Some(val) => rsx!{"{val}"},
                                        None => rsx!{"{description}"}
                                    }
                                }
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
                        if **scrape {
                            rsx!{
                                div {
                                    div {
                                        text_align: "center",
                                        margin_top: "20px",
                                        font_family: "\"Patua One\", serif",
                                        font_size: "14px",
                                        color: "#808080",
                                        "Not working too well?"
                                    }
                                    div {
                                        button{
                                            color: "#808080",
                                            font_family: "\"Patua One\", serif",
                                            font_size: "14px",
                                            background_color: "white",
                                            padding: "13px 10px",
                                            border: "3px solid #808080",
                                            display: "block",
                                            margin: "auto",
                                            margin_top: "20px",
                                            onclick: |_| {
                                                scrape.set(false)
                                            },
                                            "Disable Fallback"
                                        }
                                    }
                                }
                            }
                        } else {
                            rsx!{
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
                                            color: "#808080",
                                            font_family: "\"Patua One\", serif",
                                            font_size: "14px",
                                            background_color: "white",
                                            padding: "13px 10px",
                                            border: "3px solid #808080",
                                            display: "block",
                                            margin: "auto",
                                            margin_top: "20px",
                                            onclick: |_| {
                                                scrape.set(true)
                                            },
                                            "ENGAGE FALLBACK"
                                        }
                                    }
                                }
                            }
                        }
                        
                    }
                },
                Err(e) =>{
                    rsx!("{e} {url}")
                }
            }
        },
        _ => {
            rsx!{"loading..."}
        }
    })
}

#[component]
fn Settings(cx: Scope) -> Element{
    // todo: hardcoded id
    let channels = use_future(cx, (), |_| get_channels(1) );
    match channels.value(){
        Some(Ok(val)) => {
            render!(
                rsx!{
                    h1{
                        "Settings"
                    }
                    h2{
                        margin_top:"20px",
                        margin_bottom:"20px",
                        "Channels"
                    }
                    table{
                        for ele in val{
                            tr{
                                Link{
                                    to: Route::ChannelSetting { chparams: ChParams{cid: ele.cid} },
                                    td{
                                        "{ele.name}"
                                    }
                                }
                            }
                        }
                    }
                }
            )
        }
        _ => {
            render!("loading")
        }
    }
}

#[component]
fn ChannelSetting(cx: Scope, chparams: ChParams) -> Element{
    let subs = use_future(cx, (), |_| get_subscription_for_channel(chparams.cid) );
    let url = use_state(cx, || "".to_string());

    render!(
        h2{
            "Subscribed Feeds"
        }


        input{
            name: "url",
            value: "{url}",
            placeholder:"Enter URL here...",
            oninput: move |evt| url.set(evt.value.clone()),
        },
        input {
            r#type: "submit",
            value :"Add",
            onclick: move |_| {
                log::error!("Curr value: {url}");
                let di = cx.spawn({
                    let cid = chparams.cid;
                    let val = url.to_string();
                    async move{
                        match subscribe(cid, val).await{
                            Ok(_) => {
                                log::error!("Hooray!");
                            },
                            Err(e) => {
                                log::error!("{:#?} \n. daman.", e)
                            }
                        }
                    }
                });
            }
        }

        match subs.value(){
            Some(Ok(val)) => {
                rsx!(table{
                    for (i, ele) in val.iter().enumerate(){
                        tr{
                            td{
                                onclick: |_| {
                                    cx.spawn({
                                        let cid = ele.cid;
                                        let pid = ele.pid;

                                        async move{
                                            match unsubscribe(cid, pid).await{
                                                Ok(_) => {
                                                    log::error!("Hooray!");
                                                },
                                                Err(e) => {
                                                    log::error!("{:#?} \n. daman.", e)
                                                }
                                            }
                                        }
                                    });
                                },
                                "{ele.name} ({ele.url})"
                            }
                        }
                    }
                })
            }
            _ => {
                rsx!("loading.")
            }
        }
    )
}