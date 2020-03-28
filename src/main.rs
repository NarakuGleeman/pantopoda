#[macro_use]
extern crate diesel;
extern crate dotenv;
#[macro_use]
extern crate actix_web;
#[macro_use] extern crate serde_derive;
//use actix_web::{Json};
use actix_multipart::Multipart;
use futures::{StreamExt, TryStreamExt};
use actix_web::{web, App, middleware, HttpResponse, HttpRequest, HttpServer, Error, Result};
use actix_files as fs;
use std::path::PathBuf;
use actix_web::http::header::{ContentDisposition, DispositionType};
use actix_web::http::StatusCode;
use askama::Template;
use std::str::FromStr;

use diesel::r2d2::{ Pool, ConnectionManager };
use diesel::prelude::*;
use diesel::pg::PgConnection;

use chrono::prelude::*;

use std::borrow::BorrowMut;

mod dao;
mod utils;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info,diesel=debug");    
    env_logger::init();
    dotenv::dotenv().ok();

    let manager = dao::establish_manager();

    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    let bind = "127.0.0.1:8088";

    println!("Starting server at: {}", &bind);
    
    HttpServer::new(move || {
        App::new()
        .data(pool.clone())
        .wrap(middleware::Logger::default())
        .service(web::resource("/").route(web::get().to(index)))
        .service(favicon)
        .service(web::resource("/static/{filename:.*}").route(web::get().to(static_file)))
        .service(web::resource("/stylesheets/{filename:.*}").route(web::get().to(static_file)))
        .service(web::resource("/res/{tid:\\d+}").route(web::get().to(do_res)))
        .service(web::resource("/post").route(web::post().to(do_post)))
    })
    .bind("127.0.0.1:8088")?
    .shutdown_timeout(60)
    .keep_alive(100)
    .run()
    .await

}

#[get("/favicon.ico")]
async fn favicon() -> Result<fs::NamedFile> {
    Ok(fs::NamedFile::open("static/favicon.ico")?)
}

use dao::entities::thread::Thread;
use dao::entities::post::Post;

struct ThreadView {
    tid: i32,
    posts: Vec<PostView>,
}

pub struct PostView {
    pub pid: i32,
    pub auth: String,
    pub pdate: chrono::NaiveDateTime,
    pub subject: String,
    pub comment: String,
    pub sid: i32,
    pub tid: i32,
}

#[derive(Template)]
#[template(path = "index.html")]
struct Index {
    year: String,
    views: Vec<ThreadView>,
}

async fn index(pool: web::Data<dao::DbPool>) -> Result<HttpResponse, Error> {

    let conn = pool.get().expect("couldn't get db connection from pool");

    let mut views: Vec<ThreadView> = vec![];

    let tid_vec = web:: block(move || dao::get_all_tid_desc_limit(
        &conn,
        20,
        0,
    ))
    .await
    .map_err(|e| {
        eprintln!("{}", e);
        return HttpResponse::InternalServerError().finish();
    })?;

    for tid in tid_vec {
        let conn = pool.get().expect("couldn't get db connection from pool");
        let _posts = web:: block(move || dao::find_posts_by_tid(&conn, tid))
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            return HttpResponse::InternalServerError().finish();
        })?;

        let posts = match _posts {
            Some(posts) => {
                posts
            },
            None => {
                Vec::new()
            },
        };

        let posts = posts.iter().map(move |p| {

            let auth = p.auth.as_ref().unwrap().to_string();
            let subject = p.subject.as_ref().unwrap().to_string();
            let comment = p.comment.as_ref().unwrap().to_string();

            PostView {
                pid: p.pid,
                auth: auth,
                pdate: p.pdate,
                subject: subject,
                comment: comment,
                sid: p.sid,
                tid: p.tid,
            }
        }).collect::<Vec<PostView>>();

        let view = ThreadView {
            tid: tid,
            posts: posts,        
        };
        views.push(view);
    }

    let tmp = Index {
        year: Local::now().year().to_string(),
        views: views,
    };
    let result = tmp.render().unwrap();
    Ok(HttpResponse::Ok().content_type("text/html").body(result))
}

#[derive(Template)]
#[template(path = "res.html")]
struct Res {
    year: String,
    tid: i32,
    posts: Vec<PostView>,
}

#[derive(Deserialize)]
struct Info {
    tid: i32,
}

