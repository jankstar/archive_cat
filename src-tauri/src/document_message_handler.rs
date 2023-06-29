#![allow(unused)]
#![allow(clippy::all)]

use crate::database::*;
use crate::models::*;
use crate::schema;
use crate::schema::Response;
use crate::schema::document::dsl;

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

    let mut my_url = "http://123".to_owned();
    my_url.push_str(query.as_ref());

    let parsed_url = Url::parse(&my_url).unwrap();

    let mut query = dsl::document.into_boxed();

    let mut limit = 50;

    let mut search: String; // = "".to_string();

    for pair in parsed_url.query_pairs() {
        info!(?pair, "document url pair");
        //------------------------------------
        if pair.0 == "rows" {
            match pair.1.parse::<i64>() {
                Ok(v) => {
                    limit = v;
                }
                _ => {}
            };
        }
        //------------------------------------
        if pair.0 == "sort" {
            let mut sort_field_iter = pair.1.split_whitespace();
            let sort_field_name = sort_field_iter.next().unwrap();
            let sort_field_order = sort_field_iter.next().unwrap();
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
            let mut filter_field_iter = pair.1.split(':');
            let filter_field_name = filter_field_iter.next().unwrap();
            let filter_field_match = filter_field_iter.next().unwrap();
            search = String::from(str::replace(&filter_field_match, "*", "%"));
            match filter_field_name {
                "body" => query = query.filter(dsl::body.like(search)),
                "subject" => query = query.filter(dsl::subject.like(search)),
                "status" => query = query.filter(dsl::status.like(search)),
                "date" => query = query.filter(dsl::date.eq(search)),
                "amount" => query = query.filter(dsl::amount.eq(search.parse::<f64>().unwrap())),
                "sender_name" => query = query.filter(dsl::sender_name.like(search)),
                "recipient_name" => query = query.filter(dsl::recipient_name.like(search)),
                "category" => query = query.filter(dsl::category.like(search)),

                _ => {}
            };
        }
    }

    let mut conn = establish_connection();

    match query
        .limit(limit)
        .filter(dsl::deleted_at.is_null())
        .select(DocumentSmall::as_select())
        .load::<DocumentSmall>(&mut conn)
    {
        Ok(result) => {
            //trim whitespace for json
            let result_string = json!(&result).to_string();
            let mut result_clone = result_string.clone();
            result_clone.retain(|c| !c.is_whitespace());
            Response {
                dataname: path,
                data: result_clone,
                error: String::from(""),
            }
        }
        Err(err) => Response {
            dataname: path,
            data: "[]".to_string(),
            error: err.to_string(),
        },
    }
}
