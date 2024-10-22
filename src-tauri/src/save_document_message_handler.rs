#![allow(unused)]
#![allow(clippy::all)]

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

pub async fn save_document_message_handler(
    //window: tauri::Window,
    app_data: tauri::State<'_, crate::AppData>,
    path: String,
    query: String,
    data: String,
) -> Response {
    info!("start save_document");

    let mut my_document_new: DocumentSmall = match serde_json::from_str(&data) {
        Ok(data) => data,
        Err(err) => {
            error!(?err, "Error: ");
            return Response {
                dataname: path,
                data: "[]".to_string(),
                error: err.to_string(),
            };
        }
    };

    let mut conn = app_data.db.lock().await;

    let exec_query = dsl::document
        .filter(dsl::id.eq(my_document_new.id.clone()))
        .select(DocumentSmall::as_select());
    info!("debug sql\n{}", debug_query::<Sqlite, _>(&exec_query));

    let my_document_old = match exec_query.first::<DocumentSmall>(&mut *conn) {
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

    info!(
        "deleted at old {:?} new {:?}",
        my_document_old.deleted_at.clone(),
        my_document_new.deleted_at.clone()
    );

    if json!(&my_document_old) == json!(&my_document_new) {
        //old and new are the same
        info!("old and new are the same");

        return Response {
            dataname: path,
            data: data,
            error: "".to_string(),
        };
    };

    let new_document_id = my_document_new.id.clone();

    let exec_update = diesel::update(dsl::document.filter(dsl::id.eq(my_document_new.id.clone())))
        .set((
            &my_document_new,                             //update AsChangeset
            dsl::updated_at.eq(Local::now().to_string()), //update datetime
        ));
    info!("debug sql\n{}", debug_query::<Sqlite, _>(&exec_update));

    match exec_update.execute(&mut *conn) {
        Ok(_) => {
            drop(conn);

            save_json_by_id(app_data, new_document_id).await;

            Response {
                dataname: path,
                data: json!(&my_document_new).to_string(),
                error: "".to_string(),
            }
        }
        Err(err) => {
            error!(?err, "Error: ");

            Response {
                dataname: path,
                data: "[]".to_string(),
                error: err.to_string(),
            }
        }
    }
}
