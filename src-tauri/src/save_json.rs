#![allow(unused)]
#![allow(clippy::all)]

use crate::database::*;
use crate::models::Document;
use crate::models::*;
use crate::schema::document;
use crate::schema::document::dsl;
use crate::schema::Response;

use chrono::Datelike;
use chrono::{DateTime, Local, TimeZone};

use crate::diesel::sqlite::Sqlite;
use diesel::{insert_into, prelude::*, sql_query, debug_query};

use serde_json::json;
use tracing::{error, info};
use tracing_subscriber;
use uuid::Uuid;
use std::fs;

/// # save_json_by_doc
/// save file as json from document
pub async fn save_json_by_doc(
    i_document: &Document
) {
    info!(?i_document.id, "save_json_by_doc");
    
    let home_dir = home::home_dir().unwrap_or("".into());

    let json_file_to = format!(
        "{}/{}/{}/{}{}.json",
        home_dir.to_str().unwrap_or("").to_string(),
        MAIN_PATH,
        FILE_PATH,
        i_document.sub_path.clone().unwrap_or("".to_string()),
        i_document.id.clone()
    );

    fs::write(json_file_to, json!(i_document).to_string()).unwrap();

}


/// # save_json_by_id
/// save file as json from document by id 
pub async fn save_json_by_id(
    app_data: tauri::State<'_, crate::AppData>,
    id: String,
) {
    info!(?id, "save_json");
    
    let mut conn = app_data.db.lock().await;

    let exec_query = dsl::document
        .filter(dsl::id.eq(id.clone()))
        .select(Document::as_select());
    info!("debug sql\n{}", debug_query::<Sqlite, _>(&exec_query));

    let my_document: Document = match exec_query.first::<Document>(&mut *conn) {
        Ok(record) => record,
        Err(err) => {
            error!(?err, "Error: ");

            return ;
        }
    };

    save_json_by_doc(&my_document).await;

}