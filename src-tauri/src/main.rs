// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(target_os = "macos")]
extern crate diesel;

use chrono::Duration;
use home::home_dir;
use serde_json::json;
use std::io::{self, Write};
use std::{env, fs};

use diesel::prelude::*;
use tokio::sync::Mutex;
use tracing::{error, info};
use tracing_subscriber;
use unicode_truncate::UnicodeTruncateStr;

use crate::do_loop::*;
use crate::do_status_message_handler::*;
use crate::document_message_handler::*;
use crate::pdf_message_handler::*;
use crate::save_document_message_handler::*;
use crate::upload_files_message_handler::*;

use crate::database::{establish_connection, DATABASE_NAME, FILE_PATH, MAIN_PATH, TEMPLATE_PATH};
use crate::schema::*;

mod database;
mod models;
mod schema;

mod dot_env;

mod do_loop;
mod do_status_message_handler;
mod document_message_handler;
mod parse;
mod pdf_message_handler;
mod save_document_message_handler;
mod upload_files_message_handler;

mod migrate_db;
mod save_json;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct User {
    email: String,
    name: String,
    path_name: String,
    clone_path: String,
    scan_path: String,
    scan_filter: String,
    avatar: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct EchartData {
    pub x_value: String,
    pub y_value: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Default)]
pub struct MainData {
    pub main_path: String,
    pub email: String,
    pub name: String,
    pub clone_dir: String,
    pub scan_path: String,
    pub scan_filter: String,
    pub all_owner: bool,
    pub refresh_token: Option<oauth2::RefreshToken>,
}
/// # AppData
/// is managed via the tauri app
pub struct AppData {
    pub main_data: Mutex<MainData>,
    pub db: Mutex<SqliteConnection>,
}

/// # MainData
/// are the central data of the application and are stored in a local file and
/// read with the start of the server or initialized if the file does not yet exist.
impl MainData {
    ///constructor from main_Data as clone()
    pub fn new(main_data: &MainData) -> Self {
        info!("MainData new()");

        MainData {
            main_path: main_data.main_path.clone(),
            email: main_data.email.clone(),
            name: main_data.name.clone(),
            clone_dir: main_data.clone_dir.clone(),
            scan_path: main_data.scan_path.clone(),
            scan_filter: main_data.scan_filter.clone(),
            all_owner: main_data.all_owner.clone(),
            refresh_token: None,
        }
    }

    ///consturctor from file
    pub fn init_main_data() -> Self {
        info!("MainData init_main_data()");

        let home_dir = home_dir().unwrap_or("".into());

        let file_and_path = format!(
            "{}/{}",
            home_dir.to_str().unwrap_or("").to_string(),
            database::MAIN_DATA_FILENAME
        );

        use std::fs::read_to_string;
        let main_data_string = read_to_string(file_and_path).unwrap_or("".to_string());

        let main_data = match serde_json::from_str(&main_data_string) {
            Ok(result) => result,
            Err(err) => {
                error!(?err, "Error: ");
                MainData {
                    main_path: database::MAIN_PATH.to_string(),
                    email: "".to_string(),
                    name: "".to_string(),
                    clone_dir: "".to_string(),
                    scan_path: "".to_string(),
                    scan_filter: "Scan*.pdf".to_string(),
                    all_owner: false,
                    refresh_token: None,
                }
            }
        };
        info!("main_data: {:#?}", main_data);
        return main_data;
    }

    ///set and save the main_data
    pub fn set(
        &mut self,
        main_path: String,
        email: String,
        name: String,
        clone_dir: String,
        scan_path: String,
        scan_filter: String,
    ) {
        self.main_path = main_path;
        self.email = email;
        self.name = name;
        self.clone_dir = clone_dir;
        self.scan_path = scan_path;
        self.scan_filter = scan_filter;
        self.save_me();
    }

    pub fn set_token(&mut self, refresh_token: Option<oauth2::RefreshToken>) {
        self.refresh_token = refresh_token.clone();
        self.save_me();
    }

