use super::*;
use chrono::{NaiveDateTime, TimeZone};
use roxmltree::Node;
use std::error::Error;
use std::sync::{Arc, Mutex};

/// get_whole_feed expects a list of urls to get feed data from
pub async fn get_whole_feed(urls: Vec<URLObject>) -> Vec<Post> {
    let vec: Arc<Mutex<Vec<Post>>> = Arc::new(Mutex::new(Vec::new()));
    let mut handles = vec![];

    for url in urls {
        let vector = Arc::clone(&vec);
        let handle = tokio::spawn(async move {
            let data = from_url(&url.url).await;
            let data = match data {
                Ok(val) => val,
                Err(e) => {
                    // If this errors, means that the request to the url failed.
                    // We don't have to full-blown error here because other resources could still work.
                    // LOG THIS!
                    println!("DEBUG: Error {}", e.to_string());
                    return;
                }
            };
            let res = parse_feed(&data, &url).await;
            match res {
                Ok(mut posts) => {
                    let mut vector = vector.lock().unwrap();
                    vector.append(&mut posts);
                }
                // Again, we don't have to error here as other rss feeds may still parse well => may be ill-formed xml
                // LOG THIS!
                Err(msg) => println!("DEBUG: unable to parse {}. \n Error: {}", url.url, msg),
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        // we should handle this.
        let _ = handle.await;
    }

    // We don't need the Arc anymore as all the async tasks are done.
    // We call try_unwrap followed by unwrap() as we know it won't fail
    let data = Arc::try_unwrap(vec).unwrap();
    // We call into_inner to move the value out of the mutex as we don't need the mutex anymore.
    let data = data.into_inner();

    match data {
        Ok(data) => data,
        // Mutex got poisoned somehow.
        // LOG THIS!
        Err(msg) => {
            println!("DEBUG: Poison error. {}", msg);
            vec![]
        }
    }
}

pub async fn from_url(url: &str) -> Result<String, reqwest::Error> {
    let val = reqwest::get(url).await?.text().await?;
    Ok(val)
}

/// parse_feed takes in a slice of data representing the xml of the feed
/// it then checks if the file is a valid rss/atom feed, if not it just tries both.
/// Returns a vector of posts in the feed, or an Error
async fn parse_feed<'a>(data: &'a str, url: &URLObject) -> Result<Vec<Post>, Box<dyn Error>> {
    let doc = roxmltree::Document::parse(data);
    let doc = match doc {
        Ok(val) => val,
        Err(e) => return Err(Box::new(e)),
    };
    let ver = doc
        .descendants()
        .find(|n| n.has_tag_name("rss"))
        .map(|n| n.attribute("version"));

    match ver {
        Some(Some(i)) if i == "2.0" || i == "0.92" || i == "0.91" => parse_rss(doc, url),
        _ => {
            // try to find the feed element that signifies atom
            let feed = &doc.descendants().find(|x| x.has_tag_name("feed"));
            if let Some(_) = feed {
                parse_atom(doc, url)
            } else {
                // hail mary - run rss then parse atom cos we ball like that
                let res = parse_rss(doc, url);
                match res {
                    Ok(val) => Ok(val),
                    // we read in the data again
                    // we choose to do this instead of using a borrow because i'm too lazy to refactor parse_xxx to take in a borrow
                    // it's also more performant (FAKE)
                    Err(_) => match parse_atom(roxmltree::Document::parse(data).unwrap(), url) {
                        Ok(val) => Ok(val),
                        Err(_) => {
                            return Err("Invalid feed format! (Tried running hail mary)"
                                .to_string()
                                .into())
                        }
                    },
                }
            }
        }
    }
}

/// parse_rss returns a vector of posts or an error string
fn parse_rss<'a>(
    doc: roxmltree::Document,
    publisher: &URLObject,
) -> Result<Vec<Post>, Box<dyn Error>> {
    let mut vec: Vec<Post> = Vec::new();
    let items = doc.descendants().filter(|x| x.has_tag_name("item"));

    for item in items {
        let nodes: Vec<Node> = item.descendants().collect();

        // maybe a macro? ->-
        let title = nodes
            .iter()
            .find(|x| x.has_tag_name("title"))
            .map(|x| x.text());

        let title = match title {
            Some(Some(t)) => t.to_owned(),
            _ => {
                return Err(
                    "Missing required field title, or it's in the incorrect format!"
                        .to_string()
                        .into(),
                )
            }
        };

        let link = nodes
            .iter()
            .find(|x| x.has_tag_name("link"))
            .map(|x| x.text());

        let link = match link {
            Some(Some(t)) => t.to_owned(),
            _ => {
                return Err(
                    "Missing required field link, or it's in the incorrect format!"
                        .to_string()
                        .into(),
                )
            }
        };

        let date = nodes
            .iter()
            .find(|x| x.has_tag_name("pubDate"))
            .map(|x| x.text());

        let date = match date {
            Some(Some(text)) => match_date(text),
            _ => None,
        };

        let description = nodes
            .iter()
            .find(|x| x.has_tag_name("description"))
            .map(|x| x.text());

        let description = match description {
            Some(Some(t)) => t.to_owned(),
            _ => {
                return Err(
                    "Missing required field description, or it's in the incorrect format!"
                        .to_string()
                        .into(),
                )
            }
        };

        let content = nodes
            .iter()
            .find(|x| x.has_tag_name("encoded"))
            .map(|x| x.text());

        let content = match content {
            Some(Some(c)) => Some(c.to_owned()),
            _ => {
                // attempt to get content by using content tag
                let content = nodes
                    .iter()
                    .find(|x| x.has_tag_name("content"))
                    .map(|x| x.text());
                match content {
                    Some(Some(c)) => Some(c.to_owned()),
                    _ => None,
                }
            }
        };

        let enclosure = nodes.iter().find(|x| x.has_tag_name("enclosure"));

        let enclosure: Option<String> = match enclosure {
            Some(d) => d.attribute("url").map(|x| x.to_owned()),
            None => None,
        };

        let post = Post {
            id: None,
            title,
            link,
            date,
            description: Some(description),
            content,
            enclosure,
            pid: publisher.pid,
        };
        vec.push(post);
    }
    Ok(vec)
}

/// parse atom returns a vector of posts or an error string.
fn parse_atom<'a>(
    doc: roxmltree::Document,
    publisher: &URLObject,
) -> Result<Vec<Post>, Box<dyn Error>> {
    let mut vec: Vec<Post> = Vec::new();

    let items = doc.descendants().filter(|x| x.has_tag_name("entry"));

    for item in items {
        let nodes: Vec<Node> = item.descendants().collect();

        // maybe a macro? ->-
        let title = nodes
            .iter()
            .find(|x| x.has_tag_name("title"))
            .map(|x| x.text());

        let title = match title {
            Some(Some(t)) => t.to_owned(),
            _ => {
                return Err(
                    "Missing required field title, or it's in the incorrect format!"
                        .to_string()
                        .into(),
                )
            }
        };

        let link = nodes
            .iter()
            .find(|x| x.has_tag_name("id"))
            .map(|x| x.text());

        let link = match link {
            Some(Some(t)) => t.to_owned(),
            _ => {
                return Err(
                    "Missing required field link, or it's in the incorrect format!"
                        .to_string()
                        .into(),
                )
            }
        };

        let date = nodes
            .iter()
            .find(|x| x.has_tag_name("published"))
            .map(|x| x.text());

        let date: Option<DateTime<Utc>> = match date {
            Some(Some(d)) => {
                let res = DateTime::parse_from_rfc3339(d);
                match res {
                    Ok(dt) => Some(dt.to_utc()),
                    Err(_) => return Err("Invalid date format!".to_string().into()),
                }
            }
            _ => None,
        };

        let description = nodes
            .iter()
            .find(|x| x.has_tag_name("summary"))
            .map(|x| x.text());

        let description = match description {
            Some(Some(t)) => Some(t.to_owned()),
            _ => None,
        };

        let content = nodes
            .iter()
            .find(|x| x.has_tag_name("content"))
            .map(|x| x.text());

        let content = match content {
            Some(Some(c)) => Some(c.to_owned()),
            _ => None,
        };

        let post = Post {
            id: None,
            title,
            link,
            date,
            description,
            content,
            enclosure: None,
            pid: publisher.pid,
        };
        vec.push(post);
    }
    Ok(vec)
}

