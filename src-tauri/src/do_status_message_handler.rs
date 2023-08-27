#![allow(unused)]
#![allow(clippy::all)]

use crate::database::*;
use crate::models::*;
use crate::save_json::*;
use crate::schema::document::dsl;
use crate::schema::Response;

use chrono::{DateTime, Local, TimeZone};
use tauri::Manager;

use crate::diesel::sqlite::Sqlite;
use diesel::debug_query;
use diesel::prelude::*;
use serde_json::json;
use std::process::Command;
use tracing::{error, info};
use tracing_subscriber;

/// # do_status
/// `data`: Document
///
/// processing of the document for the status and setting the new status
pub async fn do_status(window: tauri::Window, mut data: Document) {
    info!("start async do_status");

    let my_app = window.app_handle();
    let app_data = my_app.state::<crate::AppData>();

    let mut l_change = false;

    //-----------------------------------------------------------//
    if data.status.clone().as_str() == "01_Leer" || data.status.clone().as_str() == "11_Rohdaten" {
        //status 01 or 11 - init or clear data
        data.status = "12_Rohdaten_bereinigt".to_string();
        data.sender_name = None;
        data.sender_addr = None;
        data.recipient_name = None;
        data.recipient_addr = None;
        data.from = None;
        data.to = None;
        data.body = None;
        data.metadata = None;
        data.category = None;
        data.amount = None;
        data.currency = None;
        data.template_name = None;
        data.doc_data = None;
        data.num_pages = None;
        data.base64 = None;
        data.ocr_data = None;
        data.jpg_file = None;

        data.protocol = format!("\n{} - init or clear data", Local::now());

        l_change = true;
    };

    //-----------------------------------------------------------//
    if data.status.clone().as_str() == "12_Rohdaten_bereinigt" {
        // status 12 -> 21
        let l_do = 'block: {
            data.status = "21_OCR".to_string();
            data.protocol
                .push_str(format!("\n{} - start OCR", Local::now()).as_str());

            crate::rs2js(
                json!(Response {
                    dataname: "info".to_string(),
                    data: json!("Status 21 - Generating OCR data").to_string(),
                    error: "".to_string()
                })
                .to_string(),
                &window,
            );

            if data.file.clone().unwrap_or("".to_string()).is_empty() {
                //no file found for OCR
                break 'block 1;
            };

            if data
                .file_extension
                .clone()
                .unwrap_or("".to_string())
                .is_empty()
            {
                let my_filename = data.file.clone().unwrap_or("".to_string());
                let extension_vec: Vec<&str> = my_filename.split(".").collect();
                if extension_vec.len() == 0 {
                    //no PDF
                    data.protocol
                        .push_str(format!("\n{} - no PDF file", Local::now()).as_str());
                    break 'block 2;
                }
                data.file_extension = Some(extension_vec[extension_vec.len() - 1].to_string());
            }

            if data
                .file_extension
                .clone()
                .unwrap_or("".to_string())
                .to_uppercase()
                != "PDF"
            {
                data.protocol
                    .push_str(format!("\n{} - no PDF file", Local::now()).as_str());
                break 'block 3;
            }

            let home_dir = home::home_dir().unwrap_or("".into());

            let l_path = format!(
                "{}/{}/{}/{}",
                home_dir.to_str().unwrap_or("").to_string(),
                MAIN_PATH,
                FILE_PATH,
                data.sub_path.clone().unwrap_or("".to_string())
            );

            let l_gosseract = format!(
                "{}/{}/{}/gosseract.ini",
                home_dir.to_str().unwrap_or("").to_string(),
                MAIN_PATH,
                FILE_PATH
            );

            let l_debug_txt = format!(
                "{}/{}/{}/debug.txt",
                home_dir.to_str().unwrap_or("").to_string(),
                MAIN_PATH,
                FILE_PATH
            );

            data.protocol
                .push_str(format!("\n{} - start gs", Local::now()).as_str());

            crate::rs2js(
                json!(Response {
                    dataname: "info".to_string(),
                    data: json!("gs is starting ...").to_string(),
                    error: "".to_string()
                })
                .to_string(),
                &window,
            );

            //build jpeg from PDF with gs
            let gs_output = if cfg!(target_os = "windows") {
                Command::new("cmd")
                    .args(["/C", "echo windows Not supported"])
                    .output()
                    .expect("failed to execute process")
            } else {
                let command_arg = format!("gs -dSAFER -dBATCH -dNOPAUSE -r1200 -sDEVICE=jpeg  -sOutputFile={}{}page%03d.jpg  {}{}",
                l_path.clone(),
                data.id.clone(),
                l_path.clone(),
                data.file.clone().unwrap_or("".to_string()));
                info!(command_arg, "gs command");

                Command::new("sh")
                    .arg("-c")
                    .arg(command_arg)
                    .output()
                    .expect("failed to execute process")
            };

            info!(?gs_output, "gs output");

            if !gs_output.status.success() {
                //#![no_std] !gs_output.stderr.is_empty() {
                error!("Error: {:#?}", String::from_utf8(gs_output.stderr.clone()));
                data.protocol.push_str(
                    format!(
                        "\n{} - gs Error {:#?}",
                        Local::now(),
                        String::from_utf8(gs_output.stderr)
                    )
                    .as_str(),
                );
                break 'block 5;
            }

            let mut l_filename_jpeg = l_path.clone();
            l_filename_jpeg.push_str(&data.id);

            let entrys = std::fs::read_dir(l_path.clone()).unwrap();
            let mut jpeg_data: Vec<String> = Vec::new();
            for entry in entrys {
                let entry_path = entry.unwrap().path();
                let l_path_str = entry_path.to_str().unwrap_or("").to_string();

                if !entry_path.is_dir()
                    && l_path_str.contains(&l_filename_jpeg)
                    && l_path_str.contains(".jpg")
                    && !l_path_str.contains(".jpg.txt")
                {
                    info!(?entry_path);

                    jpeg_data.push(l_path_str);
                }
            }

            if jpeg_data.len() == 0 {
                data.protocol
                    .push_str(format!("\n{} - no JPG files ", Local::now()).as_str());
                break 'block 6;
            }

            jpeg_data.sort();

            for jpeg_file in &jpeg_data {
                data.num_pages = Some((data.num_pages.unwrap_or(0.0) + 1.0));

                data.protocol.push_str(
                    format!(
                        "\n{} - start tesseract file {}",
                        Local::now(),
                        jpeg_file.clone().as_str()
                    )
                    .as_str(),
                );

                if data.num_pages.is_some_and(|x| x == 1.0) {
                    crate::rs2js(
                        json!(Response {
                            dataname: "info".to_string(),
                            data: json!("tesseract is starting ...").to_string(),
                            error: "".to_string()
                        })
                        .to_string(),
                        &window,
                    );
                }

                //build txt from jpg with tesseract
                let ts_output = if cfg!(target_os = "windows") {
                    Command::new("cmd")
                        .args(["/C", "echo windows Not supported"])
                        .output()
                        .expect("failed to execute process")
                } else {
                    let command_arg = format!(
                        "tesseract {} {} -l deu -c debug_file={} {}",
                        jpeg_file, jpeg_file, l_debug_txt, l_gosseract
                    );
                    info!(command_arg, "tesseract command");

                    Command::new("sh")
                        .arg("-c")
                        .arg(command_arg)
                        .output()
                        .expect("failed to execute process")
                };

                info!(?ts_output, "tesseract output");

                if !ts_output.status.success() {
                    //#![no_std] !ts_output.stderr.is_empty() {
                    error!(
                        "Error: {}",
                        String::from_utf8(ts_output.stderr.clone()).unwrap_or("".to_string())
                    );
                    data.protocol.push_str(
                        format!(
                            "\n{} - tesseract Error {:?}",
                            Local::now(),
                            String::from_utf8(ts_output.stderr)
                        )
                        .as_str(),
                    );
                    break 'block 5;
                }

                let mut txt_file = jpeg_file.clone();
                txt_file.push_str(".txt");

                let contents: String = std::fs::read_to_string(txt_file).unwrap_or("".to_string());
                let mut l_ocr = data.ocr_data.unwrap_or("".to_string());
                l_ocr.push_str(&contents);
                data.ocr_data = Some(l_ocr);
            }

            if data.num_pages.is_none() || data.num_pages.is_some_and(|x| x != 0.0) {
                data.jpg_file = Some(json!(&jpeg_data).to_string());
            }

            l_change = true;

            if data.body.clone().is_none() || data.body.clone().is_some_and(|x| x == "".to_string())
            {
                data.body = data.ocr_data.clone();
            }

            info!(?data.ocr_data);
            data.protocol
                .push_str(format!("\n{} - end OCR", Local::now()).as_str());

            //return 99 as end
            99
        };
    };

    //-----------------------------------------------------------//
    if data.status.clone().as_str() == "21_OCR" {
        // status 21 -> 21
        let l_do = 'block: {
            data.status = "31_Parse".to_string();
            data.protocol
                .push_str(format!("\n{} - start parse", Local::now()).as_str());

            crate::rs2js(
                json!(Response {
                    dataname: "info".to_string(),
                    data: json!("Status 31 - parsing data body").to_string(),
                    error: "".to_string()
                })
                .to_string(),
                &window,
            );

            l_change = true;
            99
        };

        data.protocol
        .push_str(format!("\n{} - end OCR", Local::now()).as_str());
    };

    if l_change == false {
        crate::rs2js(
            json!(Response {
                dataname: "info".to_string(),
                data: json!("status processed.").to_string(),
                error: "".to_string()
            })
            .to_string(),
            &window,
        );
        return;
    }

    let mut conn = app_data.db.lock().await;

    let exec_update = diesel::update(dsl::document.filter(dsl::id.eq(data.id.clone()))).set((
        &data,                                        //update AsChangeset
        dsl::updated_at.eq(Local::now().to_string()), //update datetime
    ));
    info!("debug sql\n{}", debug_query::<Sqlite, _>(&exec_update));

    match exec_update.execute(&mut *conn) {
        Ok(_) => {
            drop(conn);

            save_json_by_id(app_data, data.id.clone()).await;

            crate::rs2js(
                json!(Response {
                    dataname: "info".to_string(),
                    data: json!("status processed, all data saved").to_string(),
                    error: "".to_string()
                })
                .to_string(),
                &window,
            );
        }
        Err(err) => {
            error!(?err, "Error: ");
        }
    }
}

pub async fn do_status_message_handler(
    window: tauri::Window,
    app_data: tauri::State<'_, crate::AppData>,
    path: String,
    query: String,
    data: String,
) -> Response {
    info!("start do_status");

    let mut conn = app_data.db.lock().await;

    let exec_query = dsl::document
        .filter(dsl::id.eq(data.clone()))
        .select(Document::as_select());
    info!("debug sql\n{}", debug_query::<Sqlite, _>(&exec_query));

    let my_document = match exec_query.first::<Document>(&mut *conn) {
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
    drop(conn);

    tauri::async_runtime::spawn(async move {
        do_status(window, my_document).await;
    });

    Response {
        dataname: "info".into(),
        data: json!("Status processing started ... please wait.").to_string(),
        error: "".to_string(),
    }
}
