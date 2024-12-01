#![allow(unused)]
#![allow(clippy::all)]

use crate::database::*;
use crate::models::*;
use crate::schema::document;
use crate::schema::document::dsl;

use crate::diesel::sqlite::Sqlite;
use diesel::{debug_query, insert_into, prelude::*};
use serde_json::json;
use tracing::{error, info};

use tokio::sync::Mutex;

/**  
 * async copy a document to the clone database
 *  * **/
pub async fn do_copy_document_to_clone(
    clone_dir: String,
    document: Document,
    mutex_conn_clone: Mutex<SqliteConnection>,
) -> Result<(i32), std::io::Error> {
    
    let l_result: i32 = 'clone_db: {
        let exec_query_clone = dsl::document
            .filter(dsl::id.eq(document.id.clone())) //genau diese ID
            .select(DocumentSmall::as_select());
        info!("debug sql\n{}", debug_query::<Sqlite, _>(&exec_query_clone));

        let mut conn_clone = mutex_conn_clone.lock().await;

        match exec_query_clone.first::<DocumentSmall>(&mut *conn_clone) {
            Ok(_) => {} //already exists
            Err(err) => {
                info!(?document.id, "do copy of document");
                match insert_into(document::dsl::document)
                    .values(&document)
                    .execute(&mut *conn_clone)
                {
                    Ok(_) => {
                        //copy PDF file and JSON file
                        if document.file.clone().unwrap_or("".to_string()).is_empty() == true {
                            break 'clone_db 1;
                        }

                        let home_dir = home::home_dir().unwrap_or("".into());

                        let pdf_file_from = format!(
                            "{}/{}/{}/{}{}",
                            home_dir.to_str().unwrap_or("").to_string(),
                            MAIN_PATH,
                            FILE_PATH,
                            document.sub_path.clone().unwrap_or("".to_string()),
                            document.file.clone().unwrap_or("".to_string())
                        );

                        //check path exists
                        let pdf_path = format!(
                            "{}/{}/{}/{}",
                            clone_dir.clone(),
                            MAIN_PATH,
                            FILE_PATH,
                            document.sub_path.clone().unwrap_or("".to_string())
                        );

                        if std::path::Path::new(&pdf_path).exists() == false {
                            let _ = std::fs::create_dir_all(&pdf_path);
                        }

                        let pdf_file_to = format!(
                            "{}/{}/{}/{}{}",
                            clone_dir.clone(),
                            MAIN_PATH,
                            FILE_PATH,
                            document.sub_path.clone().unwrap_or("".to_string()),
                            document.file.clone().unwrap_or("".to_string())
                        );

                        match std::fs::copy(&pdf_file_from, &pdf_file_to) {
                            Ok(_) => {
                                info!("file copy to {}", pdf_file_to.clone())
                            }
                            Err(err) => {
                                error!("error write file {}: {}", pdf_file_to.clone(), err);
                                break 'clone_db 2;
                            }
                        };
                    }
                    Err(err) => {
                        error!(?err, "Insert document to clone DB Error: ");
                        break 'clone_db 3;
                    }
                };
            }
        };
        99
    };

    Ok(l_result)
}
