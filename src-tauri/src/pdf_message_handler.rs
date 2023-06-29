#![allow(unused)]
#![allow(clippy::all)]

use crate::database::*;
use crate::models::*;
use crate::schema;
use crate::schema::document::dsl;
use crate::schema::Response;

use diesel::prelude::*;
use serde_json::json;
use tracing::{info, warn, error};
use tracing_subscriber;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct PdfQuary {
    id: String,
    filename: String,
}

pub fn get_string(in_opt_string: Option<String>) -> String {
    match in_opt_string {
        None => "".to_string(),
        (data) => data.unwrap(),
    }
}

#[tauri::command(async)]
pub async fn pdf_message_handler(
    //window: tauri::Window,
    //database: tauri::State<'_, Database>,
    path: String,
    query: String,
    data: String,
) -> Response {
    info!("start pdf");

    let my_query = match serde_json::from_str(&query) {
        Ok(data) => data,
        _ => PdfQuary {
            id: "".to_string(),
            filename: "".to_string(),
        },
    };
    if !my_query.filename.is_empty() && !my_query.id.is_empty() {
        info!(?my_query.filename, "load pdf" );

        let mut conn = establish_connection();

        let my_document = match dsl::document
            .filter(dsl::id.eq(my_query.id))
            .select(DocumentFile::as_select())
            .first::<DocumentFile>(&mut conn)
        {
            Ok(record) => record,
            Err(err) => {
                error!(?err, "Error: ");
                let message = format!("{}", err);

                return Response {
                    dataname: data,
                    data: "[]".to_string(),
                    error: message.to_string(),
                };
            }
        };
        info!(?my_document.id, "select document id" );
        info!(?my_document.sub_path, "select document subpath" );

        use home::home_dir;
        let home_dir = home_dir().unwrap();
        let filename = get_string(my_document.file);
        if !filename.is_empty() {
            //PDF Filenamen zusammenbauen
            use std::fs::File;
            let mut pdf_file = home_dir.to_str().unwrap().to_string();
            pdf_file.push_str("/megarecords-files/data/");
            let file_sub_path = get_string(my_document.sub_path);
            pdf_file.push_str(&file_sub_path);
            pdf_file.push_str(&filename);
            info!(?pdf_file, "select document file");

            let mut file = match std::fs::File::open(&pdf_file) {
                Ok(file) => file,
                Err(err) => {
                    error!(?err, "Error: ");
                    let message = format!("{}", err);

                    return Response {
                        dataname: data,
                        data: "[]".to_string(),
                        error: message.to_string(),
                    };
                }
            };
            info!(?filename, "open file by name ");

            //PDF als Binary lesen
            use std::io::{self, Read, Seek, SeekFrom};
            let mut list_of_chunks = Vec::new();
            let chunk_size = 0x4000;

            loop {
                let mut chunk = Vec::with_capacity(chunk_size);
                let n = match file
                    .by_ref()
                    .take(chunk_size as u64)
                    .read_to_end(&mut chunk)
                {
                    Ok(data) => data,
                    Err(err) => {
                        info!(?err, "error file read");
                        break;
                    }
                };
                if n == 0 {
                    break;
                }
                for ele in chunk {
                    list_of_chunks.push(ele);
                }
                if n < chunk_size {
                    break;
                }
            }

            if list_of_chunks.len() != 0 {
                //binary in base64 codieren
                use base64::{engine::general_purpose, Engine as _};
                let base64_data = general_purpose::STANDARD_NO_PAD.encode(list_of_chunks);

                return Response {
                    dataname: data,
                    data: json!(&base64_data).to_string(),
                    error: "".to_string(),
                };
            }
        }

        Response {
            dataname: data,
            data: "[]".to_string(),
            error: r#"no pdf found"#.to_string(),
        }
    } else {
        //error als Response struktur
        Response {
            dataname: data,
            data: "[]".to_string(),
            error: r#"no pdf found"#.to_string(),
        }
    }
}
