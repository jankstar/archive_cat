#![allow(unused)]
#![allow(clippy::all)]

use crate::database::*;
use crate::do_status_message_handler::*;
use crate::models::Document;
use crate::models::*;
use crate::save_json::*;
use crate::schema::document;
use crate::schema::document::dsl;
use crate::schema::Response;

use chrono::{DateTime, Local, TimeZone, Datelike};

use crate::diesel::sqlite::Sqlite;
use diesel::{insert_into, prelude::*, sql_query};

use serde_json::json;
use tracing::{error, info};
use tracing_subscriber;
use uuid::Uuid;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct FileData {
    name: String,
    data_base64: String,
}

pub async fn upload_files_message_handler(
    window: tauri::Window,
    app_data: tauri::State<'_, crate::AppData>,
    path: String,
    query: String,
    data: String,
) -> Response {
    info!("start upload_files");

    let file_data: FileData = match serde_json::from_str(&data) {
        Ok(data) => data,
        _ => FileData {
            name: "".to_string(),
            data_base64: "".to_string(),
        },
    };

    if file_data.name.is_empty() || file_data.data_base64.is_empty() {
        return Response {
            dataname: path,
            data: "[]".to_string(),
            error: "no filenames found (1)".to_string(),
        };
    }

    let mut new_document = Document {
        id: Uuid::new_v4().to_string(),
        subject: file_data.name.clone(),
        status: "01_Leer".to_string(),
        date: Local::now().to_string(),
        sender_name: None,
        sender_addr: None,
        recipient_name: None,
        recipient_addr: None,
        from: None,
        to: None,
        body: None,
        document_type: Some("PDF".to_string()),
        metadata: None,
        category: None,
        amount: None,
        currency: None,
        template_name: None,
        doc_data: None,
        input_path: Some("01_upload".to_string()),
        langu: Some("DE".to_string()),
        num_pages: None,
        protocol: "".to_string(),
        sub_path: Some(format!("{}/", Local::now().year())),
        filename: Some(file_data.name.clone()),
        file_extension: None,
        file: None,
        base64: None,
        ocr_data: None,
        jpg_file: None,
        parent_document: None,
        created_at: Local::now().to_string(),
        updated_at: "".to_string(),
        deleted_at: None,
    };

    info!(
        "filename {} and sub_path {}",
        new_document.filename.clone().unwrap_or("".to_string()),
        new_document.sub_path.clone().unwrap_or("".to_string())
    );

    let mut extension_vec: Vec<&str> = file_data.name.split(".").collect();
    if extension_vec.len() == 0 {
        return Response {
            dataname: path,
            data: "[]".to_string(),
            error: "no filenames found (2)".to_string(),
        };
    };

    new_document.file_extension = Some(extension_vec[extension_vec.len() - 1].to_string());

    new_document.file = Some(format!(
        "{}.{}",
        new_document.id.clone(),
        new_document
            .file_extension
            .clone()
            .unwrap_or("".to_string())
    ));

    let home_dir = home::home_dir().unwrap_or("".into());

    //Build PDF Filenames
    let pdf_file_to = format!(
        "{}/{}/{}/{}{}",
        home_dir.to_str().unwrap_or("").to_string(),
        MAIN_PATH,
        FILE_PATH,
        new_document.sub_path.clone().unwrap_or("".to_string()),
        new_document.file.clone().unwrap_or("".to_string())
    );
    info!(?pdf_file_to, "new document file");

    use base64::{engine::general_purpose, Engine as _};
    ///the base64 data are from the javascript with 'pading' therefore select 'STANDARD' here!
    let data_vec = match general_purpose::STANDARD.decode(file_data.data_base64) {
        Ok(data) => data,
        Err(err) => {
            error!(?err, "base64 decode");
            Vec::new()
        }
    };

    if data_vec.len() == 0 {
        return Response {
            dataname: path,
            data: "[]".to_string(),
            error: "no data found (4)".to_string(),
        };
    };

    use std::fs;
    use std::io::Write; // bring trait into scope
    let mut file = match fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(pdf_file_to){
            Ok(i_file) => i_file,
            Err(err) => {
                error!("error open file {}", err);
                return Response {
                    dataname: path,
                    data: "[]".to_string(),
                    error: err.to_string(),
                };
            }
        };

    file.write_all(&data_vec);

    let new_document_id = new_document.id.clone();

    let mut conn = app_data.db.lock().await;

    match insert_into(document::dsl::document)
        .values(&new_document)
        .execute(&mut *conn)
    {
        Ok(_) => {
            drop(conn);

            save_json_by_doc(&new_document).await;

            // do_status_message_handler(
            //     window,
            //     app_data,
            //     "dostatus".to_string(),
            //     "".to_string(),
            //     new_document_id,
            // )
            // .await;



            Response {
                dataname: path,
                data: data.clone(),
                error: "".to_string(),
            }
        }
        Err(err) => {
            error!(?err);

            Response {
                dataname: path,
                data: "[]".to_string(),
                error: err.to_string(),
            }
        }
    }
}
