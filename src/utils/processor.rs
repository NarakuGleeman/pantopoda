use actix_multipart::{Field, Multipart};
use actix_web::{web, Error};
use bytes::Bytes;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{Map as serdeMap, Value};
use std::convert::From;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UplodFile {
    pub filename: String,
}

impl From<Tmpfile> for UplodFile {
    fn from(tmp_file: Tmpfile) -> Self {
        UplodFile {
            filename: tmp_file.name,
        }
    }
}

/*
1. savefile
2. s3 upload -> upload_data
3. deletefile
*/

#[derive(Debug, Clone)]
pub struct Tmpfile {
    pub name: String,
    pub tmp_path: String,
}

impl Tmpfile {
    fn new(filename: &str) -> Tmpfile {
        Tmpfile {
            name: filename.to_string(),
            tmp_path: format!("./tmp/{}", filename),
        }
    }

}

pub async fn split_payload(payload: &mut Multipart, ) -> (bytes::Bytes, Vec<Tmpfile>) {
    use serde_json::json;
    use serde_json::Value::String;

    let mut tmp_files: Vec<Tmpfile> = Vec::new();

    let mut tmp_json = json!({});

    while let Some(item) = payload.next().await {

        let mut field: Field = item.expect(" split_payload err");
        let content_type = field.content_disposition().unwrap();
        let name = content_type.get_name().unwrap();

        if name != "file" {
            while let Some(chunk) = field.next().await {
                let bytes = chunk.expect(" split_payload err chunk");
                let tmp = std::str::from_utf8(bytes.as_ref()).unwrap();
                let value = String(tmp.to_string());
                tmp_json.as_object_mut().unwrap().insert(name.to_string(),value);
            }
        } else {

            match content_type.get_filename() {

                Some(filename) => {

                    let tmp_file = Tmpfile::new(filename);
                    // let tmp_path = tmp_file.tmp_path.clone();

                    // let mut f = web::block(move || std::fs::File::create(&tmp_path))
                    //     .await
                    //     .unwrap();

                    // while let Some(chunk) = field.next().await {
                    //     let data = chunk.unwrap();
                    //     f = web::block(move || f.write_all(&data).map(|_| f))
                    //         .await
                    //         .unwrap();
                    // }
                    
                    tmp_files.push(tmp_file.clone());

                }

                None => {
                    println!("file none");
                }
            }
        }
    }
    let post_json = tmp_json.to_string();
    println!("{}",post_json);
    (Bytes::from(post_json), tmp_files)
}