async fn do_res(pool: web::Data<dao::DbPool>, info: web::Path<Info>) -> Result<HttpResponse, Error> {

    let tid = info.tid;
    //let tid = _tid.parse::<i32>().unwrap();
    //println!("tid: {}", tid);

    let conn = pool.get().expect("couldn't get db connection from pool");
    let _posts = web:: block(move || dao::find_posts_by_tid(&conn, tid))
    .await
    .map_err(|e| {
        eprintln!("{}", e);
        return HttpResponse::InternalServerError().finish();
    })?;

    let posts = match _posts {
        Some(posts) => {
            posts
        },
        None => {
            Vec::new()
        },
    };

    let posts = posts.iter().map(move |p| {

        let auth = p.auth.as_ref().unwrap().to_string();
        let subject = p.subject.as_ref().unwrap().to_string();
        let comment = p.comment.as_ref().unwrap().to_string();

        println!("auth: {}, subject: {}, comment: {}", auth, subject, comment);

        PostView {
            pid: p.pid,
            auth: auth,
            pdate: p.pdate,
            subject: subject,
            comment: comment,
            sid: p.sid,
            tid: p.tid,
        }
    }).collect::<Vec<PostView>>();  

    let tmp = Res {
        year: Local::now().year().to_string(),
        tid: tid,
        posts: posts,
    };
    let result = tmp.render().unwrap();
    Ok(HttpResponse::Ok().content_type("text/html").body(result))
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RawPost {
    pub comment: String,
    pub post: String,
    pub reply: String,
    pub subject: Option<String>,
    pub subsection: String,
}

async fn do_post(pool: web::Data<dao::DbPool>, mut payload: Multipart) -> Result<HttpResponse, Error> {

    println!("do post");

    use utils::processor::split_payload;

    let conn = pool.get().expect("couldn't get db connection from pool");

    let pl = split_payload(payload.borrow_mut()).await;
    //println!("bytes={:#?}", pl.0);

    let raw_data: Result<RawPost, serde_json::Error> = serde_json::from_slice(&pl.0);

    if let Ok(raw_post) = raw_data {
        // println!("converter_struct={:#?}", raw_post);
        // println!("tmpfiles={:#?}", pl.1);

        if raw_post.post == "Send" {
            
            let thread = web:: block(move || dao::create_thread(
                &conn,
                -1
            ))
            .await
            .map_err(|e| {
                eprintln!("{}", e);
                return HttpResponse::InternalServerError().finish();
            })?;

            println!("create a thread : {:?}", thread);

            let conn = pool.get().expect("couldn't get db connection from pool");            
            let tid = thread.tid;

            println!("create a thread : {}",tid);

            

            let subject = match &raw_post.subject {
                Some(s) => {
                    s.clone()
                },
                None => {
                    "".to_string()
                }
            };

            let post = web::block(move || dao::create_post(
                &conn, 
                "Nameless",
                &subject,
                &raw_post.comment,
                FromStr::from_str(&raw_post.subsection).unwrap(),
                thread.tid
            ))
            .await
            .map_err(|e| {
                eprintln!("{}", e);
                return HttpResponse::InternalServerError().finish();
            })?;

            let pid = post.pid;

            println!("create a thread : {}",pid);

            let conn = pool.get().expect("couldn't get db connection from pool");            

            web::block(move || dao::set_first_floor(
                &conn,
                tid,
                pid
            ))
            .await
            .map_err(|e| {
                eprintln!("{}", e);
                return HttpResponse::InternalServerError().finish();
            })?;

        } else if raw_post.post == "Reply" {

            // println!("mode : {:?}", raw_post.post);

            let conn = pool.get().expect("couldn't get db connection from pool");     
            
            let sid: i32 = FromStr::from_str(&raw_post.subsection).unwrap();
            let tid: i32 = FromStr::from_str(&raw_post.reply).unwrap();

            let subject;

            match &raw_post.subject {
                Some(s) => {
                    subject = s.to_string();
                },
                None => {
                    subject = "".to_string();
                }
            };

            web::block(move || dao::create_post(
                &conn, 
                "Nameless",
                &subject,
                &raw_post.comment,
                sid,
                tid,
            ))
            .await
            .map_err(|e| {
                eprintln!("{}", e);
                return HttpResponse::InternalServerError().finish();
            })?;

        }

        //create tmp file and upload s3 and remove tmp file
        // let upload_files: Vec<UplodFile> =
        //     upload_save_file(pl.1, s3_upload_key).await；
        // println!("upload_files={:#?}", upload_files);
        Ok(HttpResponse::build(StatusCode::SEE_OTHER)
            .header("Location", "/")
            .finish()
        )
    } else {
        Ok(HttpResponse::build(StatusCode::BAD_REQUEST).finish())
    }
}

async fn static_file(req: HttpRequest) -> Result<fs::NamedFile, Error> {

    let mut path = PathBuf::from("static");
    let filename: String = req.match_info().query("filename").parse().unwrap();
    path.push(filename);

    let file = fs::NamedFile::open(path)?;
    Ok(file
        .use_last_modified(true)
        .set_content_disposition(ContentDisposition {
            disposition: DispositionType::Attachment,
            parameters: vec![],
        }))
}
