use crate::dao::schema::post;
use crate::dao::entities::thread::Thread;

//Post

#[derive(Debug, Clone, Queryable, Identifiable, PartialEq, Associations)]
#[table_name="post"]
#[belongs_to(Thread, foreign_key = "pid")]
#[primary_key(pid)]
pub struct Post {
    pub pid: i32,
    pub auth: Option<String>,
    pub pdate: chrono::NaiveDateTime,
    pub subject: Option<String>,
    pub comment: Option<String>,
    pub sid: i32,
    pub tid: i32,
}

#[derive(Insertable, Debug, Clone)]
#[table_name="post"]
pub struct NewPost<'a> {
    pub auth: &'a str,
    pub subject: &'a str,
    pub comment: &'a str,
    pub sid: i32,
    pub tid: i32,
}




