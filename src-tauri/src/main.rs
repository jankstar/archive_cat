// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(target_os = "macos")]
#[macro_use]
extern crate diesel;

use home::home_dir;
use serde_json::json;
use std::env;
use std::io::{self, Write};

use tauri::Manager;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tracing::info;
use tracing_subscriber;

use crate::document_message_handler::*;
use crate::pdf_message_handler::*;

use crate::schema::*;
mod database;
mod document_message_handler;
mod models;
mod pdf_message_handler;
mod schema;

struct AsyncProcInputTx {
    inner: Mutex<mpsc::Sender<String>>,
}

fn main() {
    tracing_subscriber::fmt::init();

    let (async_proc_input_tx, async_proc_input_rx) = mpsc::channel(1);
    let (async_proc_output_tx, mut async_proc_output_rx) = mpsc::channel(1);

    tauri::Builder::default()
        .manage(AsyncProcInputTx {
            inner: Mutex::new(async_proc_input_tx),
        })
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

// A function that sends a message from Rust to JavaScript via a Tauri Event
fn rs2js<R: tauri::Runtime>(message: String, manager: &impl Manager<R>) {
    let mut sub_message = message.clone();
    sub_message.truncate(50);
    info!(?sub_message, "rs2js");
    manager.emit_all("rs2js", message).unwrap();
}

// The Tauri command that gets called when Tauri `invoke` JavaScript API is called
#[tauri::command]
async fn js2rs(message: String, state: tauri::State<'_, AsyncProcInputTx>) -> Result<(), String> {
    let mut sub_message = message.clone();
    sub_message.truncate(50);
    info!(?sub_message, "js2rs");
    let async_proc_input_tx = state.inner.lock().await;
    async_proc_input_tx.send(message).await.map_err(|e| {
        println!("{}", e.to_string());
        e.to_string()
    })
}

async fn async_process_model(
    mut input_rx: mpsc::Receiver<String>,
    output_tx: mpsc::Sender<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    while let Some(input) = input_rx.recv().await {
        let mut parse_error = false;
        let my_input_data: Receiver = match serde_json::from_str(input.as_str()) {
            Ok(data) => data,
            Err(err) => {
                parse_error = true;
                let my_output_data = Response {
                    dataname: "".to_string(),
                    data: "[]".to_string(),
                    error: err.to_string()
                };
                let output = json!(my_output_data).to_string();
                match output_tx.send(output).await {
                    _ => {}
                }
                Receiver {
                    path: "".to_string(),
                    query: "".to_string(),
                    data: "[]".to_string()
                }
            }
        };

        if !parse_error {
            let my_output_data: Response =
                message_handler(my_input_data.path, my_input_data.query, my_input_data.data).await;
            let output = json!(my_output_data).to_string();
            match output_tx.send(output).await {
                _ => {}
            }
        } 
    }

    Ok(())
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command(async)]
async fn message_handler(
    //window: tauri::Window,
    //database: tauri::State<'_, Database>,
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

            Response {
                dataname: "me".to_string(),
                data: "{\"email\":\"jankstar.berlin@gmail.com\",\"name\":\"jankstar\"}".to_string(),
                error: "".to_string(),
            }
        }
        //----
        "save_user" => Response {
            dataname: "me".to_string(),
            data: "{\"email\":\"jankstar.berlin@gmail.com\",\"name\":\"jankstar\"}".to_string(),
            error: "".to_string(),
        },
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
        "document" => document_message_handler(path, query).await,
        "pdf" => pdf_message_handler(path, query, data).await,
        _ => Response {
            dataname: path.clone(),
            data: String::from(""),
            error: format!("path {} not fund", path.as_str()),
        },
    }
}
