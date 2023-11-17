#![allow(unused)]
#![allow(clippy::all)]

use std::ops::Index;

use diesel::expression::is_aggregate::No;
use diesel::sql_types::Integer;
use diesel::sqlite::Sqlite;
use diesel::{debug_query, ExpressionMethods};
use diesel::{insert_into, prelude::*, sql_query};
use tracing::{error, info};

use chrono::NaiveDateTime;
use chrono::{DateTime, Local, TimeZone};
use diesel::{Insertable, Queryable, Selectable, Table};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::models::Document;
use crate::schema::document;
use crate::models::MailData;
use crate::schema::mail_data;

table! {
    record (id) {
        id -> Text,
        status -> Text,
        date -> Timestamp,
        sender_name -> Nullable<Text>,
        sender_addr -> Nullable<Text>,
        recipient_name -> Nullable<Text>,
        recipient_addr -> Nullable<Text>,
        subject -> Text,
        body -> Nullable<Text>,
        category -> Nullable<Text>,
        amount -> Nullable<Double>,
        currency -> Nullable<Text>,
        template_name -> Nullable<Text>,
        deleted -> Nullable<Text>,
        input_path -> Nullable<Text>,
        langu -> Nullable<Text>,
        protocol -> Nullable<Text>,
        sub_path -> Nullable<Text>,
      }
}

table! {
    attachment (id) {
        id -> Integer,
        filename -> Nullable<Text>,
        file_extension -> Nullable<Text>,
        file -> Nullable<Text>,
        ocr_data -> Nullable<Text>,
        jpg_file -> Nullable<Text>,
        record_refer -> Text,
      }
}

table! {
    email (id) {
        id -> Integer,
        name -> Nullable<Text>,
        #[sql_name = "email"]
        email_partner -> Nullable<Text>,
        id_partner -> Nullable<Text>,
        record_refer -> Text,
      }
}

table! {
    #[sql_name = "mail_data"]
    mail_data_old (email) {
        email -> Text,
        #[sql_name = "box"]
        mail_box -> Nullable<Text>,
        token_access_token -> Nullable<Text>,
        token_token_type -> Nullable<Text>,
        token_refresh_token -> Nullable<Text>,
        token_expiry -> Nullable<Text>,
      }
}

#[derive(Serialize, Deserialize, Debug, Queryable, Selectable)]
#[diesel(table_name = self::record)]
pub struct Record {
    pub id: String,
    pub status: String,
    pub date: String,
    pub sender_name: Option<String>,
    pub sender_addr: Option<String>,
    pub recipient_name: Option<String>,
    pub recipient_addr: Option<String>,
    pub subject: String,
    pub body: Option<String>,
    pub category: Option<String>,
    pub amount: Option<f64>,
    pub currency: Option<String>,
    pub template_name: Option<String>,
    pub deleted: Option<String>,
    pub input_path: Option<String>,
    pub protocol: Option<String>,
    pub sub_path: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Queryable, Selectable)]
#[diesel(table_name = self::attachment)]
pub struct Attachment {
    pub id: i32,
    pub filename: Option<String>,
    pub file: Option<String>,
    pub ocr_data: Option<String>,
    pub jpg_file: Option<String>,
    pub record_refer: String,
}

#[derive(Serialize, Deserialize, Debug, Queryable, Selectable)]
#[diesel(table_name = self::email)]
pub struct Email {
    pub id: i32,
    pub name: Option<String>,
    pub email_partner: Option<String>,
    pub id_partner: Option<String>,
    pub record_refer: String,
}

#[derive(Serialize, Deserialize, Debug, Queryable, Selectable)]
#[diesel(table_name = self::mail_data_old)]
pub struct MailDataOld {
    pub email: String,
    pub mail_box: Option<String>,
    pub token_access_token: Option<String>,
    pub token_token_type: Option<String>,
    pub token_refresh_token: Option<String>,
    pub token_expiry: Option<String>,
}

