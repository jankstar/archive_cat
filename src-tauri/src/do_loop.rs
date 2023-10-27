#![allow(unused)]
#![allow(clippy::all)]

use crate::database::*;
use crate::models::*;
use crate::save_json::*;
use crate::schema::document;
use crate::schema::mail_data;
use crate::schema::Response;

use crate::dot_env::{GOOGLE_CLIENT_ID, GOOGLE_CLIENT_SECRET};

use tauri::{Manager, Window, WindowEvent};

use crate::diesel::sqlite::Sqlite;
use diesel::{debug_query, insert_into, prelude::*, sql_query};
use serde_json::json;
use tracing::{error, info};
use tracing_subscriber;

use uuid::Uuid;

use imap::{Authenticator, ClientBuilder};
use imap_proto::types::Address;

use mailparse::*;

use oauth2::{
    basic::BasicClient, reqwest::async_http_client, revocation::StandardRevocableToken,
    AccessToken, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    RedirectUrl, RevocationUrl, Scope, TokenResponse, TokenUrl,
};

use chrono::{
    format::format, format::ParseError, DateTime, Datelike, FixedOffset, Local, NaiveDate,
    NaiveDateTime, NaiveTime, TimeZone, Utc,
};

use dotenv::dotenv;
use regex::{Matches, Regex, RegexBuilder};
use std::borrow::Cow;
use std::env;
use std::net::TcpListener;
use std::process::Command;
use url::Url;

struct GmailOAuth2 {
    user: String,
    access_token: String,
}

impl imap::Authenticator for GmailOAuth2 {
    type Response = String;
    #[allow(unused_variables)]
    fn process(&self, data: &[u8]) -> Self::Response {
        format!(
            "user={}\x01auth=Bearer {}\x01\x01",
            self.user, self.access_token
        )
    }
}

use std::error::Error;

