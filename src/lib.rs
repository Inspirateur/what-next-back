pub mod models;
pub mod schema;

use diesel::prelude::*;
use dotenvy::dotenv;
use models::{Oeuvre, NewOeuvre};
use std::env;


pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn create_oeuvre(conn: &mut SqliteConnection, new_oeuvre: NewOeuvre) -> usize {
    use crate::schema::oeuvres;

    diesel::insert_into(oeuvres::table)
        .values(&new_oeuvre)
        .execute(conn)
        .expect("Error saving new oeuvre")
}