pub fn get_deleted_at(i_date: Option<String>) -> Option<String> {
    match i_date {
        None => None,
        _ => {
            let my_date = i_date.unwrap_or("".to_string()).clone();
            if my_date.contains("0001-01-01") {
                None
            } else {
                Some(my_date)
            }
        }
    }
}

pub fn conv_obj_to_array(i_data: String) -> String {
    i_data
        .chars()
        .map(|x| match x {
            '{' => '[',
            '}' => ']',
            _ => x,
        })
        .collect()
}

#[tauri::command(async)]
pub async fn migrate_db(
    mut akt_con: diesel::SqliteConnection,
    mut mig_con: diesel::SqliteConnection,
    i_owner: String,
) {
    info!("migrate_db()");

    let exec_query = self::record::dsl::record
        .select(Record::as_select()); //sql_query("SELECT * FROM `record` ");

    info!("debug sql\n{}", debug_query::<Sqlite, _>(&exec_query));

    let data = match exec_query.load::<Record>(&mut mig_con) {
        Ok(exec_data) => exec_data,
        Err(_) => return
    };

    for ele in data {
        //print!("{:?}", ele);
        //use crate::schema::document::dsl::document;
        let category_array = conv_obj_to_array(ele.category.unwrap_or("[]".to_string()));
        //println!("category_array {:?}", &category_array);
        let amount_round = (ele.amount.unwrap_or(0_f64) * 100.0).round() / 100.0; //round 2 digits,

        //-------------
        // attachmant
        let exec_query_attachment = self::attachment::dsl::attachment
            .filter(self::attachment::dsl::record_refer.eq(&ele.id))
            .select(Attachment::as_select()); //sql_query("SELECT * FROM `record` ");

        // info!(
        //     "debug sql\n{}",
        //     debug_query::<Sqlite, _>(&exec_query_attachment)
        // );

        // let data_attachment = exec_query_attachment
        //     .first::<Attachment>(&mut mig_con)
        //     .unwrap_or(Attachment {
        //         id: 0_i32,
        //         filename: None,
        //         file: None,
        //         ocr_data: None,
        //         jpg_file: None,
        //         record_refer: "".to_string(),
        //     });

        let mut data_attachment = Attachment {
            id: 0_i32,
            filename: None,
            file: None,
            ocr_data: None,
            jpg_file: None,
            record_refer: "".to_string(),
        };

        let mut vec_attachment = match exec_query_attachment.load::<Attachment>(&mut mig_con) {
            Ok(data) => data,
            Err(_) => Vec::new(),
        };
        //info!("{:?}", data_attachment.file);

        //--------------

        //-------------
        // email
        let exec_query_email = self::email::dsl::email
            .filter(self::email::dsl::record_refer.eq(&ele.id))
            .select(Email::as_select()); //sql_query("SELECT * FROM `record` ");

        // info!(
        //     "debug sql\n{}",
        //     debug_query::<Sqlite, _>(&exec_query_email)
        // );

        #[derive(Serialize, Deserialize, Debug)]
        struct Partner {
            name: String,
            email: String,
        }

        let mut from_partner = Partner {
            name: "".to_string(),
            email: "".to_string(),
        };
        let mut to_partner = Partner {
            name: "".to_string(),
            email: "".to_string(),
        };
        match exec_query_email.load::<Email>(&mut mig_con) {
            Ok(data_email) => {
                let mut count = 0;
                for ele_email in data_email {
                    count = count + 1;
                    if count == 1 {
                        //1ter Einteag ist "from_partne"
                        from_partner = Partner {
                            name: ele_email.name.unwrap_or("".to_string()),
                            email: ele_email.email_partner.unwrap_or("".to_string()),
                        }
                    } else if count == 2 {
                        //2ter Eintrag ist "to_partner"
                        to_partner = Partner {
                            name: ele_email.name.unwrap_or("".to_string()),
                            email: ele_email.email_partner.unwrap_or("".to_string()),
                        }
                    }
                }
            }
            Err(_) => {}
        };

        //info!("{:?}", data_attachment.file);

        //init while loop
        let mut n = 1;
        let mut my_parent_document: Option<String> = None; //the first document is the parent for all further

        while n <= vec_attachment.len() || n == 1 && vec_attachment.len() == 0 {
            if vec_attachment.len() != 0 {
                data_attachment = match vec_attachment.pop() {
                    Some(data) => data,
                    None => Attachment {
                        id: 0_i32,
                        filename: None,
                        file: None,
                        ocr_data: None,
                        jpg_file: None,
                        record_refer: "".to_string(),
                    },
                }
            }

            let mut my_id = ele.id.clone();
            if n != 1 {
                my_id = format!("{}{}", ele.id.clone(), n)
            }

            //--------------

            match insert_into(document::dsl::document)
                .values(Document {
                    id: my_id,
                    subject: ele.subject.clone(),
                    status: ele.status.clone(),
                    date: ele.date.clone(),
                    sender_name: ele.sender_name.clone(),
                    sender_addr: ele.sender_addr.clone(),
                    recipient_name: ele.recipient_name.clone(),
                    recipient_addr: ele.recipient_addr.clone(),
                    from: Some(json!(from_partner).to_string()),
                    to: Some(json!(to_partner).to_string()),
                    body: ele.body.clone(),
                    document_type: Some("PDF".to_string()),
                    metadata: None,
                    //ersetzen {} in []
                    category: Some(category_array.clone()),
                    //wert runden auf 2 Nachkommastellen
                    amount: Some(amount_round),
                    currency: ele.currency.clone(),
                    template_name: ele.template_name.clone(),
                    doc_data: None,
                    input_path: ele.input_path.clone(),
                    langu: Some("DE".to_string()),
                    num_pages: None,
                    protocol: ele.protocol.clone().unwrap_or("".to_string()),
                    sub_path: ele.sub_path.clone(),
                    filename: data_attachment.filename.clone(),
                    file_extension: None,
                    file: data_attachment.file.clone(),
                    base64: None,
                    ocr_data: data_attachment.ocr_data.clone(),
                    jpg_file: Some(conv_obj_to_array(
                        data_attachment.jpg_file.clone().unwrap_or("[]".to_string()),
                    )),
                    parent_document: my_parent_document.clone(),
                    owner: i_owner.clone().to_lowercase(),
                    created_at: Local::now().to_string(),
                    updated_at: "".to_string(),
                    deleted_at: get_deleted_at(ele.deleted.clone()),
                })
                .execute(&mut akt_con)
            {
                Ok(_) => {}
                Err(err) => {
                    error!("insert document: {}", err)
                }
            };

            if n == 1 {
                //the 1st document will be the parent for all subsequent ones!
                my_parent_document = Some(ele.id.clone());
            }
            n += 1;
        }
    }

    //---------

    let exec_query_mail = self::mail_data_old::dsl::mail_data_old
        .select(MailDataOld::as_select()); //sql_query("SELECT * FROM `mail_data` ");

    info!("debug sql\n{}", debug_query::<Sqlite, _>(&exec_query_mail));

    let data_mail = exec_query_mail.load::<MailDataOld>(&mut mig_con).unwrap();

    for ele in data_mail {
        match insert_into(mail_data::dsl::mail_data)
            .values(MailData {
                email: ele.email,
                mail_box: ele.mail_box,
                token_access_token: ele.token_access_token,
                token_token_type: ele.token_token_type,
                token_refresh_token: ele.token_refresh_token,
                token_expiry: ele.token_expiry,
                created_at: Local::now().to_string(),
                updated_at: "".to_string(),
                deleted_at: None,
            })
            .execute(&mut akt_con)
        {
            Ok(_) => {}
            Err(err) => {
                error!("insert document: {}", err)
            }
        };
    }

    //panic!("***")
}
