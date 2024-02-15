use crate::{rss_parser::validate_feed, Channel, Post, URLObject};
use chrono::NaiveDateTime;
use mysql::{params, prelude::Queryable, Pool};
use std::error::Error;

pub struct DatabaseConnection {
    pool: Pool,
}

impl DatabaseConnection {
    pub fn new() -> Self {
        let url = "mysql://root:test@localhost:3306/rss";
        let pool = Pool::new(url).unwrap();
        DatabaseConnection { pool }
    }

    pub async fn subscribe(&self, cid: u32, url: String) -> Result<(), Box<dyn Error>> {
        let mut conn = self.pool.get_conn().unwrap();
        let query = conn.prep("Select pid from publisher where url=:url")?;

        let res: Result<Option<u64>, mysql::error::Error> =
            conn.exec_first(query, params! {"url"=>&url});

        let id = match res {
            Ok(Some(id)) => id,
            Ok(None) => {
                let name = validate_feed(&url).await?;
                let query = conn.prep("INSERT INTO publisher(url, name) VALUES (:url, :name)")?;
                let res = conn.exec_drop(query, params! {"url" => &url, "name" => name});
                match res {
                    Ok(()) => conn.last_insert_id(),
                    Err(e) => return Err(e.into()),
                }
            }
            Err(e) => return Err(e.into()),
        };

        // STEP 2: String: USE PID TO CREATE
        let query = conn.prep("INSERT IGNORE into subscription(cid, pid) VALUES (:cid, :pid)")?;
        let res = conn.exec_drop(query, params! {"cid" => cid, "pid"=>id});
        if let Err(e) = res {
            Err(e.into())
        } else {
            Ok(())
        }
    }

    pub async fn get_subbed(&self, cid: u64) -> Result<Vec<URLObject>, mysql::Error> {
        let mut conn = self.pool.get_conn().unwrap();

        let query = conn.prep("SELECT url, subscription.pid from subscription INNER JOIN publisher on subscription.pid=publisher.pid where cid=:cid")?;

        conn.exec_map(query, params! {"cid" => cid}, |(url, pid)| URLObject {
            url,
            pid,
        })
    }

    pub async fn insert_posts(&self, posts: &Vec<crate::Post>) -> Result<(), mysql::Error> {
        let mut conn = self.pool.get_conn().unwrap();

        conn.exec_batch(
            r"INSERT IGNORE into post (url, title, content, date_added, description, image, pid) 
                VALUES (:url, :title, :content, :date_added, :description, :image, :pid)",
            posts.iter().map(|p| {
                params! {
                    "url" => &p.link,
                    "title" => &p.title,
                    "content" => &p.content,
                    "date_added" => {
                        match &p.date{
                            // YYYY-MM-DD hh:mm:ss
                            Some(d) => {
                                format!("{}", d.format("%Y-%m-%d %H:%M:%S"))
                            },
                            // TODO: ERROR HANDLING HERE LOL. Also check how null values are handled. Are they placed in front?
                            None => "1992-01-01".to_string()
                        }
                    },
                    "description" => &p.description,
                    "image" => &p.enclosure,
                    "pid" => &p.pid,
                }
            }),
        )?;
        Ok(())
    }

    pub async fn get_post(
        &self,
        id: Option<u64>,
        url: Option<String>,
    ) -> Result<Post, Box<dyn Error>> {
        let mut conn = self.pool.get_conn().unwrap();

        let (query, params) = match id {
            Some(i) => {
                let q = conn.prep("SELECT * from post where id=:id")?;
                let p = params! { "id" => i };
                (q, p)
            }
            None => match url {
                Some(u) => {
                    let q = conn.prep("SELECT * from post where url=:url")?;
                    let p = params! {"url" => u};
                    (q, p)
                }
                None => return Err("Invalid parameters.".to_string().into()),
            },
        };

        let post = conn.exec_map(
            query,
            params,
            |(id, url, title, content, date_added, description, image, pid): (
                u64,
                String,
                String,
                Option<String>,
                Option<NaiveDateTime>,
                Option<String>,
                Option<String>,
                u64,
            )| {
                let date = match date_added {
                    Some(dt) => Some(dt.and_utc()),
                    None => None,
                };
                Post {
                    id: Some(id),
                    link: url,
                    title: title,
                    content: content,
                    date: date,
                    description: description,
                    enclosure: image,
                    pid: pid,
                }
            },
        );

        // fails here
        match post {
            Ok(val) => {
                let new_val = val.into_iter().next();
                if let Some(post) = new_val {
                    Ok(post)
                } else {
                    Err("No post found!".to_string().into())
                }
            }
            Err(e) => Err(Box::new(e)),
        }
    }

    pub async fn get_post_list(&self, cid: u64, offset: u64) -> Result<Vec<Post>, mysql::Error> {
        let mut conn = self.pool.get_conn().unwrap();

        let query = conn.prep(
            " \
                SELECT id, url, title, date_added, description, image, post.pid FROM post \
                INNER JOIN subscription ON post.pid=subscription.pid \
                WHERE cid=:cid \
                ORDER BY date_added DESC \
                LIMIT :offset, 10;
                ",
        )?;

        conn.exec_map(
            query,
            params! {"cid" => cid, "offset" => offset},
            |(id, url, title, date_added, description, image, pid): (
                u64,
                String,
                String,
                Option<NaiveDateTime>,
                Option<String>,
                Option<String>,
                u64,
            )| {
                let date = match date_added {
                    Some(dt) => Some(dt.and_utc()),
                    None => None,
                };

                Post {
                    id: Some(id),
                    link: url,
                    title: title,
                    content: None,
                    date: date,
                    description: description,
                    enclosure: image,
                    pid: pid,
                }
            },
        )
    }

    pub async fn get_channels_for_user(&self, uid: u64) -> Result<Vec<Channel>, Box<dyn Error>> {
        let mut conn = self.pool.get_conn()?;

        let query = conn.prep("SELECT cid, name FROM channel where uid=:uid")?;

        let res = conn.exec_map(
            query,
            params! {"uid" => uid},
            |(cid, name): (u64, String)| Channel { cid, name },
        );

        match res {
            Ok(val) => Ok(val),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn insert_channel_for_user(
        &self,
        uid: u64,
        name: String,
    ) -> Result<(), mysql::error::Error> {
        let mut conn = self.pool.get_conn()?;

        let query = conn.prep("INSERT INTO channel (uid, name) VALUES (:uid, :name)")?;

        conn.exec_drop(query, params! {"uid" => uid, "name" => name})
    }
}

impl Clone for DatabaseConnection {
    fn clone(&self) -> Self {
        let rf = self.pool.clone();
        DatabaseConnection { pool: rf }
    }
}
