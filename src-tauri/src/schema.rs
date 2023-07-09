#[derive(serde::Serialize, Debug)]
pub struct Response {
    pub dataname: String,
    pub data: String,
    pub error: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Receiver {
    pub path: String,
    pub query: String,
    pub data: String,
}

table! {
    document (id) {
        id -> Text,
        subject -> Text,
        status -> Text,
        date -> Timestamp,
        sender_name -> Nullable<Text>,
        sender_addr -> Nullable<Text>,
        recipient_name -> Nullable<Text>,
        recipient_addr -> Nullable<Text>,
        from -> Nullable<Text>,
        to -> Nullable<Text>,
        body -> Nullable<Text>,
        #[sql_name = "type"]
        document_type -> Nullable<Text>,
        metadata -> Nullable<Text>,
        category -> Nullable<Text>,
        amount -> Nullable<Double>,
        currency -> Nullable<Text>,
        template_name -> Nullable<Text>,
        doc_data -> Nullable<Text>,
        input_path -> Nullable<Text>,
        langu -> Nullable<Text>,
        num_pages -> Nullable<Double>,
        protocol -> Nullable<Text>,
        sub_path -> Nullable<Text>,
        filename -> Nullable<Text>,
        file_extension -> Nullable<Text>,
        file -> Nullable<Text>,
        base64 -> Nullable<Text>,
        ocr_data -> Nullable<Text>,
        jpg_file -> Nullable<Text>,
        parent_document -> Nullable<Text>,
        #[sql_name = "createdAt"]
        created_at -> Timestamp,
        #[sql_name = "updatedAt"]
        updated_at -> Timestamp,
        #[sql_name = "deletedAt"]
        deleted_at -> Nullable<Timestamp>,
    }
}

table! {
    ftp_data (id) {
        id -> Nullable<Integer>,
        host -> Text,
        user -> Nullable<Text>,
        password -> Nullable<Text>,
        dir -> Nullable<Text>,
        filter -> Nullable<Text>,
        #[sql_name = "createdAt"]
        created_at -> Timestamp,
        #[sql_name = "updatedAt"]
        updated_at -> Timestamp,
        #[sql_name = "deletedAt"]
        deleted_at -> Nullable<Timestamp>,
    }
}

table! {
    mail_data (id) {
        id -> Nullable<Integer>,
        email -> Text,
        #[sql_name = "box"]
        mail_box -> Nullable<Text>,
        token_access_token -> Nullable<Text>,
        token_token_type -> Nullable<Text>,
        token_refresh_token -> Nullable<Text>,
        token_expiry -> Nullable<Text>,
        #[sql_name = "createdAt"]
        created_at -> Timestamp,
        #[sql_name = "updatedAt"]
        updated_at -> Timestamp,
        #[sql_name = "deletedAt"]
        deleted_at -> Nullable<Timestamp>,
    }
}

allow_tables_to_appear_in_same_query!(document, ftp_data, mail_data,);

use diesel::prelude::*;
use tracing::info;
pub fn check_tables(mut con: diesel::SqliteConnection) -> Result<usize, diesel::result::Error> {
    info!("start check_tables()");

    info!("CREATE TABLE IF NOT EXISTS `document` ...");
    let mut sql_sing = concat!(
        "CREATE TABLE IF NOT EXISTS `document` (",
        "`id` TEXT NOT NULL PRIMARY KEY, ",
        "`subject` TEXT NOT NULL, ",
        "`status` TEXT NOT NULL, ",
        "`date` DATETIME NOT NULL, ",
        "`sender_name` TEXT DEFAULT '', ",
        "`sender_addr` TEXT DEFAULT '', ",
        "`recipient_name` TEXT DEFAULT '', ",
        "`recipient_addr` TEXT DEFAULT '', ",
        "`from` TEXT DEFAULT '[]', ",
        "`to` TEXT DEFAULT '[]', ",
        "`body` TEXT DEFAULT '', ",
        "`type` TEXT DEFAULT '', ",
        "`metadata` TEXT DEFAULT '{}', ",
        "`category` TEXT DEFAULT '[]', ",
        "`amount` DECIMAL(10,2) DEFAULT 0, ",
        "`currency` TEXT DEFAULT '', ",
        "`template_name` TEXT DEFAULT '', ",
        "`doc_data` TEXT DEFAULT '{}', ",
        "`input_path` TEXT DEFAULT '', ",
        "`langu` TEXT DEFAULT '', ",
        "`num_pages` NUMBER DEFAULT 0, ",
        "`protocol` TEXT DEFAULT '', ",
        "`sub_path` TEXT DEFAULT '', ",
        "`filename` TEXT DEFAULT '', ",
        "`file_extension` TEXT DEFAULT '',",
        "`file` TEXT DEFAULT '', ",
        "`base64` TEXT DEFAULT '', ",
        "`ocr_data` TEXT DEFAULT '', ",
        "`jpg_file` TEXT DEFAULT '[]', ",
        "`parent_document` TEXT DEFAULT '', ",
        "`createdAt` DATETIME NOT NULL, ",
        "`updatedAt` DATETIME NOT NULL, ",
        "`deletedAt` DATETIME ",
        ");"
    );
    diesel::sql_query(sql_sing.to_string()).execute(&mut con)?;

    info!("CREATE TABLE IF NOT EXISTS `ftp_data` ...");
    sql_sing = concat!(
        "CREATE TABLE IF NOT EXISTS `ftp_data` (`id` INTEGER PRIMARY KEY AUTOINCREMENT, ",
        "`host` TEXT NOT NULL, ",
        "`user` TEXT, ",
        "`password` TEXT, ",
        "`dir` TEXT, ",
        "`filter` TEXT, ",
        "`createdAt` DATETIME NOT NULL, ",
        "`updatedAt` DATETIME NOT NULL, ",
        "`deletedAt` DATETIME);"
    );
    diesel::sql_query(sql_sing.to_string()).execute(&mut con)?;

    info!("CREATE TABLE IF NOT EXISTS `mail_data` ...");
    sql_sing = concat!(
        "CREATE TABLE IF NOT EXISTS `mail_data` (",
        "`id` INTEGER PRIMARY KEY AUTOINCREMENT, ",
        "`email` TEXT NOT NULL, ",
        "`box` TEXT, ",
        "`token_access_token` TEXT, ",
        "`token_token_type` TEXT, ",
        "`token_refresh_token` TEXT, ",
        "`token_expiry` TEXT, ",
        "`createdAt` DATETIME NOT NULL, ",
        "`updatedAt` DATETIME NOT NULL, ",
        "`deletedAt` DATETIME);"
    );
    diesel::sql_query(sql_sing.to_string()).execute(&mut con)    
}
