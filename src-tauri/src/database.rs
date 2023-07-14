//use std::env;

use diesel::prelude::*;
//use dotenvy::dotenv;
use tracing::info;
//use tracing_subscriber;
use home::home_dir;

pub const MAIN_PATH: &str = r#"archive_cat"#;
pub const DATABASE_NAME: &str = r#"archive_cat.db"#;
pub const FILE_PATH: &str = r#"data"#;
pub const APP_DATA_FILENAME: &str = r#".archive_cat"#;


// pub fn establish_connection() -> SqliteConnection {
//     info!("start establish_connection()",);

//     dotenv().ok();

//     let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
//     SqliteConnection::establish(&database_url)
//         .unwrap_or_else(|_| panic!("Error connecting to the database: {}", database_url))
// }

pub fn establish_connection(database_filename: &str) -> SqliteConnection {
    info!("start establish_connection({})",database_filename);

    let home_dir = home_dir().unwrap_or("".into()); 

    let database_url = format!("sqlite://{}/{}",home_dir.to_string_lossy(), database_filename);

    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to the database: {}", database_url))
}
