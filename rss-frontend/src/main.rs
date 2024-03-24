// TODO: cookies, loading anims

#![allow(non_snake_case)]
mod querystructs;
use log::LevelFilter;
use dioxus::prelude::*;
use dioxus_router::prelude::*;
use rss_frontend::*;
use querystructs::*;
use chrono::{DateTime, Local};

#[derive(Routable, PartialEq, Debug, Clone)]
pub enum Route{
    #[layout(NavBar)]
        #[route("/")]
        DailyFeed {},


        #[route("/settings")]
        Settings {},

        #[route("/all")]
        AllFeed {},
    #[end_layout]

    #[route("/article?:article_params")]
    Article{
        article_params: ArticleParams,
    },

    #[route("/ch?:chparams")]
    ChannelSetting{
        chparams: ChParams,
    },

    #[route("/set")]
    Set{},

    #[route("/get")]
    Get{}
}

struct UID(u64);



fn main(){
    dioxus_logger::init(LevelFilter::Info).expect("failed to init logger");
    dioxus_web::launch(App);
}

fn App(cx: Scope) -> Element {
    // TODO: hardcoded uid - related to cookies
    use_shared_state_provider(cx, || UID(1));
    render!{
        Router::<Route>{}
    }
}

#[component]
fn NavBar(cx: Scope) -> Element {
    render! {
        nav {
            div { Link { to: Route::DailyFeed {}, "Today" } }
            div { Link { to: Route::AllFeed {}, "All" } }
            div {
                float: "right",
                margin_right:"20px",
                Link{
                    to: Route::Settings {},
                    img{
                        src:"./assets/cog.png",
                        style: "height:30px;",
                        position: "relative",
                        top: "7px",
                    }
                }   
            }
        }
        Outlet::<Route> {}
    }
}

#[component]
fn Set(cx: Scope) -> Element{
    let fut = use_future(cx, (), |_| set_pref());
    render!(match fut.value(){
        Some(Ok(_))=>{
            rsx!(p{"woohoo"})
        },
        _ => {
            rsx!(p{"aww :("})
        }
    })
}
#[component]
fn Get(cx: Scope) -> Element{
    let fut = use_future(cx, (), |_| get_pref());
    render!(match fut.value(){
        Some(Ok(val))=>{
            rsx!(p{"got your preferences dawg. {val}"})
        },
        Some(Err(val)) => {
            rsx!(p{"failed to get your preferences dawg. {val}"})
        }
        _ => {
            rsx!(p{"aww :("})
        }
    })
}