/// # get_token
/// Function to determine the access token for access to gmail
///
/// https://developers.google.com/identity/protocols/
async fn get_token(
    window: &tauri::Window,
    email: String,
    refresh_token: Option<oauth2::RefreshToken>,
) -> Result<(AccessToken, Option<oauth2::RefreshToken>), Box<dyn Error>> {
    //get the google client ID and the client secret from .env file
    dotenv().ok();

    //let google_client_id = ClientId::new(std::env::var("GOOGLE_CLIENT_ID")?);
    let google_client_id = ClientId::new(GOOGLE_CLIENT_ID.to_string());
    //let google_client_secret = ClientSecret::new(std::env::var("GOOGLE_CLIENT_SECRET")?);
    let google_client_secret = ClientSecret::new(GOOGLE_CLIENT_SECRET.to_string());
    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())?; //.expect("Invalid authorization endpoint URL");
    let token_url = TokenUrl::new("https://www.googleapis.com/oauth2/v3/token".to_string())?; //.expect("Invalid token endpoint URL");

    // Set up the config for the Google OAuth2 process.
    let client = BasicClient::new(
        google_client_id,
        Some(google_client_secret),
        auth_url,
        Some(token_url),
    )
    // This example will be running its own server at http://127.0.0.1:1421
    // See below for the server implementation.
    .set_redirect_uri(
        RedirectUrl::new("http://127.0.0.1:1421".to_string())?, //.expect("Invalid redirect URL"),
    )
    // Google supports OAuth 2.0 Token Revocation (RFC-7009)
    .set_revocation_uri(
        RevocationUrl::new("https://oauth2.googleapis.com/revoke".to_string())?, //.expect("Invalid revocation endpoint URL"),
    ); //.set_introspection_uri(introspection_url);

    if refresh_token.is_some() {
        println!("get_token() refresh_token found");

        match client
            .exchange_refresh_token(&refresh_token.unwrap().clone())
            .request_async(async_http_client)
            .await
        {
            Ok(token_response) => {
                let access_token = token_response.access_token().clone();
                let refresh_token = token_response.refresh_token().cloned();
                return Ok((access_token, refresh_token));
            }
            Err(_) => {}
        };
        println!("get_token() refresh_token not valid, login required");
    }

    // Google supports Proof Key for Code Exchange (PKCE - https://oauth.net/2/pkce/).
    // Create a PKCE code verifier and SHA-256 encode it as a code challenge.
    let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate the authorization URL to which we'll redirect the user.
    let (authorize_url, csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        // This example is requesting access to the "gmail" features and the user's profile.
        .add_scope(Scope::new("https://mail.google.com".into()))
        .add_scope(Scope::new("profile email".into()))
        .add_extra_param("access_type", "offline")
        .add_extra_param("login_hint", email)
        //.add_extra_param("prompt", "none")
        .set_pkce_challenge(pkce_code_challenge)
        .url();

    println!("The authorization URL is:\n{}\n", authorize_url.to_string());

    let handle = window.app_handle();

    let login_window = tauri::WindowBuilder::new(
        &handle,
        "Google_Login", /* the unique window label */
        tauri::WindowUrl::External(
            authorize_url.to_string().parse()?, //.expect("error WindowBuilder WindowUrl parse"),
        ),
    )
    .build()?; //.expect("error WindowBuilder build");
    login_window.set_title("Google Login");
    login_window.set_always_on_top(true);

    // A very naive implementation of the redirect server.
    let listener = std::net::TcpListener::bind("127.0.0.1:1421")?; //.expect("error TcpListener bind");
    let local_addr = listener.local_addr()?;

    let timer = timer::Timer::new();

    let _guard = timer.schedule_with_delay(chrono::Duration::seconds(25), move || {
        //the time out as connect to close server
        let _ = std::net::TcpStream::connect(local_addr);
    });

    login_window.on_window_event(move |event| {
        if let WindowEvent::CloseRequested { api, .. } = &event {
            info!("event close-requested");
            let _ = std::net::TcpStream::connect(local_addr); //connect to server to close it
        };
    });

    //this is blocking listener! we use guard schedule for time out
    for stream in listener.incoming() {
        let _ = login_window.is_visible()?; //check if login_window is visible

        if let Ok(mut stream) = stream {
            info!("listener stream");

            let code;
            let state;
            let errorinfo;
            {
                use std::io::{BufRead, BufReader};
                let mut reader = BufReader::new(&stream);

                let mut request_line = String::new();
                reader.read_line(&mut request_line)?;

                let redirect_url = match request_line.split_whitespace().nth(1) {
                    Some(url_data) => url_data,
                    _ => {
                        login_window.close()?;
                        break;
                    }
                };
                println!("redirect_url: \n{}", redirect_url.clone());
                let url = url::Url::parse(&("http://localhost".to_string() + redirect_url))?;

                use std::borrow::Cow;
                //extract code from url
                let code_pair = url
                    .query_pairs()
                    .find(|pair| {
                        let &(ref key, _) = pair;
                        key == "code"
                    })
                    .unwrap_or((Cow::from(""), Cow::from("")));

                let (_, value) = code_pair;
                code = AuthorizationCode::new(value.into_owned());

                //extract state from url
                let state_pair = url
                    .query_pairs()
                    .find(|pair| {
                        let &(ref key, _) = pair;
                        key == "state"
                    })
                    .unwrap_or((Cow::from(""), Cow::from("")));

                let (_, value) = state_pair;
                state = CsrfToken::new(value.into_owned());

                //extract error from url
                let errorinfo_pair = url
                    .query_pairs()
                    .find(|pair| {
                        let &(ref key, _) = pair;
                        key == "error"
                    })
                    .unwrap_or((Cow::from(""), Cow::from("")));

                let (_, value) = errorinfo_pair;
                errorinfo = String::from(value.into_owned());
            }

            //if error found
            if !errorinfo.is_empty() {

                crate::rs2js(
                    json!(Response {
                        data: json!(format!("error (234) - Access token could not be retrieved {}", errorinfo)).to_string(),
                        dataname: "info".to_string(),
                        error: "".to_string()
                    })
                    .to_string(),
                    window,
                );

                login_window.close()?;

                Err(errorinfo)?
            }

            let message = "Verification completed, please close window.";
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n{}",
                message.len(),
                message
            );
            {
                use std::io::Write; // bring trait into scope
                stream.write_all(response.as_bytes())?;
            }
            println!("Google returned the following code:\n{}\n", code.secret());
            println!(
                "Google returned the following state:\n{} (expected `{}`)\n",
                state.secret(),
                csrf_state.secret()
            );

            // Exchange the code with a token.
            let token_response = match client
                .exchange_code(code)
                .set_pkce_verifier(pkce_code_verifier)
                .request_async(async_http_client)
                .await
            {
                Ok(res) => res,
                Err(err) => {

                    crate::rs2js(
                        json!(Response {
                            data: json!(format!("error - no permission ")).to_string(),
                            dataname: "info".to_string(),
                            error: "".to_string()
                        })
                        .to_string(),
                        window,
                    );

                    login_window.close()?;
                    Err("--  no permission --")?
                }
            };

            println!("\n{:#?}", token_response);

            // println!(
            //     "\naccess-token:\n{:#?}\ntoken_type:\n{:#?}\
            //     \nexpires_in\n{:#?}\nrefresh_token\n{:#?}\
            //     \nscopes\n{:#?}\nextra_fields\n{:#?}",
            //     token_response.access_token().clone(),
            //     token_response.token_type().clone(),
            //     token_response.expires_in().clone(),
            //     token_response.refresh_token().clone(),
            //     token_response.scopes().clone(),
            //     token_response.extra_fields().clone()
            // );

            let access_token = token_response.access_token().clone();
            let refresh_token = token_response.refresh_token().cloned();

            println!("Google returned the following token:\n{:?}\n", access_token);

            // // Revoke the obtained token
            // let token_response = token_response.unwrap();
            // let token_to_revoke: StandardRevocableToken = match token_response.refresh_token() {
            //     Some(token) => token.into(),
            //     None => token_response.access_token().into(),
            // };

            // client
            //     .revoke_token(token_to_revoke)
            //     .unwrap()
            //     .request_async(async_http_client).await
            //     //.request(http_client)
            //     .expect("Failed to revoke token");

            login_window.close()?; //.expect("error closw login window");

            return Ok((access_token, refresh_token));
            // The server will terminate itself after revoking the token.
            break;
        } else {

            crate::rs2js(
                json!(Response {
                    data: json!(format!("error - Access token could not be retrieved " )).to_string(),
                    dataname: "info".to_string(),
                    error: "".to_string()
                })
                .to_string(),
                window,
            );

            println!("error on stream");
            break;
        }
    } //listener.incoming() loop

    Err("-- login window time out --")?

    //return "".to_string(); //token_result.access_token().clone();
}

