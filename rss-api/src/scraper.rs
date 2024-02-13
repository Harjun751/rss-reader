// use async_trait::async_trait;
use axum::async_trait;
use roxmltree::{self, ParsingOptions};
use rss_api::rss_parser::from_url;
use rss_api::Post;
use scraper::{Html, Selector};
use std::fs;

#[async_trait]
pub trait Scraper {
    async fn scrape(&self, p: Post) -> Result<String, Box<dyn std::error::Error>>;
}

enum Sites {
    TheVerge,
    Wired,
}

#[async_trait]
impl Scraper for Sites {
    async fn scrape(&self, post: Post) -> Result<String, Box<dyn std::error::Error>> {
        let data = from_url(&post.link).await?;

        // let fsd = fs::read(post.link).unwrap();
        // let data = String::from_utf8_lossy(&fsd);

        match self {
            Self::TheVerge => the_verge(&data),
            Self::Wired => wired(&data),
        }
    }
}

fn the_verge(data: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut opt = ParsingOptions::default();
    opt.allow_dtd = true;
    let doc = roxmltree::Document::parse_with_options(data, opt)?;
    let class = "duet--article--article-body-component-container";

    // Find root element for article container
    let ele = doc.descendants().find(|x| match x.attribute("class") {
        Some(class_name) if class_name.contains(class) => true,
        _ => false,
    });

    // Extract text from root element
    match ele {
        Some(node) => {
            let node = node.first_child().unwrap();
            let str_vec: Vec<&str> = node
                .descendants()
                .filter(|x| x.tag_name().name() == "")
                .map(|x| x.text().unwrap_or(""))
                .collect();
            let contents = str_vec.join("");
            Ok(contents)
        }
        None => Err("Scraping failed!".to_string().into()),
    }
}

fn wired(doc: &str) -> Result<String, Box<dyn std::error::Error>> {
    let doc = Html::parse_document(doc);
    let class = ".body__inner-container";
    let find = Selector::parse(class).unwrap();

    let mut builder = String::new();
    for ele in doc.select(&find) {
        let str_vec: Vec<&str> = ele.text().collect();
        builder.push_str(&str_vec.join(""));
    }

    Ok(builder)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Post;

    #[ignore = "online"]
    #[tokio::test]
    async fn test_verge_scraper() {
        let verge = Sites::TheVerge;
        let post = Post::new_link("https://www.theverge.com/2024/2/11/24069251/waymo-driverless-taxi-fire-vandalized-video-san-francisco-china-town".to_string());
        // let post = Post::new_link("test-files/scrape-example-verge.html".to_string());
        let res = verge.scrape(post).await;
        match res {
            Ok(val) => {
                println!("{val}");
                assert!(val.contains(
                    "A person jumped on the hood of a Waymo driverless taxi and smashed its windshield in San Francisco’s \
                    Chinatown last night around 9PM PT, generating applause before a crowd formed around the car and covered \
                    it in spray paint, breaking its windows, and ultimately set it on fire. The fire department arrived minutes \
                    later, according to a report in The Autopian, but by then flames had already fully engulfed the car."
                )
            )
            }
            Err(e) => {
                println!("{:#?} => {}", e, e.to_string());
                assert!(false)
            }
        }
    }

    #[ignore = "online"]
    #[tokio::test]
    async fn test_wired_scraper() {
        // let post = Post::new_link("test-files/scrape-example-wired.html".to_string());
        let post =
            Post::new_link("https://www.wired.com/story/developers-revolt-apple-dma/".to_string());
        let res = Sites::Wired.scrape(post).await;

        match res {
            Ok(val) => {
                assert!(val.contains(
                    "A battle for control is taking place inside iPhones across Europe. \
                    While Apple introduced new rules that ostensibly loosen its control over \
                    the App Store, local developers are seething at the new system, which they \
                    say entrenches the power Apple already wields over their businesses. \
                    They’re now breaking into a rare open revolt, mounting pressure on lawmakers to step in."
                ));
            }
            Err(e) => {
                println!("{:#?}, {}", e, e.to_string());
                assert!(false);
            }
        }
    }
}
