#![allow(unused)]
#![allow(clippy::all)]

use crate::database::*;
use crate::models::*;
use crate::save_json::*;
use crate::schema::document;
use crate::schema::mail_data;
use crate::schema::Response;

use chrono::format::format;
use tauri::Manager;
use tauri::Window;

use crate::diesel::sqlite::Sqlite;
use diesel::{debug_query, insert_into, prelude::*, sql_query};
use serde_json::json;
use tracing::{error, info};
use tracing_subscriber;

use uuid::Uuid;

use imap::{Authenticator, ClientBuilder};
use imap_proto::types::Address;

use mailparse::*;

use oauth2::reqwest::async_http_client;
use oauth2::{basic::BasicClient, revocation::StandardRevocableToken, TokenResponse};

use chrono::format::ParseError;
use chrono::{DateTime, FixedOffset, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use oauth2::{
    AccessToken, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    RedirectUrl, RevocationUrl, Scope, TokenUrl,
};

use dotenv::dotenv;
use std::borrow::Cow;
use std::env;
use std::io::{BufRead, BufReader, Write};
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
    dotenv().ok();

    let google_client_id = ClientId::new(env::var("GOOGLE_CLIENT_ID")?);
    let google_client_secret = ClientSecret::new(env::var("GOOGLE_CLIENT_SECRET")?);
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
        let token_response = client
            .exchange_refresh_token(&refresh_token.unwrap().clone())
            .request_async(async_http_client)
            .await?;
        let access_token = token_response.access_token().clone();
        let refresh_token = token_response.refresh_token().cloned();
        return Ok((access_token, refresh_token));
    }

    // Google supports Proof Key for Code Exchange (PKCE - https://oauth.net/2/pkce/).
    // Create a PKCE code verifier and SHA-256 encode it as a code challenge.
    let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate the authorization URL to which we'll redirect the user.
    let (authorize_url, csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        // This example is requesting access to the "gmail" features and the user's profile.
        .add_scope(Scope::new("https://mail.google.com".into()))
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

    // A very naive implementation of the redirect server.
    let listener = TcpListener::bind("127.0.0.1:1421")?; //.expect("error TcpListener bind");
    for stream in listener.incoming() {
        println!("stream");

        if let Ok(mut stream) = stream {
            let code;
            let state;
            {
                let mut reader = BufReader::new(&stream);

                let mut request_line = String::new();
                reader.read_line(&mut request_line)?; //.unwrap();

                let redirect_url = request_line.split_whitespace().nth(1).unwrap();
                println!("redirect_url: \n{}", redirect_url.clone());
                let url = Url::parse(&("http://localhost".to_string() + redirect_url))?; //.unwrap();

                use std::borrow::Cow;
                let code_pair = url
                    .query_pairs()
                    .find(|pair| {
                        let &(ref key, _) = pair;
                        key == "code"
                    })
                    .unwrap_or((Cow::from(""), Cow::from("")));

                let (_, value) = code_pair;
                code = AuthorizationCode::new(value.into_owned());

                let state_pair = url
                    .query_pairs()
                    .find(|pair| {
                        let &(ref key, _) = pair;
                        key == "state"
                    })
                    .unwrap_or((Cow::from(""), Cow::from("")));

                let (_, value) = state_pair;
                state = CsrfToken::new(value.into_owned());
            }

            let message = "Verification completed, please close window.";
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n{}",
                message.len(),
                message
            );
            stream.write_all(response.as_bytes())?;

            println!("Google returned the following code:\n{}\n", code.secret());
            println!(
                "Google returned the following state:\n{} (expected `{}`)\n",
                state.secret(),
                csrf_state.secret()
            );

            // Exchange the code with a token.
            let token_response = client
                .exchange_code(code)
                .set_pkce_verifier(pkce_code_verifier)
                .request_async(async_http_client)
                .await?; //.expect("token response error");

            println!(
                "\naccess-token:\n{:#?}\ntoken_type:\n{:#?}\
                \nexpires_in\n{:#?}\nrefresh_token\n{:#?}\
                \nscopes\n{:#?}\nextra_fields\n{:#?}",
                token_response.access_token().clone(),
                token_response.token_type().clone(),
                token_response.expires_in().clone(),
                token_response.refresh_token().clone(),
                token_response.scopes().clone(),
                token_response.extra_fields().clone()
            );

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
        }
    }
    Err("-- the login window is still waiting --")?

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

