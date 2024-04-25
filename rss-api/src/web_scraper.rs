use crate::Post;
use crate::{logger::DetailedError, rss_parser::from_url};
use lazy_static::lazy_static;
use scraper::{ElementRef, Html, Selector};
use std::error::Error;
use std::sync::Mutex;

pub struct CleanedHTML {
    html: String,
    pub raw: String,
}

impl ToString for CleanedHTML {
    fn to_string(&self) -> String {
        self.html.to_string()
    }
}

// TODO: remove
#[derive(Debug)]
struct Site {
    url: &'static str,
    root_element_selector: Selector,
}

// Define sites to scrape here
// Go in decreasing specificity
lazy_static! {
    static ref SITES: Vec<Site> = vec![
        Site {
            url: "theverge.com",
            root_element_selector: Selector::parse(
                ".duet--article--article-body-component-container"
            )
            .unwrap(),
        },
        Site {
            url: "www.wired.com/story",
            root_element_selector: Selector::parse(".body__inner-container").unwrap(),
        },
        Site {
            url: "www.wired.com/20",
            root_element_selector: Selector::parse("article.content").unwrap(),
        },
        Site {
            url: "arstechnica.com",
            root_element_selector: Selector::parse(".article-guts").unwrap(),
        },
        Site {
            url: "straitstimes.com",
            root_element_selector: Selector::parse(".field:not(.field--name-field-related-articles,.field--name-field-display-headline,.field--name-dynamic-twig-fieldnode-st-boilerplate,.field--name-dynamic-twig-fieldnode-social-icons-bottom,.field--name-body)").unwrap(),
        },
        Site {
            url: "rockpapershotgun.com",
            root_element_selector: Selector::parse(".article_body_content,.headline_image").unwrap(),
        }
    ];
}

pub async fn scrape(post: &mut Post) -> Result<(), Box<dyn Error>> {
    let data = from_url(&post.link).await?;
    let possible_site = SITES.iter().find(|x| post.link.contains(x.url));

    let new_body = match possible_site {
        Some(site) if site.url == "arstechnica.com" => {
            ars_technica(data, &site.root_element_selector)
                .await
                .to_string()
        }
        Some(site) => clean_html(&data, Some(&site.root_element_selector)).to_string(),
        None => clean_html(&data, None).to_string(),
    };

    post.set_content(new_body);
    Ok(())
}

pub fn clean_html(data: &str, selector: Option<&Selector>) -> CleanedHTML {
    let doc = Html::parse_document(data);
    let mut builder = String::new();
    let mut raw = String::new();

    let roots = match selector {
        Some(s) => {
            let desc = doc.select(&s);
            desc.collect()
        }
        None => vec![doc.root_element()],
    };

    for root in roots {
        for e in root.descendants() {
            let element = e.value().as_element();
            if let Some(ele) = element {
                match ele.name() {
                    "p" => {
                        let refs = ElementRef::wrap(e).unwrap();
                        let text = refs.text().collect::<Vec<&str>>().join("");
                        raw.push_str(&text);
                        let text = "<p>".to_string() + &text + "</p>";
                        builder.push_str(&text);
                    }
                    "img" => {
                        if let Some(src) = ele.attr("src") {
                            let text = format!("<img src=\"{}\"/>", src);
                            builder.push_str(&text);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    CleanedHTML { html: builder, raw }
}

async fn ars_technica(data: String, root_selector: &Selector) -> CleanedHTML {
    let curr_data = Mutex::new(data);
    let selector = Selector::parse("span.next").unwrap();
    let mut aggregated = CleanedHTML {
        raw: String::from(""),
        html: String::from(""),
    };
    loop {
        {
            let lock = curr_data.lock().unwrap();
            let cleaned = clean_html(&*lock, Some(root_selector));
            aggregated.raw.push_str(&cleaned.raw);
            aggregated.html.push_str(&cleaned.html);
        }

        // check if there is a next page
        let next_link = {
            let lock = curr_data.lock().unwrap();
            let parsed = Html::parse_document(&*lock);
            let link = parsed.select(&selector).next();
            match link {
                Some(next) => ElementRef::wrap(next.parent().unwrap())
                    .unwrap()
                    .attr("href")
                    .unwrap()
                    .to_owned(),
                None => return aggregated,
            }
        };

        let next_data = from_url(&next_link).await;
        match next_data {
            Ok(data) => {
                let mut lock = curr_data.lock().unwrap();
                *lock = data;
            }
            Err(e) => {
                DetailedError::new_descriptive(Box::new(e), "Failed scraping ars link.");
            }
        }
    }
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

    #[test]
    fn test_clean_html() {
        let html = r#" <figure> <img alt="Samsung Galaxy S24 Ultra" src="https://cdn.vox-cdn.com/thumbor/yfM4CVWVjzjMZlkhIWwWX01FMrc=/0x0:2000x1333/1310x873/cdn.vox-cdn.com/uploads/chorus_image/image/73226505/DSC06482.0.jpg" /> <figcaption><em>The stinky stylus in question.</em> | Photo by Allison Johnson / The Verge</figcaption> </figure> <p id="ihJ4FM">I smelled the S Pen, and the reports are true: it kind of stinks. </p> <p id="HdqSFq">The S Pen is one of the <a href="https://www.theverge.com/24053907/samsung-galaxy-s24-ultra-review-ai-screen-camera-battery">Samsung Galaxy S24 Ultra’s</a> signature features — it’s a stylus that lives in the phone. A report from Reddit user LatifYil <a href="https://www.reddit.com/r/samsung/comments/1bixq94/why_does_my_s_pen_smell_so_bad/">kicked off the S Pen aroma discussion</a> earlier this week, noting that the S Pen on their Samsung Galaxy S24 Ultra “absolutely reeks.” Dozens of commenters with S24 Ultras (and earlier stylus-toting Galaxy phones) responded in affirmation: their styli stank.</p> <p id="WclJas"><a href="https://www.sammobile.com/news/galaxy-s24-ultra-s-pen-smells-burnt-plastic/">As noted by <em>SamMobile</em></a>, a moderator on Samsung’s EU community forums <a href="https://eu.community.samsung.com/t5/galaxy-s24-series/spen-tip-smells-burnt/td-p/9309704">offered a reasonable explanation</a> for the smell:</p> <blockquote><p id="W7ic9L">This isn’t anything to be concerned about. While the S Pen is in its holster, it is close to the internal components of the phone, which will generate heat while...</p></blockquote> <p> <a href="https://www.theverge.com/2024/3/22/24108848/samsung-galaxy-s24-ultra-s-pen-stylus-smell">Continue reading&hellip;</a> </p> "#;
        let new = clean_html(html, None);
        println!("{}", new.to_string());
        assert!(false);
    }
}