pub fn match_date(date: &str) -> Option<DateTime<Utc>> {
    let possible_dt_formats = vec![
        "%a, %d %b %Y %H:%M:%S".to_string(),
        "%a, %d %b %y %H:%M:%S".to_string(),
    ];

    // attempt to extract with timezone
    for fmt in &possible_dt_formats {
        let mut dtzfmt = String::from(fmt);
        // add the timezone format to the date
        dtzfmt.push_str(" %z");
        let val = DateTime::parse_from_str(date, &dtzfmt);
        match val {
            Ok(val) => return Some(val.to_utc()),
            Err(_) => (),
        };
    }

    for fmt in possible_dt_formats {
        let mut dtzfmt = String::from(fmt);
        dtzfmt.push_str(" %Z");
        let val = NaiveDateTime::parse_from_str(date, &dtzfmt);
        match val {
            Ok(val) => return Some(Utc.from_utc_datetime(&val)),
            Err(_) => (),
        };
    }
    None
}

// validate feed takes in a url pointing to an xml.
// it returns the feed name
pub async fn validate_feed(url: &str) -> Result<String, Box<dyn Error>> {
    let val = reqwest::get(url).await?.text().await?;
    let doc = roxmltree::Document::parse(&val)?;

    // look for version in rss
    let ver = doc
        .descendants()
        .find(|n| n.has_tag_name("rss"))
        .map(|n| n.attribute("version"));

    match ver {
        Some(Some(i)) if i == "2.0" || i == "0.92" || i == "0.91" => {
            // get the feed title and return
            let title = &doc
                .descendants()
                .find(|x| x.has_tag_name("title"))
                .map(|x| x.text());
            if let Some(Some(val)) = title {
                Ok(val.to_string())
            } else {
                Err("Malformed feed format!".to_string().into())
            }
        }
        _ => {
            // it may be an atom feed, let's check.
            let feed = &doc.descendants().find(|x| x.has_tag_name("feed"));
            match feed {
                Some(_) => {
                    // get the feed title & return
                    let title = &doc
                        .descendants()
                        .find(|x| x.has_tag_name("title"))
                        .map(|x| x.text());
                    if let Some(Some(val)) = title {
                        Ok(val.to_string())
                    } else {
                        Err("Malformed feed format!".to_string().into())
                    }
                }
                None => Err("Unrecognized feed format!".to_string().into()),
            }
        }
    }
}

