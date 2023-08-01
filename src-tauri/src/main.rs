// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(target_os = "macos")]
extern crate diesel;

use chrono::Duration;
use home::home_dir;
use serde_json::json;
use std::env;
use std::fs;
use std::io::{self, Write};

use tauri::Manager;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tracing::{error, info};
use tracing_subscriber;

use crate::document_message_handler::*;
use crate::pdf_message_handler::*;

use crate::database::{establish_connection, DATABASE_NAME, FILE_PATH, MAIN_PATH};
use crate::schema::*;

mod database;
mod document_message_handler;
mod models;
mod pdf_message_handler;
mod schema;
mod migrate_db;

struct AsyncProcInputTx {
    inner: Mutex<mpsc::Sender<(String, AppData)>>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct UserData {
    email: String,
    name: String,
    path_name: String,
    clone_path: String,
    avatar: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct EchartData {
    pub x_value: String,
    pub y_value: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct AppData {
    pub main_path: String,
    pub email: String,
    pub name: String,
    pub clone_dir: String,
}

/// # AppData
/// are the central data of the application and are stored in a local file and
/// read with the start of the server or initialized if the file does not yet exist.
impl AppData {
    ///constructor from app_Data as clone()
    pub fn new(app_data: &AppData) -> Self {
        info!("AppData new()");

        AppData {
            main_path: app_data.main_path.clone(),
            email: app_data.email.clone(),
            name: app_data.name.clone(),
            clone_dir: app_data.clone_dir.clone(),
        }
    }

    ///consturctor from file
    pub fn init_app_data() -> Self {
        info!("AppData init_app_data()");

        let home_dir = home_dir().unwrap_or("".into());

        let file_and_path = format!(
            "{}/{}",
            home_dir.to_str().unwrap_or("").to_string(),
            database::APP_DATA_FILENAME
        );

        use std::fs::read_to_string;
        let app_data_string = read_to_string(file_and_path).unwrap_or("".to_string());

        let app_data = match serde_json::from_str(&app_data_string) {
            Ok(result) => result,
            Err(err) => {
                error!(?err, "Error: ");
                AppData {
                    main_path: database::MAIN_PATH.to_string(),
                    email: "".to_string(),
                    name: "".to_string(),
                    clone_dir: "".to_string(),
                }
            }
        };
        return app_data;
    }

    ///set and save the app_data
    pub fn set(&mut self, main_path: String, email: String, name: String, clone_dir: String) {
        self.main_path = main_path;
        self.email = email;
        self.name = name;
        self.clone_dir = clone_dir;
        self.save_me();
    }

    ///save app_Data in file
    pub fn save_me(&self) {
        info!("AppData save_me()");

        let home_dir = home_dir().unwrap_or("".into());

        let file_and_path = format!(
            "{}/{}",
            home_dir.to_str().unwrap_or("").to_string(),
            database::APP_DATA_FILENAME
        );

        let app_data_json = json!(self).to_string();

        match fs::write(file_and_path, app_data_json) {
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
fn generate_directory_database() {
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
        .unwrap_or_else(|_| panic!("Error when creating the working directory: {}", MAIN_PATH));

    //wenn es noch keine DB Datei gibt, gucken prüfen ob eine Migration durchgeführt werden muss
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

    if check_file(&my_db_name) == (false, false) //aktuelle DB existiert noch nicht
        && check_file(&my_db_migrate) == (false, true)  { //migrations DB existiert
        //kein Direktory und keine Datei, aber eine Migrationsdatenbank
        db_migration = true;
    }


    //define database and create table IF NOT EXISTS
    let database_name = format!("{}/{}", MAIN_PATH, DATABASE_NAME);
    let con = establish_connection(&database_name);
    schema::check_tables(con).unwrap_or_else(|e| panic!("Error connecting to the database: {}", e));


if db_migration == true {
    //jetzt ist aktuelle DB initialisiert und es gibt eine für die Migration
    let mig_database_name = format!("{}/{}", MAIN_PATH, "megarecords.db");

    use crate::migrate_db::*;
    migrate_db(establish_connection(&database_name), establish_connection(&mig_database_name))
   
}

}

fn main() {
    tracing_subscriber::fmt::init();

    generate_directory_database();

    let (async_proc_input_tx, async_proc_input_rx) = mpsc::channel(1);
    let (async_proc_output_tx, mut async_proc_output_rx) = mpsc::channel(1);

    tauri::Builder::default()
        .manage(AsyncProcInputTx {
            inner: Mutex::new(async_proc_input_tx),
        })
        .manage(AppData::init_app_data()) // AppData to manage
        .invoke_handler(tauri::generate_handler![js2rs])
        .setup(|app| {
            tauri::async_runtime::spawn(async move {
                async_process_model(async_proc_input_rx, async_proc_output_tx).await
            });

            let app_handle = app.handle();
            tauri::async_runtime::spawn(async move {
                loop {
                    if let Some(output) = async_proc_output_rx.recv().await {
                        rs2js(output, &app_handle);
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// A function that sends a message from Rust to JavaScript via a Tauri Event
fn rs2js<R: tauri::Runtime>(message: String, manager: &impl Manager<R>) {
    let mut sub_message = message.clone();
    sub_message.truncate(50);
    info!(?sub_message, "rs2js");
    manager.emit_all("rs2js", message).unwrap();
}

/// The Tauri command that gets called when Tauri `invoke` JavaScript API is called
#[tauri::command]
async fn js2rs(
    message: String,
    state: tauri::State<'_, AsyncProcInputTx>,
    app_data: tauri::State<'_, AppData>,
) -> Result<(), String> {
    let mut sub_message = message.clone();
    sub_message.truncate(50);
    info!(?sub_message, "js2rs");

    let async_proc_input_tx = state.inner.lock().await;
    async_proc_input_tx
        .send((message, AppData::new(app_data.inner())))
        .await
        .map_err(|e| {
            println!("{}", e.to_string());
            e.to_string()
        })
}

/// asynchronous processing of events from/to tauri server as message
async fn async_process_model(
    mut input_rx: mpsc::Receiver<(String, AppData)>,
    output_tx: mpsc::Sender<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    while let Some((message, app_data)) = input_rx.recv().await {
        let mut parse_error = false;
        let my_message_data: Receiver = match serde_json::from_str(message.as_str()) {
            Ok(data) => data,
            Err(err) => {
                parse_error = true;
                let my_output_data = Response {
                    dataname: "".to_string(),
                    data: "[]".to_string(),
                    error: err.to_string(),
                };
                let output = json!(my_output_data).to_string();
                match output_tx.send(output).await {
                    _ => {}
                }
                Receiver {
                    path: "".to_string(),
                    query: "".to_string(),
                    data: "[]".to_string(),
                }
            }
        };

        if !parse_error {
            let my_output_data: Response = message_handler(
                app_data,
                my_message_data.path,
                my_message_data.query,
                my_message_data.data,
            )
            .await;
            let output = json!(my_output_data).to_string();
            match output_tx.send(output).await {
                _ => {}
            }
        }
    }

    Ok(())
}

async fn message_handler(
    //window: tauri::Window,
    //database: tauri::State<'_, Database>,
    mut app_data: AppData,
    path: String,
    query: String,
    data: String,
) -> Response {
    let message = format!(
        "path: {}, query: {}, data: {}",
        path.as_str().clone(),
        query.as_str().clone(),
        data.as_str().clone()
    );
    info!(message, "message_handler");
    io::stdout().flush().unwrap();

    match path.as_str() {
        //----
        "user" => {
            let home_dir = home_dir().unwrap();
            let message = format!("Your home directory, probably: {}", home_dir.display());
            info!(message, "message_handler");

            let my_data = json!(UserData {
                email: app_data.email,
                name: app_data.name,
                path_name: app_data.main_path,
                clone_path: app_data.clone_dir,
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
            let my_user_data: UserData = match serde_json::from_str(&data) {
                Ok(result) => result,
                Err(err) => {
                    error!(?err, "Error: ");

                    return Response {
                        dataname: data,
                        data: "[]".to_string(),
                        error: format!("{}", err),
                    };
                }
            };

            app_data.set(
                my_user_data.path_name.clone(),
                my_user_data.email.clone(),
                my_user_data.name.clone(),
                my_user_data.clone_path.clone(),
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
            let database_name = format!("{}/{}", MAIN_PATH, DATABASE_NAME);
            let mut conn = establish_connection(&database_name);

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

                let mut y_value = exec_query.first::<f64>(&mut conn).unwrap_or(0_f64);
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

        "document" => document_message_handler(path, query).await,
        "pdf" => pdf_message_handler(path, query, data).await,
        _ => Response {
            dataname: path.clone(),
            data: String::from(""),
            error: format!("path {} not fund", path.as_str()),
        },
    }
}
