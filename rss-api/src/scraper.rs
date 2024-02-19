use crate::rss_parser::from_url;
use roxmltree::{self, ParsingOptions};
use rss_api::Post;
use scraper::{Html, Selector};
// use std::fs;
use std::error::Error;

pub async fn scrape(post: &mut Post) -> Result<(), Box<dyn Error>> {
    let data = from_url(&post.link).await?;

    // let fsd = fs::read(&post.link).unwrap();
    // let data = String::from_utf8_lossy(&fsd);

    let new_body = match &post.link {
        x if x.contains("theverge.com") => the_verge(&data),
        x if x.contains("www.wired.com") => wired(&data),

        _ => Err("Unknown pattern for url!".to_string().into()),
    }?;

    post.set_content(new_body);
    Ok(())
}

fn the_verge(data: &str) -> Result<String, Box<dyn Error>> {
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

fn wired(doc: &str) -> Result<String, Box<dyn Error>> {
    let doc = Html::parse_document(doc);
    let class = ".body__inner-container";
    let find = Selector::parse(class).unwrap();

    let mut builder = String::new();
    for ele in doc.select(&find) {
        let str_vec: Vec<&str> = ele.text().filter(|x| !x.contains("img")).collect();
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
        let mut post = Post::new_link("https://www.theverge.com/2024/2/11/24069251/waymo-driverless-taxi-fire-vandalized-video-san-francisco-china-town".to_string());
        // let mut post = Post::new_link("test-files/scrape-example-theverge.com.html".to_string());
        let res = scrape(&mut post).await;
        match res {
            Ok(_) => {
                assert!(post.get_content().contains(
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
        // let mut post = Post::new_link("test-files/scrape-example-www.wired.com.html".to_string());
        let mut post = Post::new_link(
            "https://www.wired.com/story/cryptography-algorithm-upgrade-security/".to_string(),
        );
        let res = scrape(&mut post).await;

        match res {
            Ok(_) => {
                assert!(post.get_content().contains(
                    "In our increasingly digital lives, security depends on cryptography. \
                    Send a private message or pay a bill online, and you’re relying on \
                    algorithms designed to keep your data secret. Naturally, some people \
                    want to uncover those secrets—so researchers work to test the strength \
                    of these systems to make sure they won’t crumble at the hands of a clever attacker."
                ));
                // ensure no leftover html tag remains
                assert!(!post.get_content().contains("<img"));
                println!("{}", post.get_content());
                assert!(false);
            }
            Err(e) => {
                println!("{:#?}, {}", e, e.to_string());
                assert!(false);
            }
        }
    }
}
