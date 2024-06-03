pub mod models;
pub mod api_models;
pub mod jwt_auth;
pub use models::*;
use dotenvy::dotenv;
use anyhow::Result;
use rusqlite::Connection;
use std::{env, fs};

pub fn setup_db() -> Result<()> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let _ = fs::remove_file(&database_url);
    let conn = Connection::open(database_url)?;
    conn.execute_batch(&fs::read_to_string("what_next_setup.sql")?)?;
    Ok(())
}

pub fn establish_connection() -> Result<Connection> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    Ok(Connection::open(database_url)?)
}