#[cfg(test)]
mod rss_tests {
    use super::*;
    use std::fs;

    // NEED REFACTOR
    #[tokio::test]
    async fn test_rss_2_0() {
        let data: String = String::from_utf8_lossy(&fs::read("test-files/rss-20.xml").unwrap())
            .parse()
            .unwrap();
        let obj = URLObject {
            url: "rss-20.xml".to_string(),
            pid: 3,
        };
        let res = parse_feed(&data, &obj).await;
        match res {
            Ok(vec) => {
                let first = &vec[0];
                assert_eq!(
                    first.title,
                    "The Best Dumb Stuff to Buy With Your Tax Refund Money"
                );
            }
            Err(error) => {
                println!("{error}");
                assert!(false);
            }
        }
    }

    #[tokio::test]
    async fn test_rss_0_91() {
        let data: String = String::from_utf8_lossy(&fs::read("test-files/rss-91.xml").unwrap())
            .parse()
            .unwrap();
        let obj = URLObject {
            url: "rss-91.xml".to_string(),
            pid: 4,
        };
        let res = parse_feed(&data, &obj).await;
        match res {
            Ok(vec) => {
                let first = &vec[0];
                assert_eq!(first.title, "Giving the world a pluggable Gnutella");
                assert_eq!(first.description.as_ref().unwrap(), "WorldOS is a framework on which to build programs that work like Freenet or Gnutella -allowing distributed applications using peer-to-peer routing.");
            }
            Err(error) => {
                println!("{error}");
                assert!(false);
            }
        }
    }

    #[tokio::test]
    async fn test_rss_0_92() {
        let data: String = String::from_utf8_lossy(&fs::read("test-files/rss-92.xml").unwrap())
            .parse()
            .unwrap();
        let obj = URLObject {
            url: "rss-92.xml".to_string(),
            pid: 5,
        };
        let res = parse_feed(&data, &obj).await;
        match res {
            Ok(vec) => {
                let first = &vec[0];
                assert_eq!(first.title, "Cats and Dogs Form Unlikely Friendship");
                assert_eq!(first.description.as_ref().unwrap(), "In a heartwarming turn of events, a cat and a dog were spotted playing together in the park, proving that friendships can transcend species.");
            }
            Err(error) => {
                println!("{error}");
                assert!(false);
            }
        }
    }

    #[tokio::test]
    async fn test_atom() {
        let data: String = String::from_utf8_lossy(&fs::read("test-files/atom.xml").unwrap())
            .parse()
            .unwrap();
        let obj = URLObject {
            url: "atom.xml".to_string(),
            pid: 3,
        };
        let res = parse_feed(&data, &obj).await;
        match res {
            Ok(vec) => {
                let first = &vec[0];
                assert_eq!(first.title, "Google’s use of student data could effectively ban Chromebooks from Denmark schools");
                assert_eq!(first.link, "https://www.theverge.com/2024/2/7/24065332/denmark-google-student-data-collection-privacy");
            }
            Err(error) => {
                println!("{error}");
                assert!(false);
            }
        }
    }

    #[tokio::test]
    async fn test_get_url_works() {
        // a url pointing to the raw data of the atom.xml file hosted on github
        let text = from_url(
            "https://raw.githubusercontent.com/Harjun751/rss-reader/main/rss-api/atom.xml",
        )
        .await
        .unwrap();
        println!("{text}");
        assert_eq!(text.contains("Google’s use of student data could effectively ban Chromebooks from Denmark schools"), true)
    }

    #[tokio::test]
    async fn test_get_feed_works() {
        // this should be a urls pointing to .xml files online
        let obj = URLObject {
            url: "https://raw.githubusercontent.com/Harjun751/rss-reader/main/rss-api/atom.xml"
                .to_string(),
            pid: 6,
        };
        let urls: Vec<URLObject> = vec![obj];
        let posts = get_whole_feed(urls).await;
        let post = posts.iter().find(|x| x.title == "Google’s use of student data could effectively ban Chromebooks from Denmark schools");
        // should not error if all is good
        let _post = post.unwrap();
        assert!(posts.len() > 0);
    }

    #[tokio::test]
    async fn date_test() {
        let date = "Sat, 10 Feb 2024 23:18:59 GMT";
        let res = match_date(date);
        assert!(res.is_some())
    }
}