#[component]
fn DailyFeed(cx: Scope) -> Element{
    let uid = use_shared_state::<UID>(cx).unwrap().read().0;
    let channel_future = use_future(cx, (), |_| get_channels(uid));
    let cid = use_state(cx, || 1);
    let post_future = use_future(cx, cid, |cid| get_daily_feed(*cid));
    
    match channel_future.value() {
        Some(Ok(channels)) => {
            let ch1 = channels.get(0);
            match ch1{
                Some(ch) => {
                    cid.set(ch.cid);
                    cx.render(
                        rsx!{
                            div{
                                display: "flex",
                                align_items: "center",
                                margin_bottom: "40px",
                                margin_top:"20px",
                                div {
                                    class: "fancy-select",
                                    select{
                                        all: "unset",
                                        class: "fancy-select",
                                        font_family: "\"Patua One\", serif",
                                        font_size: "20px",
                                        background: "var(--light-secondary)",
                                        padding: "5px 32px 5px 10px",
                                        onchange: |e| {
                                            let new_cid = &e.value.parse::<u64>().unwrap();
                                            cid.set(*new_cid);
                                        },
                                        for ch in channels {
                                            option{
                                                value: ch.cid as i64,
                                                "{ch.name}"
                                            }
                                        }
                                    }
                                }
                            }
                            match post_future.state(){
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
                                            rsx!{"failed to load..."}
                                        }
                                    }
                                },
                                _ => {
                                    rsx!{"loading feed.."}
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
            render!("failed loading channels...")
        }
        None => {
            render!("loading channels...")
        }
    }
}

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
fn AllFeed(cx: Scope) -> Element{
    // use future?
    let posts: &UseState<Vec<Post>> = use_state(cx, || vec![]);
    let counter = use_state(cx, || 0);
    const INCREMENT: u64 = 10;
    use_future(cx, (counter,), |(counter,)| {
        let posts = posts.to_owned();
        let err_eval = use_eval(cx).to_owned();
        async move {
            match get_all_posts(*counter).await {
                Ok(mut p) => {
                    posts.with_mut(|x| {
                        (*x).append(&mut p);
                    });
                },
                Err(e) => {
                    err_eval(ERROR_ALERT_SCRIPT).unwrap();
                    log::error!("{e}");
                }
            }
        }
    });

    
    let create_eval = use_eval(cx);

    // You can create as many eval instances as you want
    let eval = create_eval(
        r#"
        const handleInfiniteScroll = () => {

            const endOfPage = window.innerHeight + window.pageYOffset >= document.body.offsetHeight;
            dioxus.send(endOfPage);
          
          };
          window.addEventListener("scroll", handleInfiniteScroll);
          // snippet taken more or less as a whole from:
          // https://webdesign.tutsplus.com/how-to-implement-infinite-scrolling-with-javascript--cms-37055t
        "#,
    )
    .unwrap();

    use_future(cx, (), |_| {
        to_owned![eval];
        let counter = counter.to_owned();
        async move {
            loop {
                let message = eval.recv().await.unwrap();
                if let "true" = message.to_string().as_str() {
                    // modify counter state, so that future restarts, getting more posts
                    counter.modify(|x| x + INCREMENT);
                }
            }
        }
    });

    render!{
        div{
            margin_top: "40px",
            for item in posts.iter() {
                FeedItem { post: item.clone() }
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
                        article{ 
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
                                        Some(val) => rsx!{ div{ dangerous_inner_html: "{val}" }},
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
    let uid = use_shared_state::<UID>(cx).unwrap().read().0;
    let channels = use_future(cx, (), |_| get_channels(uid) );
    let ch_name = use_state(cx, || "".to_string());
    match channels.value(){
        Some(Ok(val)) => {
            render!(
                rsx!{
                    div{
                        padding:"0 15px",
                        margin_top: "20px",
                        h1{
                            "Settings"
                        }
                        h2{
                            margin_top:"20px",
                            "Channels"
                        }
    
                        div{
                            margin:"20px auto",
                            width: "96%",
                            class: "fw-input",
                            input{
                                name: "channel_name",
                                style: "width:80%;height:33px;",
                                box_sizing: "border-box",
                                border: "1px solid black",
                                border_right: "none",
                                value: "{ch_name}",
                                placeholder:"Enter channel name...",
                                oninput: move |evt| ch_name.set(evt.value.clone()),
                            },
                            input {
                                r#type: "submit",
                                style: "width:20%;height:33px;",
                                value :"Add",
                                border: "1px solid black",
                                border_left: "none",
                                onclick: move |_| {
                                    cx.spawn({
                                        let val = ch_name.to_string();
                                        let channels = channels.clone();
                                        let err_eval = use_eval(cx).to_owned();

                                        async move{
                                            match create_channel(uid, val).await{
                                                Ok(_) => {
                                                    channels.restart()
                                                },
                                                Err(e) => {
                                                    err_eval(ERROR_ALERT_SCRIPT).unwrap();
                                                    log::error!("{e}");
                                                }
                                            }
                                        }
                                    });
                                }
                            }
                        }
    
                        table{
                            for ele in val{
                                tr{
                                    td{
                                        Link{
                                            to: Route::ChannelSetting { chparams: ChParams{cid: ele.cid} },
                                            p{
                                                padding_left:"20px",
                                                margin:"0px",
                                                height:"34px",
                                                line_height:"34px",
                                                "{ele.name}"
                                            }
                                        }
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
    let subs = use_future(
        cx,
        (),
        move |_| get_subscription_for_channel(chparams.cid)
    );
    let url = use_state(cx, || "".to_string());
    let uid = use_shared_state::<UID>(cx).unwrap().read().0;
    let nav = use_navigator(cx);

    let create_eval = use_eval(cx);
    let alert_box = |text| {
        format!(
            r#"
            swal("Are you sure you want to delete this {text}?", {{
                buttons: {{
                    cancel: {{
                        text: "Cancel",
                        value: false,
                        visible: true,
                    }},
                        confirm: {{
                        text: "Delete",
                        value: true,
                        visible: true,
                    }}
                }},
            }})
            .then((value) => {{
                switch (value) {{
                    case true:
                        dioxus.send(true);
                        console.log("true");
                    case false:
                        dioxus.send(false);
                        console.log("false");
                }}
            }})
            ;
            "#
        )
    };

    

    render!(
        div{
            padding:"0 15px",
            h1{
                "Settings"
            }
            h2{
                margin_top:"20px",
                "Subscribed Feeds"
            }
    
            div{
                margin:"20px auto",
                width: "96%",
                class: "fw-input",
    
                input{
                    name: "url",
                    style: "width:80%;height:33px;",
                    border: "1px solid black",
                    border_right: "none",
                    value: "{url}",
                    placeholder:"Enter URL here...",
                    box_sizing: "border-box",
                    oninput: move |evt| url.set(evt.value.clone()),
                },
                input {
                    r#type: "submit",
                    style: "width:20%;height:33px;",
                    value :"Add",
                    border_radius: 0,
                    border: "1px solid black",
                    border_left: "none",
                    onclick: move |_| {
                        cx.spawn({
                            let cid = chparams.cid;
                            let val = url.to_string();
                            let subs = subs.clone();
                            let err_eval = use_eval(cx).to_owned();

                            async move {
                                match subscribe(cid, val).await{
                                    Ok(_) => {
                                        subs.restart();
                                    },
                                    Err(e) => {
                                        err_eval(ERROR_ALERT_SCRIPT).unwrap();
                                        log::error!("{e}");
                                    }
                                }
                            }
                        })
                    }
                }
            }
    
            match subs.value(){
                Some(Ok(val)) => {
                    rsx!(table{
                        margin_top:"20px",
                        for (i, ele) in val.iter().enumerate(){
                            tr{
                                td{
                                    cursor: "pointer",
                                    onclick: move |_| {
                                        let alert = alert_box("subscription");
                                        cx.spawn({
                                            let cid = ele.cid;
                                            let pid = ele.pid;
                                            let subs = subs.clone();
                                            let eval = create_eval(&alert).unwrap();
                                            let err_eval = use_eval(cx).to_owned();
    
                                            async move{
                                                let message = eval.recv().await.unwrap();
                                                if let "true" = message.to_string().as_str() {
                                                    match unsubscribe(cid, pid).await{
                                                        Ok(_) => {
                                                            subs.restart();
                                                        },
                                                        Err(e) => {
                                                            err_eval(ERROR_ALERT_SCRIPT).unwrap();
                                                            log::error!("{e}");
                                                        }
                                                    }
                                                }
                                            }
                                        });
                                    },
                                    p{
                                        padding_left:"20px",
                                        margin:"0px",
                                        height:"34px",
                                        line_height:"34px",
                                        "{ele.name} ({ele.url})"
                                    }
                                }
                            }
                        }
                    })
                }
                _ => {
                    rsx!("loading.")
                }
            }
    
            hr{ margin_top: "20px" }
            div{
                text_align:"center",
                h2{
                    text_decoration:"None",
                    margin_top: "40px",
                    "DANGER ZONE!!!"
                }
                button{
                    color: "black",
                    cursor: "pointer",
                    font_family: "\"Patua One\", serif",
                    font_size: "14px",
                    background_color: "white",
                    padding: "13px 10px",
                    border: "3px solid black",
                    margin_top: "40px",
                    onclick: move |_| {
                        let alert = alert_box("channel");
                        cx.spawn({
                            let cid = chparams.cid;
                            let nav = nav.clone();
                            let eval = create_eval(&alert).unwrap();
                            let err_eval = use_eval(cx).to_owned();
    
                            async move{
                                let message = eval.recv().await.unwrap();
                                if let "true" = message.to_string().as_str() {
                                    match delete_channel(uid, cid).await{
                                        Ok(_) => {
                                            nav.push(Route::Settings {  });
                                        },
                                        Err(e) => {
                                            err_eval(ERROR_ALERT_SCRIPT).unwrap();
                                            log::error!("{e}");
                                        }
                                    }
                                }
                            }
                        });
                    },
                    "DELETE CHANNEL"
                }
            }
        }
    )
}

const ERROR_ALERT_SCRIPT: &str = r#"
swal("Unfortunately, an error occured. :(");
;
"#;