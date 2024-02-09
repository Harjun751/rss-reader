#[derive(Debug)]
struct Post{
    title: String,
    link: String,
    date: Option<String>, // CHECK THIS
    description: Option<String>,
    content: Option<String>,
    enclosure: Option<String>,
}

pub mod database{
    use mysql::{prelude::Queryable, Pool};

    pub struct DatabaseConnection{
        pool :Pool,
    }
    
    impl DatabaseConnection{
        pub fn new() -> Self {
            let url = "mysql://root:test@localhost:3306/RSS";
            let pool = Pool::new(url).unwrap();
            DatabaseConnection { pool }
        }
    
        pub async fn subscribe(&self, uid: u32, publisher: String){
            let mut conn = self.pool.get_conn().unwrap();
    
            // STEP 1: GET PID FROM PUBLISHER STRING
            let query = format!("SELECT pubid from publisher where url='{}'", publisher);
            let result:Result<Option<u32>, _> = conn.query_first(query);
            // error if query fails or returns None
            let pubid = result.unwrap().unwrap();
            
            // STEP 2: USE PID TO CREATE
            let query = format!("INSERT into user_pubs(uid, pubid) VALUES ({}, {});", uid, pubid);
            conn.query_drop(query).unwrap();
        }
    
        pub async fn get_subbed(&self, uid: u32) -> Vec<String> {
            let mut conn = self.pool.get_conn().unwrap();
            let query = format!(
                "select url from user_pubs inner join publisher on user_pubs.pubid = publisher.pubid where uid={};", uid
            );
    
            let result: Result<Vec<String>, _> = conn.query(query);
            result.unwrap()
        }
    }
    
    impl Clone for DatabaseConnection{
        fn clone(&self) -> Self {
            let rf = self.pool.clone();
            DatabaseConnection{ pool:rf }
        }
    }
}

pub mod rss_parser{
    use std::fs;
    use std::sync::{Mutex, Arc};
    use crate::Post;

    use roxmltree::Node;

