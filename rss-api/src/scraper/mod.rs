// use async_trait::async_trait;
use axum::async_trait;
use roxmltree::{self, ParsingOptions};
use rss_api::rss_parser::from_url;
use rss_api::Post;

#[async_trait]
pub trait Scraper {
    async fn scrape(&self, p: Post) -> Result<String, Box<dyn std::error::Error>>;
}

enum Sites {
    TheVerge,
}

#[async_trait]
impl Scraper for Sites {
    async fn scrape(&self, post: Post) -> Result<String, Box<dyn std::error::Error>> {
        let data = from_url(&post.link).await?;

        let mut opt = ParsingOptions::default();
        opt.allow_dtd = true;

        match self {
            Self::TheVerge => {
                let res = roxmltree::Document::parse_with_options(&data, opt)?;
                let class = "duet--article--article-body-component-container";
                let ele = res.descendants().find(|x| {
                    let has = x.has_attribute("class");
                    if has {
                        let text = x.attribute("class");
                        match text {
                            Some(t) if t.contains(class) => true,
                            _ => false,
                        }
                    } else {
                        false
                    }
                });

                match ele {
                    Some(node) => {
                        let node = node.first_child().unwrap();
                        let mut contents = String::new();
                        collect_string(node, &mut contents);
                        // println!("{contents}");

                        // println!("{bruh}");
                        Err("yes".to_string().into())
                    }
                    // what does into() do? find out.
                    None => Err("Scraping failed!".to_string().into()),
                }
            }
        }
    }
}

fn collect_string(node: roxmltree::Node, str: &mut String) {
    let blacklist = vec!["div", ""];

    for desc in node.descendants() {
        let tag = desc.tag_name();
        if !blacklist.contains(&tag.name()) {
            let text = desc.text();
            if let Some(t) = text {
                println!("{}\n{t}\n", tag.name());
                str.push_str(&format!("{t} \n"));
            }
        }
    }
}

fn verge_scraper() {}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Post;

    #[tokio::test]
    async fn test_verge_scraper() {
        let verge = Sites::TheVerge;
        let post = Post::new_link("https://www.theverge.com/2024/2/11/24069251/waymo-driverless-taxi-fire-vandalized-video-san-francisco-china-town".to_string());
        let res = verge.scrape(post).await;
        match res {
            Ok(val) => assert!(true),
            Err(e) => {
                println!("{:#?} {}", e, e.to_string());
                assert!(false)
            }
        }
    }
}
