#![allow(unused)]
#![allow(clippy::all)]

use crate::database::*;
use crate::models::*;
use crate::schema;
use crate::schema::document::dsl;
use crate::schema::Response;

use diesel::debug_query;
use diesel::prelude::*;
use serde_json::json;
use tracing::info;
use tracing_subscriber;

#[tauri::command(async)]
pub async fn document_message_handler(
    //window: tauri::Window,
    //database: tauri::State<'_, Database>,
    path: String,
    query: String,
    //data: String,
) -> Response {
    info!("start document");
    //use self::schema::document::dsl::*;
    use url::Url;

    let mut my_query_url = "http://123".to_owned();
    my_query_url.push_str(query.as_ref());

    let parsed_url = match Url::parse(&my_query_url) {
        Ok(result) => result,
        Err(err) => {
            return Response {
                dataname: path,
                data: "[]".to_string(),
                error: err.to_string(),
            }
        }
    };

    let mut query = dsl::document.into_boxed();

    let mut limit = 50;

    let mut search: String; // = "".to_string();

    //loop via URL parameter
    for pair in parsed_url.query_pairs() {
        info!(?pair, "document url pair");
        //------------------------------------
        if pair.0 == "rows" {
            //limit of rows parameter
            match pair.1.parse::<i64>() {
                Ok(v) => {
                    limit = v;
                }
                _ => {}
            };
        }
        //------------------------------------
        if pair.0 == "sort" {
            //sort parameter
            let mut sort_field_iter = pair.1.split_whitespace();
            let sort_field_name = sort_field_iter.next().unwrap_or(r#""#);
            let sort_field_order = sort_field_iter.next().unwrap_or(r#""#);
            match sort_field_name {
                "date" => {
                    if sort_field_order == "desc" {
                        query = query.order_by(dsl::date.desc());
                    } else {
                        query = query.order_by(dsl::date.asc());
                    }
                }
                "subject" => {
                    if sort_field_order == "desc" {
                        query = query.order_by(dsl::subject.desc())
                    } else {
                        query = query.order_by(dsl::subject.asc())
                    }
                }
                "status" => {
                    if sort_field_order == "desc" {
                        query = query.order_by(dsl::status.desc());
                    } else {
                        query = query.order_by(dsl::status.asc());
                    }
                }
                "amount" => {
                    if sort_field_order == "desc" {
                        query = query.order_by(dsl::amount.desc());
                    } else {
                        query = query.order_by(dsl::amount.asc());
                    }
                }
                _ => query = query.order_by(dsl::date.desc()),
            }
        }
        //------------------------------------
        if pair.0 == "q" {
            //where parameter
            let mut filter_field_iter = pair.1.split(':');
            let filter_field_name = filter_field_iter.next().unwrap_or(r#""#);
            let filter_field_match = filter_field_iter.next().unwrap_or(r#""#);
            //the `*`from the transfer string into placeholder `%`for the selection
            search = String::from(str::replace(&filter_field_match, "*", "%"));
            match filter_field_name {
                "body" => query = query.filter(dsl::body.like(search)),
                "subject" => query = query.filter(dsl::subject.like(search)),
                "status" => query = query.filter(dsl::status.like(search)),
                "date" => query = query.filter(dsl::date.eq(search)),
                "amount" => {
                    //Conversion of the transfer string into a number
                    query = query.filter(dsl::amount.eq(search.parse::<f64>().unwrap_or(0_f64)))
                }
                "sender_name" => query = query.filter(dsl::sender_name.like(search)),
                "recipient_name" => query = query.filter(dsl::recipient_name.like(search)),
                "category" => query = query.filter(dsl::category.like(search)),

                _ => {}
            };
        }
    }

    let database_name = format!("{}/{}", MAIN_PATH, DATABASE_NAME);
    let mut conn = establish_connection(&database_name);

    use crate::diesel::sqlite::Sqlite;
    let exec_query = query
        .limit(limit)
        .filter(dsl::deleted_at.is_null())
        .select(DocumentSmall::as_select());
    info!("debug sql\n{}", debug_query::<Sqlite, _>(&exec_query));

    match exec_query.load::<DocumentSmall>(&mut conn)
    {
        Ok(result) => Response {
            dataname: path,
            data: json!(&result).to_string(),
            error: String::from(""),
        },
        Err(err) => Response {
            dataname: path,
            data: "[]".to_string(),
            error: err.to_string(),
        },
    }
}