fn processed_attachment(
    i_sub_path: String,
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
                if e_filename.contains(".pdf") {
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
                    use std::io::Write; // bring trait into scope
                    match i_part.get_body_encoded() {
                        Body::Base64(body) | Body::QuotedPrintable(body) => {
                            let mut file = fs::OpenOptions::new()
                                .create(true)
                                .write(true)
                                .open(pdf_file_to)
                                .unwrap();

                            file.write_all(&body.get_decoded().unwrap());

                            (Some(e_filename), Some(l_file))
                        }
                        Body::SevenBit(body) | Body::EightBit(body) => {
                            let mut file = fs::OpenOptions::new()
                                .create(true)
                                .write(true)
                                .open(pdf_file_to)
                                .unwrap();

                            file.write_all(&body.get_as_string().unwrap().as_bytes());

                            (Some(e_filename), Some(l_file))
                        }
                        Body::Binary(body) => {
                            let mut file = fs::OpenOptions::new()
                                .create(true)
                                .write(true)
                                .open(pdf_file_to)
                                .unwrap();

                            file.write_all(&body.get_raw());

                            (Some(e_filename), Some(l_file))
                        }
                        _ => {
                            error!("Error body parse");
                            (Some(e_filename), None)
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

pub async fn do_loop(window: tauri::Window, app_data: tauri::State<'_, crate::AppData>,) {
    info!("start async do_loop");

    let mut main_data = app_data.main_data.lock().await;


    let l_do: i32 = 'block: {
        if main_data.email.is_empty() {
            break 'block 1;
        }
        info!("{:?}",main_data);


        let (l_access_token, l_refresh_token) = match get_token(
            &window,
            main_data.email.clone(),
            main_data.refresh_token.clone(),
        )
        .await
        {
            Ok(token) => token,
            Err(e) => {
                error!("error - Access token could not be retrieved {}", e);
                break 'block 2;
            }
        };

        if l_refresh_token.is_some() {
            println!("do_loop() refresh_token {:?} found", l_refresh_token.clone());
        } else {
            println!("do_loop() no refresh_token found");
        }

        main_data.set_token(l_refresh_token);

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
                break 'block 3;
            }
        };

        let mut imap_session = match client.authenticate("XOAUTH2", &gmail_auth) {
            Ok(c) => c,
            Err((e, _unauth_client)) => {
                error!("error authenticating: {}", e);
                break 'block 4;
            }
        };

        let l_do = 'mbox: {
            //login is valide

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
                            let mut vec_filename: Vec<(Option<String>, Option<String>)> =
                                Vec::new();
                            l_document.subject = ut8_str(
                                &envelope.subject.clone().unwrap_or(Cow::from("".as_bytes())),
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
                                let (l_filename, l_file) = processed_attachment(
                                    l_document.sub_path.clone().unwrap_or("".to_string()),
                                    part,
                                );
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
                                    let (l_filename, l_file) = processed_attachment(
                                        l_document.sub_path.clone().unwrap_or("".to_string()),
                                        part_part,
                                    );
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

                                let new_document_id = l_document.id.clone();

                                let database_name = format!("{}/{}", MAIN_PATH, DATABASE_NAME);
                                let mut conn = establish_connection(&database_name);
                                match insert_into(document::dsl::document)
                                    .values(&l_document)
                                    .execute(&mut conn)
                                {
                                    Ok(_) => {
                                        save_json(new_document_id).await;
                                    }
                                    Err(e) => {
                                        error!("error DB insert: {}", e);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!("Error Fetching email: {}", e);
                        break;
                    }
                };
                //break; //nur den ersten
            }

            99
        };

        match imap_session.logout() {
            Ok(_) => {}
            Err(e) => println!("Error logout: {}", e),
        }

        99 //return 99 the end
    };
}