    ///save main_Data in file
    pub fn save_me(&self) {
        info!("MainData save_me()");

        let home_dir = home_dir().unwrap_or("".into());

        let file_and_path = format!(
            "{}/{}",
            home_dir.to_str().unwrap_or("").to_string(),
            database::MAIN_DATA_FILENAME
        );

        let main_data_json = json!(self).to_string();

        match fs::write(file_and_path, main_data_json) {
            Ok(_) => {}
            Err(err) => {
                error!(?err, "Error: ");
            }
        };
    }
}

fn check_file(file_name: &str) -> (bool, bool) {
    match fs::metadata(file_name) {
        Ok(data) => (data.is_dir(), data.is_file()),
        Err(_) => (false, false),
    }
}

/// # generate_directory_database
/// is called when the server is started so that the working directories
/// and database files are present
/// it use the consts from database.rs mod
/// * `MAIN_PATH` - under the home directory
/// * `FILE_PATH` - path for the pdf-files unter MAIN_PATH
/// * `DATABASE_NAME` - the name of the database
fn generate_directory_database(i_owner: String) {
    info!("generate_directory_database()");

    //define and generate directory structure
    let home_dir = home_dir().unwrap_or("".into());
    let my_main_path = format!("{}/{}", home_dir.to_string_lossy(), MAIN_PATH);
    info!(?my_main_path, "main path");

    fs::create_dir_all(my_main_path)
        .unwrap_or_else(|_| panic!("Error when creating the working directory: {}", MAIN_PATH));

    let my_data_path = format!("{}/{}/{}", home_dir.to_string_lossy(), MAIN_PATH, FILE_PATH);
    info!(?my_data_path, "data path");

    fs::create_dir_all(my_data_path)
        .unwrap_or_else(|_| panic!("Error when creating the data directory: {}", FILE_PATH));

    let my_template_path = format!(
        "{}/{}/{}",
        home_dir.to_string_lossy(),
        MAIN_PATH,
        TEMPLATE_PATH
    );
    info!(?my_template_path, "template path");

    fs::create_dir_all(my_template_path).unwrap_or_else(|_| {
        panic!(
            "Error when creating the template directory: {}",
            TEMPLATE_PATH
        )
    });

    //if there is no DB file yet, look and check if a migration has to be done
    let mut db_migration = false;
    let my_db_name = format!(
        "{}/{}/{}",
        home_dir.to_string_lossy(),
        MAIN_PATH,
        DATABASE_NAME
    );

    let my_db_migrate = format!(
        "{}/{}/{}",
        home_dir.to_string_lossy(),
        MAIN_PATH,
        "megarecords.db"
    );

    if check_file(&my_db_name) == (false, false) //current DB does not exist yet
        && check_file(&my_db_migrate) == (false, true)
    {
        //migrations DB exists
        //no directory and no file, but a migration database
        db_migration = true;
    }

    //define database and create table IF NOT EXISTS
    let database_name = format!("{}/{}", MAIN_PATH, DATABASE_NAME);
    let conn = establish_connection(&database_name);
    schema::check_tables(conn)
        .unwrap_or_else(|e| panic!("Error connecting to the database: {}", e));

    if db_migration == true {
        //now current DB is initialized and there is one for migration
        let mig_database_name = format!("{}/{}", MAIN_PATH, "megarecords.db");

        use crate::migrate_db::*;
        tauri::async_runtime::spawn(async move {
            //the mirgation is async
            migrate_db(
                establish_connection(&database_name),
                establish_connection(&mig_database_name),
                i_owner,
            )
            .await;
        });
    }

    // if there is no gosseract.ini file yet, create it
    let l_gosseract = format!(
        "{}/{}/{}/gosseract.ini",
        home_dir.to_str().unwrap_or("").to_string(),
        MAIN_PATH,
        FILE_PATH
    );

    if check_file(&l_gosseract) == (false, false) {
        //gosseract.ini
        fs::write(
            l_gosseract,
            "tessedit_pageseg_mode 4\ntessedit_ocr_engine_mode 1".to_string(),
        )
        .unwrap();
    }
}