fn ut8_str(i_u8: &[u8]) -> String {
    match std::str::from_utf8(i_u8) {
        Ok(data) => data.to_string(),
        Err(_) => "".to_string(),
    }
}

fn vec_str(i_vec: &Vec<Address>) -> String {
    let mut e_str: String = "[".to_string();
    let mut count = 0;
    for a in i_vec {
        count += 1;
        if a.name.is_none() || a.mailbox.is_none() || a.host.is_none() {
            break;
        }
        if count > 1 {
            e_str.push_str(",");
        }
        e_str.push_str("{");

        e_str.push_str(&format!(
            "\"name\":\"{}\", \"email\": \"{}@{}\"",
            &ut8_str(&a.name.as_ref().unwrap()),
            &ut8_str(&a.mailbox.as_ref().unwrap()),
            &ut8_str(&a.host.as_ref().unwrap()).to_string()
        ));

        e_str.push_str("}");
    }
    e_str.push_str("]");

    return e_str;
}

fn processed_text(
    i_doc_body_old: Option<String>,
    i_part: &mailparse::ParsedMail,
) -> Option<String> {
    if i_part.ctype.mimetype.clone().contains("text/") {
        println!("mime-type processed: {}", &i_part.ctype.mimetype);

        let mut l_doc_body = i_doc_body_old.unwrap_or("".to_string());
        l_doc_body.push_str(&i_part.get_body().unwrap_or("".to_string()).as_str());
        Some(l_doc_body)
    } else {
        println!(
            "mime-type NOT processed as TEXT: {}",
            &i_part.ctype.mimetype
        );
        i_doc_body_old
    }
}

