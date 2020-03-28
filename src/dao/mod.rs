use actix::*;
use diesel::r2d2::{ Pool, ConnectionManager };
use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel::result::Error;
use dotenv::dotenv;
use std::env;
use crate::dao::schema::*;

pub mod entities;
pub mod schema;

use entities::post::{Post,NewPost};
use entities::thread::{Thread,NewThread};


pub type DbPool = Pool<ConnectionManager<PgConnection>>;

/// Run query using Diesel to insert a new database row and return the result.
pub fn find_posts_by_tid(
    conn: &PgConnection,
    mytid: i32,
) -> Result<Option<Vec<Post>>, diesel::result::Error> {

    use schema::post::dsl::*;

    let posts = post
        .filter(tid.eq(mytid))
        .load::<Post>(conn)
        .optional()?;

    Ok(posts)
}

pub fn find_thread_by_tid(
    conn: &PgConnection,
    mytid: i32,
) -> Result<Option<Thread>, diesel::result::Error> {

    use schema::thread::dsl::*;

    let threads = thread
        .filter(tid.eq(mytid))
        .first::<Thread>(conn)
        .optional()?;

    Ok(threads)
}

// pub fn get_all_posts_by_thread(
//     conn: &PgConnection,
//     thread: Thread,
// ) -> Result<Vec<Post>,Error> {

//     Post::belonging_to(&thread).load::<Post>(conn)
// }

pub fn get_all_threads(
    conn: &PgConnection,
) -> Result<Vec<Thread>,Error> {

    use schema::thread::dsl::*;

    thread.load::<Thread>(conn)

}

pub fn get_thread_count(
    conn: &PgConnection,
) -> Result<i64,Error> {

    use schema::thread::dsl::*;
    use diesel::dsl::count;
    thread.select(count(tid)).first(conn)

}

pub fn get_all_tid_desc_limit(
    conn: &PgConnection,
    m: i64,
    n: i64,
) -> Result<Vec<i32>,Error> {

    use schema::thread::dsl::*;

    thread.select(tid)
    .order(tid.desc())
    .limit(m)
    .offset(n)
    .load(conn)

}

pub fn establish_manager() -> ConnectionManager::<PgConnection> {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    
    ConnectionManager::<PgConnection>::new(database_url)
    
}


pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
    
}

pub fn create_thread<'a>(
    conn: &PgConnection,
    tid: i32 
) -> Result<Thread, Error> {

    let new_thread = NewThread {
        first_floor: tid,
    };

    diesel::insert_into(thread::table)
        .values(&new_thread)
        .get_result(conn)
}

pub fn set_first_floor<'a>(
    conn: &PgConnection,
    thread_id: i32,
    ff: i32 
) -> QueryResult<usize> {

    use schema::thread::dsl::*;
    

    diesel::update(thread)
        .filter(tid.eq(thread_id))
        .set(first_floor.eq(ff))
        .execute(conn)
}

pub fn create_post<'a>(
    conn: &PgConnection, 
    auth: &'a str, 
    subject: &'a str, 
    comment: &'a str, 
    sid: i32, 
    tid: i32
) -> Result<Post, Error> {

    let new_post = NewPost {
        auth: auth,
        subject: subject,
        comment: comment,
        sid: sid,
        tid: tid,
    };
    diesel::insert_into(post::table)
        .values(&new_post)
        .get_result(conn)
}

