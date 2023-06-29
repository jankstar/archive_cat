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

allow_tables_to_appear_in_same_query!(
    document,
    ftp_data,
    mail_data,
);
