#![allow(unused)]
#![allow(clippy::all)]

use std::io::{copy, stdout};

use std::fs::File;
use std::io::{self, Read, Write, BufReader, BufWriter};

use crate::database::*;
use crate::models::*;
use crate::save_json::*;
use crate::schema::document::dsl;
use crate::schema::Response;

use chrono::{DateTime, Local, TimeZone};

use crate::diesel::sqlite::Sqlite;
use diesel::debug_query;
use diesel::prelude::*;
use serde_json::json;
use tracing::{error, info};
use tracing_subscriber;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct SaveFile {
    id: String,
    file: String,
}

fn copy_file(source: &str, destination: &str) -> io::Result<()> {
    let mut reader = BufReader::new(File::open(source)?);
    let mut writer = BufWriter::new(File::create(destination)?);

    let mut buffer = [0; 8192]; // 8KB Puffer

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        writer.write_all(&buffer[..bytes_read])?;
    }

    Ok(())
}

pub async fn save_file_message_handler(
    window: tauri::Window,
    app_data: tauri::State<'_, crate::AppData>,
    path: String,
    query: String,
    data: String,
) -> Response {
    info!("start save_document");

    let my_doc_data: SaveFile = match serde_json::from_str(&data) {
        Ok(result) => result,
        Err(err) => {
            error!(?err, "Error: ");

            return Response {
                dataname: "info".to_string(),
                data: "[]".to_string(),
                error: format!("{}", err),
            };
        }
    };
    if my_doc_data.file.is_empty() {
        return Response {
            dataname: "info".to_string(),
            data: "no file selected".to_string(),
            error: "".to_string(),
        };
    }

    use tauri_plugin_dialog::DialogExt;

    let destination_path = window
        .dialog()
        .file()
        .add_filter("Filter", &["pdf"])
        .set_file_name(my_doc_data.file)
        .blocking_save_file();

    if destination_path.is_none() {
        return Response {
            dataname: "info".to_string(),
            data: json!("no file selected or operation canceled").to_string(),
            error: "".to_string(),
        };
    }

    let destination_path = destination_path.unwrap().to_string();
    info!(?destination_path, "file_path");

    let mut conn = app_data.db.lock().await;
    let main_data = app_data.main_data.lock().await;
    let mut sel_owner = '%'.to_string();
    if !main_data.all_owner {
        sel_owner = main_data.email.clone().to_lowercase();
    }

    let exec_query = dsl::document
        .filter(dsl::id.eq(my_doc_data.id).and(dsl::owner.like(sel_owner)))
        .select(DocumentFile::as_select());
    info!("debug sql\n{}", debug_query::<Sqlite, _>(&exec_query));

    let my_document = match exec_query.first::<DocumentFile>(&mut *conn) {
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

    let home_dir = home::home_dir().unwrap_or("".into());

    let filename = my_document.file.unwrap_or("".to_string());
    if filename.is_empty() {
        return Response {
            dataname: data,
            data: "[]".to_string(),
            error: r#"no pdf found"#.to_string(),
        };
    }

    //Build PDF Filenames
    let source_path = format!(
        "{}/{}/{}/{}{}",
        home_dir.to_str().unwrap_or("").to_string(),
        MAIN_PATH,
        FILE_PATH,
        my_document.sub_path.unwrap_or("".to_string()),
        filename
    );
    info!(?source_path, "select document file");
    
    match  copy_file(&source_path, &destination_path) {
        Ok(_) => {}
        Err(err) => {
            error!(?err, "Error: ");
            return Response {
                dataname: data,
                data: "[]".to_string(),
                error: format!("Error: {}", err),
            };
        }
    };

    return Response {
        dataname: "info".to_string(),
        data: json!("file saved").to_string(),
        error: "".to_string(),
    };
}