/// # processed_attachment
/// the mail part is examined for attachments
/// i.e. attachments with pdf files are searched for and these are then
/// in the `i_sub_path` directory under a uuid.
/// The function returns the name and created file.
fn processed_attachment(
    i_sub_path: &str,
    i_part: &mailparse::ParsedMail,
) -> (Option<String>, Option<String>) {
    let mut l_header_field = "".to_string();
    for head_elemenet in &i_part.headers {
        l_header_field = head_elemenet.get_value();

        if l_header_field.contains("attachment;") && l_header_field.contains("filename=") {
            break;
        }
        l_header_field = "".to_string();
    }
    if l_header_field.is_empty() {
        (None, None)
    } else {
        match l_header_field.find("filename=") {
            Some(pos) => {
                let l_filename = l_header_field[pos + 9..].to_string();
                let e_filename = l_filename.replace("\"", "");
                if e_filename.to_lowercase().contains(".pdf") {
                    //save pdf as file
                    println!("attachment found filename is {}", e_filename);
                    let l_file = format!("{}.pdf", Uuid::new_v4().to_string());

                    let home_dir = home::home_dir().unwrap_or("".into());

                    //Build PDF Filenames
                    let pdf_file_to = format!(
                        "{}/{}/{}/{}{}",
                        home_dir.to_str().unwrap_or("").to_string(),
                        MAIN_PATH,
                        FILE_PATH,
                        i_sub_path,
                        l_file
                    );
                    info!(?pdf_file_to, "new document file");

                    use mailparse::body::Body;
                    use std::fs;
                    {
                        use std::io::Write; // bring trait into scope

                        let mut file = match fs::OpenOptions::new()
                            .create(true)
                            .write(true)
                            .open(pdf_file_to)
                        {
                            Ok(i_file) => i_file,
                            Err(_) => {
                                error!("Error file create");
                                return (Some(e_filename), None);
                            }
                        };

                        match i_part.get_body_encoded() {
                            Body::Base64(body) | Body::QuotedPrintable(body) => {
                                file.write_all(&body.get_decoded().unwrap());

                                (Some(e_filename), Some(l_file))
                            }
                            Body::SevenBit(body) | Body::EightBit(body) => {
                                file.write_all(&body.get_as_string().unwrap().as_bytes());

                                (Some(e_filename), Some(l_file))
                            }
                            Body::Binary(body) => {
                                file.write_all(&body.get_raw());

                                (Some(e_filename), Some(l_file))
                            }
                            _ => {
                                error!("Error body encoded");
                                (Some(e_filename), None)
                            }
                        }
                    }
                } else {
                    (Some(e_filename), None)
                }
            }
            _ => (None, None),
        }
    }
}

use tokio::sync::Mutex;