// A function that sends a message from Rust to JavaScript via a Tauri Event
pub fn rs2js<R: tauri::Runtime>(message: String, manager: &impl tauri::Manager<R>) {
    let mut sub_message = message.clone();
    let (truc_msg, _) = sub_message.unicode_truncate(50);
    sub_message = truc_msg.to_string();
    info!(?sub_message, "rs2js");
    match manager.emit_all("rs2js", message) {
        Ok(_) => {}
        Err(err) => {
            error!(?err);
        }
    };
}

/// The Tauri command that gets called when Tauri `invoke` JavaScript API is called
#[tauri::command(async)]
async fn js2rs(
    window: tauri::Window,
    message: String,
    app_data: tauri::State<'_, AppData>,
) -> Result<String, String> {
    let mut sub_message = message.clone();
    let (msg_truc, _) = sub_message.unicode_truncate(50);
    sub_message = msg_truc.to_string();
    info!(?sub_message, "js2rs");

    let my_message_data: Receiver = match serde_json::from_str(message.as_str()) {
        Ok(data) => data,
        Err(err) => {
            error!("Error: {}", err);
            return Ok(json!(Response {
                dataname: "".to_string(),
                data: "[]".to_string(),
                error: err.to_string(),
            })
            .to_string());
        }
    };

    //mapping for usering

    let data = my_message_data.data;
    let path = my_message_data.path;
    let query = my_message_data.query;

    // info
    let mut my_data = data.clone();

    let (my_data_trunc, _) = my_data.unicode_truncate(150);
    my_data = my_data_trunc.to_string();

    let message = format!(
        "path: {}, query: {}, data: {}",
        path.as_str().clone(),
        query.as_str().clone(),
        my_data
    );
    info!(message, "message_handler");
    io::stdout().flush().unwrap_or(());

    let e_message = match path.as_str() {
        //----
        "user" => {
            let home_dir = home::home_dir().unwrap_or("".into());
            let message = format!("Your home directory, probably: {}", home_dir.display());
            info!(message, "message_handler");

            let main_data = app_data.main_data.lock().await;

            let my_data = json!(User {
                email: main_data.email.clone(),
                name: main_data.name.clone(),
                path_name: main_data.main_path.clone(),
                clone_path: main_data.clone_dir.clone(),
                scan_path: main_data.scan_path.clone(),
                scan_filter: main_data.scan_filter.clone(),
                avatar: "".to_string()
            })
            .to_string();

            Response {
                dataname: "me".to_string(),
                data: my_data,
                error: "".to_string(),
            }
        }
        //----
        "save_user" => {
            let my_user_data: User = match serde_json::from_str(&data) {
                Ok(result) => result,
                Err(err) => {
                    error!(?err, "Error: ");

                    return Ok(json!(Response {
                        dataname: data,
                        data: "[]".to_string(),
                        error: format!("{}", err),
                    })
                    .to_string());
                }
            };

            let mut main_data = app_data.main_data.lock().await;

            main_data.set(
                my_user_data.path_name.clone(),
                my_user_data.email.clone(),
                my_user_data.name.clone(),
                my_user_data.clone_path.clone(),
                my_user_data.scan_path.clone(),
                my_user_data.scan_filter.clone(),
            );

            let my_data = json!(my_user_data).to_string();

            Response {
                dataname: "me".to_string(),
                data: my_data,
                error: "".to_string(),
            }
        }
        //----
        "category" => {
            let category = [
                "Rechnung",
                "Auftrag",
                "Gas",
                "Strom",
                "Auto",
                "Steuern",
                "Versicherung",
                "Bank",
                "Schule",
                "Familie",
                "Haus",
                "SPAM",
                "Rest",
                "unbekannt",
            ];

            Response {
                dataname: path,
                data: json!(&category).to_string(),
                error: String::from(""),
            }
        }
        //-----
        "status" => {
            let status = [
                "01_Leer",
                "11_Rohdaten",
                "12_Rohdaten_bereinigt",
                "21_OCR",
                "31_Parse",
                "99_End",
            ];
            Response {
                dataname: path,
                data: json!(&status).to_string(),
                error: String::from(""),
            }
        }
        //------
        "chart_count" | "chart_amount" => {
            use chrono::prelude::*;
            let mut local_end: DateTime<Local> = Local::now();
            let month_duration = Duration::days(30);
            let mut local_start = Local::now()
                .checked_sub_signed(month_duration)
                .unwrap_or(Local::now());

            let mut vec_my_data: Vec<EchartData> = Vec::new();
            for n in 1..13 {
                use crate::diesel::sqlite::Sqlite;
                use crate::schema::document::dsl;
                use diesel::debug_query;
                use diesel::prelude::*;

                let x_value: String = format!("{}/{}", local_start.month(), local_start.year());
                let operation: &str;
                if path.as_str() == "chart_count" {
                    operation = "count(id) AS count";
                } else {
                    operation = "sum(amount) AS sum";
                }

                use diesel::dsl::sql;
                use diesel::sql_types::Double;

                let exec_query = document::table
                    .into_boxed()
                    .filter(
                        dsl::deleted_at
                            .is_null()
                            .and(dsl::date.le(local_start.to_string()))
                            .and(dsl::date.ge(local_end.to_string()))
                            .and(dsl::category.like(format!("%{}%", query)))
                            .and(dsl::amount.is_not_null()),
                    )
                    .select(sql::<Double>(operation));

                if n == 1 {
                    info!("debug first sql\n{}", debug_query::<Sqlite, _>(&exec_query));
                }

                let mut conn = app_data.db.lock().await;
                let mut y_value = exec_query.first::<f64>(&mut *conn).unwrap_or(0_f64);

                y_value = (y_value * 100.0).round() / 100.0; //round 2 digits

                info!("step {} x:{} y:{}", n, &x_value, &y_value);
                vec_my_data.push(EchartData {
                    x_value: x_value,
                    y_value: y_value.to_string(),
                });

                //shift to next time slot
                local_start = local_end
                    .checked_sub_days(chrono::Days::new(1))
                    .unwrap_or(local_end);
                local_end = local_start
                    .checked_sub_signed(month_duration)
                    .unwrap_or(local_start);
            }
            //
            if path.as_str() == "chart_count" {
                Response {
                    dataname: "count".to_string(),
                    data: json!(&vec_my_data).to_string(),
                    error: "".to_string(),
                }
            } else {
                //else chart_amount
                Response {
                    dataname: "amount".to_string(),
                    data: json!(&vec_my_data).to_string(),
                    error: "".to_string(),
                }
            }
        }

        "document" => document_message_handler(app_data, path, query).await,
        "save_document" => save_document_message_handler(app_data, path, query, data).await,
        "upload_files" => upload_files_message_handler(window, app_data, path, query, data).await,
        "dostatus" => do_status_message_handler(window, app_data, path, query, data).await,

        "doloop" => {
            tauri::async_runtime::spawn(async move {
                do_loop(window).await;
            });

            Response {
                dataname: "info".into(),
                data: json!("Loop started in the background ... .").to_string(),
                error: "".to_string(),
            }
        }

        "pdf" => pdf_message_handler(app_data, path, query, data).await,
        _ => Response {
            dataname: path.clone(),
            data: String::from(""),
            error: format!("path {} not fund", path.as_str()),
        },
    };

    Ok(json!(e_message).to_string())
}

fn main() {
    let _ = fix_path_env::fix();
    tracing_subscriber::fmt::init();

    let main_data = MainData::init_main_data();

    generate_directory_database(main_data.email.clone());

    let database_name = format!("{}/{}", MAIN_PATH, DATABASE_NAME);

    tauri::Builder::default()
        .manage(AppData {
            main_data: main_data.into(),
            db: establish_connection(&database_name).into(),
        }) // MainData to manage
        .invoke_handler(tauri::generate_handler![js2rs])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
