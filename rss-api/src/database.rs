use crate::logger::DetailedError;
use crate::{rss_parser::validate_feed, Channel, Post, Subscription};
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

    pub async fn subscribe(&self, cid: u64, url: String) -> Result<(), DetailedError> {
        let mut conn = self.pool.get_conn().unwrap();
        let query = match conn.prep("Select pid from publisher where url=:url") {
            Ok(val) => val,
            Err(e) => return Err(DetailedError::new(Box::new(e))),
        };

        let res: Result<Option<u64>, mysql::error::Error> =
            conn.exec_first(query, params! {"url"=>&url});

        let id = match res {
            Ok(Some(id)) => id,
            Ok(None) => {
                let name = match validate_feed(&url).await {
                    Ok(val) => val,
                    Err(e) => return Err(DetailedError::new(e)),
                };
                let query = match conn.prep("INSERT INTO publisher(url, name) VALUES (:url, :name)")
                {
                    Ok(val) => val,
                    Err(e) => return Err(DetailedError::new(Box::new(e))),
                };
                let res = conn.exec_drop(query, params! {"url" => &url, "name" => name});
                match res {
                    Ok(()) => conn.last_insert_id(),
                    Err(e) => return Err(DetailedError::new(Box::new(e))),
                }
            }
            Err(e) => return Err(DetailedError::new(Box::new(e))),
        };

        // STEP 2: String: USE PID TO CREATE
        let query = match conn.prep("INSERT IGNORE into subscription(cid, pid) VALUES (:cid, :pid)")
        {
            Ok(val) => val,
            Err(e) => return Err(DetailedError::new(Box::new(e))),
        };
        let res = conn.exec_drop(query, params! {"cid" => cid, "pid"=>id});
        if let Err(e) = res {
            Err(DetailedError::new(Box::new(e)))
        } else {
            Ok(())
        }
    }

    pub async fn unsubscribe(&self, pid: u64, cid: u64) -> Result<(), DetailedError> {
        let mut conn = self.pool.get_conn().unwrap();
        let query = match conn.prep("Delete from subscription where pid=:pid and cid=:cid") {
            Ok(val) => val,
            Err(e) => return Err(DetailedError::new(Box::new(e))),
        };

        match conn.exec_drop(query, params! {"pid"=>pid, "cid" => cid}) {
            Ok(_) => Ok(()),
            Err(e) => return Err(DetailedError::new(Box::new(e))),
        }
    }

    pub async fn get_subbed(&self, cid: u64) -> Result<Vec<Subscription>, mysql::Error> {
        let mut conn = self.pool.get_conn().unwrap();

        let query = conn.prep("SELECT url, subscription.pid, name from subscription INNER JOIN publisher on subscription.pid=publisher.pid where cid=:cid")?;

        conn.exec_map(query, params! {"cid" => cid}, |(url, pid, name)| {
            Subscription {
                cid: cid,
                pid: pid,
                url: url,
                name: name,
            }
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
                    "date_added" => format!("{}", &p.date.format("%Y-%m-%d %H:%M:%S")),
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
                NaiveDateTime,
                String,
                Option<String>,
                u64,
            )| {
                Post {
                    id: id,
                    link: url,
                    title: title,
                    content: content,
                    date: date_added.and_utc(),
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
                NaiveDateTime,
                String,
                Option<String>,
                u64,
            )| {
                Post {
                    id: id,
                    link: url,
                    title: title,
                    content: None,
                    date: date_added.and_utc(),
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

    pub async fn delete_channel_for_user(
        &self,
        uid: u64,
        cid: u64,
    ) -> Result<(), mysql::error::Error> {
        let mut conn = self.pool.get_conn()?;

        let query = conn.prep("DELETE FROM channel WHERE uid=:uid and cid=:cid")?;

        conn.exec_drop(query, params! {"uid" => uid, "cid" => cid})
    }
}

impl Clone for DatabaseConnection {
    fn clone(&self) -> Self {
        let rf = self.pool.clone();
        DatabaseConnection { pool: rf }
    }
}