    /// get_whole_feed expects a list of urls to get feed data from
    async fn get_whole_feed(urls: Vec<String>) -> Vec<Post>{
        let vec: Arc<Mutex<Vec<Post>>> = Arc::new(Mutex::new(Vec::new()));
        let mut handles = vec![];

        for url in urls{
            let vector = Arc::clone(&vec);
            let handle = tokio::spawn(async move {
                let data = from_url(&url).await;
                let data = match data{
                    Ok(val) => val,
                    // failed to get data from url, print error and return
                    Err(e) => {println!("DEBUG: Error {}",e.to_string()); return}
                };
                let res = parse_feed(&data).await;
                match res{
                    Ok(mut posts) => {
                        let mut vector = vector.lock().unwrap();
                        vector.append(&mut posts);

                    },
                    Err(msg) => println!("DEBUG: unable to parse {}. \n Error: {}", url, msg)
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            // we should handle this.
            handle.await;
        }

        // We don't need the Arc anymore as all the async tasks are done.
        // We call try_unwrap followed by unwrap() as we know it won't fail
        let data = Arc::try_unwrap(vec).unwrap();
        // We call into_inner to move the value out of the mutex as we don't need the mutex anymore.
        let data = data.into_inner();

        match data {
            Ok(data) => data,
            // return an empty vector if failed.
            Err(msg) => {println!("DEBUG: Poison error. {}", msg); vec![]}
        }
    }

    async fn from_url(url: &str) -> Result<String, reqwest::Error>{
        let val = reqwest::get(url)
            .await?
            .text()
            .await?;
        Ok(val)
    }

    /// parse_feed takes in a slice of data representing the xml of the feed
    /// it then checks if the file is a valid rss/atom feed, if not it just tries both.
    /// Returns a vector of posts in the feed, or an Error string
    async fn parse_feed(data: &str) -> Result<Vec<Post>, &str>{
        let doc = roxmltree::Document::parse(data);
        let doc = match doc {
            Ok(val) => val,
            Err(e) => return Err("Error parsing xml")
        };
        let ver = doc.descendants()
                .find(|n| n.has_tag_name("rss"))
                .map(|n| n.attribute("version"));
        
        match ver {
            Some(Some(i)) if i == "2.0" || i == "0.92" || i == "0.91" => parse_rss(doc),
            _ => {
                // try to find the feed element that signifies atom
                let feed = &doc.descendants().find(|x| x.has_tag_name("feed"));
                if let Some(_) = feed {
                    parse_atom(doc)
                } else {
                    // hail mary - run rss then parse atom cos we ball like that
                    let res = parse_rss(doc);
                    match res {
                        Ok(val) => Ok(val),
                        // we read in the data again
                        // we choose to do this instead of using a borrow because i'm too lazy to refactor parse_xxx to take in a borrow
                        // it's also more performant (FAKE)
                        Err(_) => parse_atom(roxmltree::Document::parse(data).unwrap())   
                    }
                }
            }
        }
    }

    /// parse_rss returns a vector of posts or an error string
    fn parse_rss(doc:roxmltree::Document) -> Result<Vec<Post>, &str>{
        let mut vec: Vec<Post> = Vec::new();
        let items = doc.descendants()
                .filter(|x| x.has_tag_name("item"));
        
        for item in items{
            let nodes: Vec<Node> = item.descendants().collect();

            // maybe a macro? ->-
            let title = nodes.iter()
                .find(|x| x.has_tag_name("title"))
                .map(|x| x.text());

            let title = match title {
                Some(Some(t)) => t.to_owned(),
                _ => return Err("Missing required field title, or it's in the incorrect format!")
            };


            let link = nodes.iter()
                .find(|x| x.has_tag_name("link"))
                .map(|x| x.text());

            let link = match link {
                Some(Some(t)) => t.to_owned(),
                _ => return Err("Missing required field link, or it's in the incorrect format!")
            };
            
            let date = nodes.iter()
                .find(|x| x.has_tag_name("pubDate"))
                .map(|x| x.text());
            
            let date: Option<String> = match date {
                Some(Some(d)) => Some(d.to_owned()),
                _ => None
            };

            let description = nodes.iter()
                .find(|x| x.has_tag_name("description"))
                .map(|x| x.text());

            let description = match description {
                Some(Some(t)) => t.to_owned(),
                _ => return Err("Missing required field description, or it's in the incorrect format!")
            };
            
            let content = nodes.iter()
                .find(|x| x.has_tag_name("encoded"))
                .map(|x| x.text());

            let content = match content{
                Some(Some(c)) => Some(c.to_owned()),
                _ => {
                    // attempt to get content by using content tag
                    let content = nodes.iter()
                            .find(|x| x.has_tag_name("content"))
                            .map(|x| x.text());
                    match content{
                        Some(Some(c)) => Some(c.to_owned()),
                        _ => None
                    }
                }
            };
            

            let enclosure = nodes.iter()
                .find(|x| x.has_tag_name("enclosure"));

            let enclosure: Option<String> = match enclosure {
                Some(d) => d.attribute("url").map(|x| x.to_owned()),
                None => None
            };

            let post = Post { title , link, date, description: Some(description), content, enclosure };
            vec.push(post);
        }
        Ok(vec)
    }

    /// parse atom returns a vector of posts or an error string.
    fn parse_atom(doc: roxmltree::Document) -> Result<Vec<Post>, &str>{
        let mut vec: Vec<Post> = Vec::new();

        let items = doc.descendants()
                .filter(|x| x.has_tag_name("entry"));
        
        for item in items{
            let nodes: Vec<Node> = item.descendants().collect();

            // maybe a macro? ->-
            let title = nodes.iter()
                .find(|x| x.has_tag_name("title"))
                .map(|x| x.text());

            let title = match title {
                Some(Some(t)) => t.to_owned(),
                _ => return Err("Missing required field title, or it's in the incorrect format!")
            };


            let link = nodes.iter()
                .find(|x| x.has_tag_name("id"))
                .map(|x| x.text());

            let link = match link {
                Some(Some(t)) => t.to_owned(),
                _ => return Err("Missing required field link, or it's in the incorrect format!")
            };
            
            let date = nodes.iter()
                .find(|x| x.has_tag_name("published"))
                .map(|x| x.text());
            
            let date: Option<String> = match date {
                Some(Some(d)) => Some(d.to_owned()),
                _ => None
            };

            let description = nodes.iter()
                .find(|x| x.has_tag_name("summary"))
                .map(|x| x.text());

            let description = match description {
                Some(Some(t)) => Some(t.to_owned()),
                _ => None
            };
            
            let content = nodes.iter()
                .find(|x| x.has_tag_name("content"))
                .map(|x| x.text());

            let content = match content{
                Some(Some(c)) => Some(c.to_owned()),
                _ => None
            };
            

            let post = Post { title , link, date, description, content, enclosure: None };
            vec.push(post);
        }


        Ok(vec)
    }

    fn from_fs(path: &str) -> String{
        String::from_utf8_lossy(&fs::read(path).unwrap()).parse().unwrap()
    }

    #[cfg(test)]
    mod tests{
        use super::*;

        #[tokio::test]
        async fn test_rss_2_0(){
            let data = from_fs("rss-20.xml");
            let res = parse_feed(&data).await;
            assert_eq!(res.is_ok(), true);
            match res{
                Ok(vec) => {
                    let first = &vec[0];
                    assert_eq!(first.title, "The Best Dumb Stuff to Buy With Your Tax Refund Money");
                }
                Err(error) => {
                    println!("{error}");
                    assert_eq!(res.is_ok(), true);
                }
            }
        }
            

        #[tokio::test]
        async fn test_rss_0_91(){
            let data = from_fs("rss-91.xml");
            let res = parse_feed(&data).await;
            match res{
                Ok(vec) => {
                    let first = &vec[0];
                    assert_eq!(first.title, "Giving the world a pluggable Gnutella");
                    assert_eq!(first.description.as_ref().unwrap(), "WorldOS is a framework on which to build programs that work like Freenet or Gnutella -allowing distributed applications using peer-to-peer routing.");
                }
                Err(error) => {
                    println!("{error}");
                    assert_eq!(res.is_ok(), true);
                }
            }
        }

        #[tokio::test]
        async fn test_rss_0_92(){
            let data = from_fs("rss-92.xml");
            let res = parse_feed(&data).await;
            match res{
                Ok(vec) => {
                    let first = &vec[0];
                    assert_eq!(first.title, "Cats and Dogs Form Unlikely Friendship");
                    assert_eq!(first.description.as_ref().unwrap(), "In a heartwarming turn of events, a cat and a dog were spotted playing together in the park, proving that friendships can transcend species.");
                }
                Err(error) => {
                    println!("{error}");
                    assert_eq!(res.is_ok(), true);
                }
            }
        }

        #[tokio::test]
        async fn test_atom(){
            let data = from_fs("atom.xml");
            let res = parse_feed(&data).await;
            match res{
                Ok(vec) => {
                    let first = &vec[0];
                    assert_eq!(first.title, "Google’s use of student data could effectively ban Chromebooks from Denmark schools");
                    assert_eq!(first.link, "https://www.theverge.com/2024/2/7/24065332/denmark-google-student-data-collection-privacy");
                }
                Err(error) => {
                    println!("{error}");
                    assert_eq!(res.is_ok(), true);
                }
            }
        }

        #[tokio::test]
        async fn test_get_url_works(){
            let text = from_url("https://www.theverge.com/rss/index.xml").await.unwrap();
            println!("{text}");
            assert_eq!(text.contains("Google’s use of student data could effectively ban Chromebooks from Denmark schools"), true)
        }
    }
}