/// # do_loop
/// This function performs an Oauth2 authentication for a google email.
/// Afterwards, the email account is accessed with the access token and
/// all unread emails are downloaded from the INBOX and processed as a new document.
pub async fn do_loop(window: tauri::Window) {
    let my_app = window.app_handle();
    let app_data = my_app.state::<crate::AppData>();

    let mut main_data = app_data.main_data.lock().await;

    let l_do_email: i32 = 'email: {
        if main_data.email.is_empty() {
            break 'email 1;
        }

        crate::rs2js(
            json!(Response {
                data: json!(format!("email {}", main_data.email.clone())).to_string(),
                dataname: "info".to_string(),
                error: "".to_string()
            })
            .to_string(),
            &window,
        );

        info!("do_loop() email: {:?}", main_data.email);

        let (l_access_token, l_refresh_token) = match get_token(
            &window,
            main_data.email.clone(),
            main_data.refresh_token.clone(),
        )
        .await
        {
            Ok(token) => token,
            Err(e) => {
                
                crate::rs2js(
                    json!(Response {
                        data: json!(format!("error (534) - Access token could not be retrieved {}", e)).to_string(),
                        dataname: "info".to_string(),
                        error: "".to_string()
                    })
                    .to_string(),
                    &window,
                );

                error!("error - Access token could not be retrieved {}", e);
                break 'email 2;
            }
        };

        if l_refresh_token.is_some() {
            println!(
                "do_loop() refresh_token {:?} found",
                l_refresh_token.clone()
            );

            main_data.set_token(l_refresh_token);
        } else {
            println!("do_loop() no refresh_token found");
        }

        let gmail_auth = GmailOAuth2 {
            user: main_data.email.clone(),
            access_token: String::from(
                l_access_token.secret(), //element.token_access_token.unwrap_or("".into()).as_str(),
            ),
        };

        let client = match imap::ClientBuilder::new("imap.gmail.com", 993).native_tls() {
            Ok(c) => c,
            Err(e) => {
                error!("error - Could not connect to imap.gmail.com: {}", e);
                break 'email 3;
            }
        };

        let mut imap_session = match client.authenticate("XOAUTH2", &gmail_auth) {
            Ok(c) => c,
            Err((e, _unauth_client)) => {
                error!("error authenticating: {}", e);
                break 'email 4;
            }
        };

        let l_do = 'mbox: {
            //login is valide

            crate::rs2js(
                json!(Response {
                    data: json!("Email INBOX reading ... .").to_string(),
                    dataname: "info".to_string(),
                    error: "".to_string()
                })
                .to_string(),
                &window,
            );

            let l_mailbox = match imap_session.select("INBOX") {
                Ok(mailbox) => mailbox,
                Err(e) => {
                    error!("Error selecting INBOX: {}", e);
                    break 'mbox 1;
                }
            };

            println!("INBOX:\n{:?}", l_mailbox);

            let l_search = match imap_session.search("NOT SEEN") {
                Ok(result) => result,
                Err(e) => {
                    info!("no unseen message: {}", e);
                    break 'mbox 2;
                }
            };

            //println!("search:\n{:?}", l_search);

            for id in l_search {
                match imap_session.fetch(id.clone().to_string(), "(FLAGS ENVELOPE RFC822)") {
                    Ok(msgs) => {
                        if let Some(message) = msgs.iter().next() {
                            let mut flags = "".to_string();
                            for flag in message.flags().iter() {
                                flags.push_str(format!("{:?}, ", flag).as_str())
                            }

                            let envelope = message.envelope().expect("error: envelope");

                            let mut l_document = Document::new();

                            let my_sub_path = l_document.sub_path.clone().unwrap_or("".to_string());
                            let mut vec_filename: Vec<(Option<String>, Option<String>)> =
                                Vec::new();
                            l_document.subject = ut8_str(
                                &envelope.subject.clone().unwrap_or(Cow::from("".as_bytes())),
                            );

                            let my_message = format!(
                                "Email 'Subject' {} processed.",
                                l_document.subject.clone()
                            );
                            crate::rs2js(
                                json!(Response {
                                    data: json!(my_message).to_string(),
                                    dataname: "info".to_string(),
                                    error: "".to_string()
                                })
                                .to_string(),
                                &window,
                            );

                            if envelope.date.is_some() {
                                l_document.date = format!(
                                    "{}",
                                    DateTime::parse_from_rfc2822(&ut8_str(
                                        &envelope.date.clone().unwrap()
                                    ))
                                    .unwrap()
                                );
                            }
                            if envelope.from.is_some() {
                                l_document.from = Some(vec_str(&envelope.from.as_ref().unwrap()));
                            }
                            if envelope.to.is_some() {
                                l_document.to = Some(vec_str(&envelope.to.as_ref().unwrap()));
                            }

                            // extract the message's body
                            let body = message.body().unwrap_or(&[]);

                            let parsed = parse_mail(body).unwrap();

                            println!("********************");
                            for l_value in vec!["Date", "Subject", "From", "To"] {
                                println!(
                                    "{}: {}",
                                    l_value,
                                    parsed
                                        .headers
                                        .get_first_value(l_value)
                                        .unwrap_or("".to_string())
                                );
                            }
                            l_document.body = processed_text(l_document.body.clone(), &parsed);

                            println!("Subparts: {}", parsed.subparts.len());

                            let mut part_nr = 0;
                            for part in &parsed.subparts {
                                let mut count = 0;
                                for haeder_part in &part.headers {
                                    println!("{}/ {}: {}", part_nr, count, haeder_part.get_value());
                                    count += 1;
                                }
                                println!("{}/ mime-type: {}", part_nr, part.ctype.mimetype);
                                l_document.body = processed_text(l_document.body.clone(), part);
                                let (l_filename, l_file) = processed_attachment(&my_sub_path, part);
                                if l_file.is_some() {
                                    //if file saved then push to vec
                                    vec_filename.push((l_filename.clone(), l_file.clone()));
                                }

                                let mut part_part_nr = 0;
                                for part_part in &part.subparts {
                                    let mut count = 0;
                                    for haeder_part in &part_part.headers {
                                        println!(
                                            "{}/{}/ {}: {}",
                                            part_nr,
                                            part_part_nr,
                                            count,
                                            haeder_part.get_value()
                                        );
                                        count += 1;
                                    }
                                    println!(
                                        "{}/{}/ mime-type: {}",
                                        part_nr, part_part_nr, &part_part.ctype.mimetype
                                    );
                                    l_document.body =
                                        processed_text(l_document.body.clone(), part_part);
                                    let (l_filename, l_file) =
                                        processed_attachment(&my_sub_path, part_part);
                                    if l_file.is_some() {
                                        //if file saved then push to vec
                                        vec_filename.push((l_filename.clone(), l_file.clone()));
                                    }

                                    part_part_nr += 1;
                                }
                                part_nr += 1;
                            }

                            println!("********************");
                            if vec_filename.len() == 0 {
                                //keine attachments
                                vec_filename.push((None, None));
                            }

                            let mut count = 0;
                            for (l_filename, l_file) in vec_filename {
                                count += 1;

                                if count == 2 {
                                    //ab dem 2ten Dokument gibt es ein parent!
                                    l_document.parent_document = Some(l_document.id.clone());
                                }
                                if count != 1 {
                                    //mehrere Dokuemnte -> eigene UUID
                                    l_document.id = Uuid::new_v4().to_string();
                                }

                                l_document.filename = l_filename;
                                l_document.file = l_file;
                                if l_document.file.is_some() {
                                    l_document.file_extension = Some("PDF".to_string());
                                }

                                l_document.owner = main_data.email.clone().to_lowercase();

                                let new_document_id = l_document.id.clone();
                                let mut conn = app_data.db.lock().await;

                                match insert_into(document::dsl::document)
                                    .values(&l_document)
                                    .execute(&mut *conn)
                                {
                                    Ok(_) => {
                                        drop(conn);
                                        //drop(main_data);

                                        save_json_by_doc(&l_document).await;
                                    }
                                    Err(e) => {
                                        drop(conn);

                                        error!("error DB insert: {}", e);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error Fetching email: {}", e);
                        break;
                    }
                };
                //break; //only the first
            }

            99
        };

        match imap_session.logout() {
            Ok(_) => {}
            Err(e) => println!("Error logout: {}", e),
        }

        99 //return 99 the end
    };

    /** file scan */
    let l_do_scan: i32 = 'scan: {
        if main_data.scan_path.is_empty() || main_data.scan_filter.is_empty() {
            break 'scan 1;
        }

        crate::rs2js(
            json!(Response {
                data: json!(format!("file scan {}", main_data.scan_path.clone())).to_string(),
                dataname: "info".to_string(),
                error: "".to_string()
            })
            .to_string(),
            &window,
        );

        info!("do_loop() scan: {:?}", main_data.scan_path);
        info!("do_loop() filter: {:?}", main_data.scan_filter);

        let mut l_filter = main_data.scan_filter.clone();

        l_filter = l_filter.replace("?", "."); //beliebiges Zeichen
        l_filter = l_filter.replace("+", "."); //beliebiges Zeichen
        l_filter = l_filter.replace("*", "[[:alnum:]]*"); //mehrere beliebige Zeichen

        let mut re_filter = main_data.scan_path.clone();
        re_filter.push_str("/");
        re_filter.push_str(&l_filter);

        re_filter = re_filter.replace("/", "\\/");
        re_filter = re_filter.replace(".", "\\."); //echter Punkt

        let re = Regex::new(&re_filter).unwrap();
        info!(?re_filter);

        let entrys = std::fs::read_dir(main_data.scan_path.clone()).unwrap();
        let mut pdf_data: Vec<String> = Vec::new();
        for entry in entrys {
            let entry_path = entry.unwrap().path();
            let l_path_str = entry_path.to_str().unwrap_or("").to_string();

            if !entry_path.is_dir()
                && !l_path_str.is_empty()
                && re.is_match(&l_path_str)
                && l_path_str.to_uppercase().contains(".PDF")
            {
                info!(?l_path_str);

                pdf_data.push(l_path_str);
            };
        }
        if pdf_data.len() == 0 {
            info!("no file found");
            break 'scan 2;
        }

        pdf_data.sort();
        for pdf_file in &pdf_data {
            //read PDF file
            let mut data_vec = Vec::new();
            let chunk_size = 0x4000;

            {
                use std::io::{self, Read};
                let mut file = match std::fs::File::open(&pdf_file) {
                    Ok(file) => file,
                    Err(err) => {
                        error!("error read file {:?}", err);
                        continue;
                    }
                };

                loop {
                    let mut chunk = Vec::with_capacity(chunk_size);
                    let n = match file
                        .by_ref()
                        .take(chunk_size as u64)
                        .read_to_end(&mut chunk)
                    {
                        Ok(data) => data,
                        Err(err) => {
                            error!("error read file {}", err);
                            break;
                        }
                    };
                    if n == 0 {
                        break;
                    }
                    for l_char in chunk {
                        data_vec.push(l_char);
                    }
                    //data_vec.push(chunk.as_slice());
                    if n < chunk_size {
                        break;
                    }
                }

                if data_vec.len() == 0 {
                    //error reading file
                    error!("error read file {}", pdf_file);

                    continue;
                };
            }
            let mut new_document = Document::new();

            let mut extension_vec: Vec<&str> = pdf_file.split(".").collect();
            if extension_vec.len() == 0 {
                error!("no valide extension {}", pdf_file.clone());
                continue;
            }

            new_document.file_extension = Some(extension_vec[extension_vec.len() - 1].to_string());

            new_document.subject = pdf_file.clone();
            new_document.input_path = Some("01_upload".to_string());
            new_document.filename = Some(pdf_file.clone());
            new_document.owner = main_data.email.clone().to_lowercase();

            new_document.file = Some(format!(
                "{}.{}",
                new_document.id.clone(),
                new_document
                    .file_extension
                    .clone()
                    .unwrap_or("".to_string())
            ));

            let home_dir = home::home_dir().unwrap_or("".into());

            //Build PDF Filenames
            let pdf_file_to = format!(
                "{}/{}/{}/{}{}",
                home_dir.to_str().unwrap_or("").to_string(),
                MAIN_PATH,
                FILE_PATH,
                new_document.sub_path.clone().unwrap_or("".to_string()),
                new_document.file.clone().unwrap_or("".to_string())
            );
            info!(?pdf_file_to, "new document file");

            {
                use std::fs;
                use std::io::Write; // bring trait into scope
                let mut file = match fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .open(pdf_file_to.clone())
                {
                    Ok(i_file) => i_file,
                    Err(err) => {
                        error!("error write file {}: {}", pdf_file_to.clone(), err);
                        continue;
                    }
                };

                match file.write_all(&data_vec) {
                    Ok(_) => {}
                    Err(err) => {
                        error!("error write file {}: {}", pdf_file_to.clone(), err);
                        continue;
                    }
                };
            }

            let new_document_id = new_document.id.clone();
            let mut conn = app_data.db.lock().await;

            match insert_into(document::dsl::document)
                .values(&new_document)
                .execute(&mut *conn)
            {
                Ok(_) => {
                    drop(conn);

                    save_json_by_doc(&new_document).await;
                }
                Err(err) => {
                    drop(conn);
                    error!(?err);

                    continue;
                }
            };

            /** rename file Scan directory */
            let rename_file = format!(
                "{}/{}",
                main_data.scan_path.clone(),
                new_document.file.clone().unwrap_or("".to_string())
            );
            match std::fs::rename(&pdf_file, &rename_file) {
                Ok(_) => {
                    info!("file rename to {}", pdf_file_to)
                }
                Err(err) => {
                    error!("error write file {}: {}", pdf_file_to.clone(), err);
                    continue;
                }
            };
        }

        99
    };

    crate::rs2js(
        json!(Response {
            data: json!("Loop ends.").to_string(),
            dataname: "info".to_string(),
            error: "".to_string()
        })
        .to_string(),
        &window,
    );
}
