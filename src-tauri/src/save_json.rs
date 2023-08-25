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



#[tauri::command(async)]
pub async fn save_json(
    id: String,
) {
    info!(?id, "save_json");
    
    let database_name = format!("{}/{}", MAIN_PATH, DATABASE_NAME);
    let mut conn = establish_connection(&database_name);

    let exec_query = dsl::document
        .filter(dsl::id.eq(id.clone()))
        .select(Document::as_select());
    info!("debug sql\n{}", debug_query::<Sqlite, _>(&exec_query));

    let my_document: Document = match exec_query.first::<Document>(&mut conn) {
        Ok(record) => record,
        Err(err) => {
            error!(?err, "Error: ");

            return ;
        }
    };

    let home_dir = home::home_dir().unwrap_or("".into());

    let json_file_to = format!(
        "{}/{}/{}/{}{}.json",
        home_dir.to_str().unwrap_or("").to_string(),
        MAIN_PATH,
        FILE_PATH,
        my_document.sub_path.clone().unwrap_or("".to_string()),
        my_document.id.clone()
    );

    fs::write(json_file_to, json!(my_document).to_string()).unwrap();

}