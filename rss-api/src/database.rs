use crate::{Post, URLObject};
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

    pub async fn subscribe(&self, uid: u32, publisher: String) -> Result<(), mysql::error::Error> {
        let mut conn = self.pool.get_conn().unwrap();
        let query = conn.prep("Select pid from publisher where url=:publisher")?;

        let res: Result<Option<u64>, mysql::error::Error> =
            conn.exec_first(query, params! {"publisher"=>&publisher});

        let id = match res {
            Ok(Some(id)) => id,
            Ok(None) => {
                let query = conn.prep("INSERT INTO publisher(url) VALUES (:publisher)")?;
                let res = conn.exec_drop(query, params! {"publisher" => publisher});
                match res {
                    Ok(()) => conn.last_insert_id(),
                    Err(e) => return Err(e),
                }
            }
            Err(e) => return Err(e),
        };

        // STEP 2: String: USE PID TO CREATE
        let query = conn.prep("INSERT IGNORE into user_pubs(uid, pid) VALUES (:uid, :pid)")?;
        conn.exec_drop(query, params! {"uid" => uid, "pid"=>id})
    }

    pub async fn get_subbed(&self, uid: u64) -> Result<Vec<URLObject>, mysql::Error> {
        let mut conn = self.pool.get_conn().unwrap();

        let query = conn.prep("SELECT url, user_pubs.pid from user_pubs INNER JOIN publisher on user_pubs.pid=publisher.pid where uid=:uid")?;

        conn.exec_map(query, params! {"uid" => uid}, |(url, pid)| URLObject {
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

        // let (query, params) = match id {
        //     Some(i) => {
        //         let q = conn.prep("SELECT * from post where id=:id")?;
        //         let p = params! { "id" => i };
        //         (q, p)
        //     }
        //     None => match url {
        //         Some(u) => {
        //             let q = conn.prep("SELECT * from post where url=:url")?;
        //             let p = params! {"url" => u};
        //             (q, p)
        //         }
        //         None => return Err("Invalid parameters.".to_string().into()),
        //     },
        // };
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

    pub async fn get_post_list(&self, uid: u64, offset: u64) -> Result<Vec<Post>, mysql::Error> {
        let mut conn = self.pool.get_conn().unwrap();

        let query = conn.prep(
            " \
                SELECT id, url, title, date_added, description, image, post.pid FROM post \
                INNER JOIN user_pubs ON post.pid=user_pubs.pid \
                WHERE uid=:uid \
                ORDER BY date_added DESC \
                LIMIT :offset, 10;
                ",
        )?;

        conn.exec_map(
            query,
            params! {"uid" => uid, "offset" => offset},
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
}

impl Clone for DatabaseConnection {
    fn clone(&self) -> Self {
        let rf = self.pool.clone();
        DatabaseConnection { pool: rf }
    }
}
