#![allow(unused)]
#![allow(clippy::all)]

use crate::database::*;
use crate::models::*;
use crate::schema;
use crate::schema::document::dsl;
use crate::schema::Response;

use diesel::debug_query;
use crate::diesel::sqlite::Sqlite;
use diesel::prelude::*;
use serde_json::json;
use tracing::{error, info, warn};
use tracing_subscriber;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct PdfQuary {
    id: String,
    filename: String,
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

        let database_name = format!("{}/{}", MAIN_PATH, DATABASE_NAME);
        let mut conn = establish_connection(&database_name);

        let exec_query = dsl::document
            .filter(dsl::id.eq(my_query.id))
            .select(DocumentFile::as_select());
        info!("debug sql\n{}", debug_query::<Sqlite, _>(&exec_query));

        let my_document = match exec_query.first::<DocumentFile>(&mut conn) {
            Ok(record) => record,
            Err(err) => {
                error!(?err, "Error: ");

                return Response {
                    dataname: data,
                    data: "[]".to_string(),
                    error: format!("{}", err),
                };
            }
        };
        info!(?my_document.id, "select document id" );
        info!(?my_document.sub_path, "select document subpath" );

        use home::home_dir;
        let home_dir = match home_dir() {
            Some(result) => result,
            None => {
                return Response {
                    dataname: data,
                    data: "[]".to_string(),
                    error: r#"no pdf found"#.to_string(),
                };
            }
        };

        let filename = my_document.file.unwrap_or("".to_string());
        if filename.is_empty() {
            return Response {
                dataname: data,
                data: "[]".to_string(),
                error: r#"no pdf found"#.to_string(),
            };
        }

        //Build PDF Filenames
        let pdf_file = format!(
            "{}/{}/{}/{}{}",
            home_dir.to_str().unwrap_or("").to_string(),
            MAIN_PATH,
            FILE_PATH,
            my_document.sub_path.unwrap_or("".to_string()),
            filename
        );
        info!(?pdf_file, "select document file");

        //open file by name
        let mut file = match std::fs::File::open(&pdf_file) {
            Ok(file) => file,
            Err(err) => {
                error!(?err, "Error: ");

                return Response {
                    dataname: data,
                    data: "[]".to_string(),
                    error: format!("{}", err),
                };
            }
        };
        info!(?filename, "open file by name ");

        //Read PDF as binary file
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
            //binary encode to base64
            use base64::{engine::general_purpose, Engine as _};
            let base64_data = general_purpose::STANDARD_NO_PAD.encode(list_of_chunks);

            return Response {
                dataname: data,
                data: json!(&base64_data).to_string(),
                error: "".to_string(),
            };
        }

        //an diesem Punkt angekommen liegt ein Fehler vor
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
