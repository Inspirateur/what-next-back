pub mod models;
pub mod schema;

use diesel::prelude::*;
use dotenvy::dotenv;
use models::{ImdbMap, NewOeuvre, NewTag, Oeuvre, OeuvreTag, Tag};
use schema::imdb_map;
use std::env;

use crate::models::NewImdbMap;

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn add_imdb_oeuvre(conn: &mut SqliteConnection, new_oeuvre: NewOeuvre, imdb_id: &str) -> diesel::result::QueryResult<()> {
    todo!()
}

pub fn add_imdb_oeuvre_no_check(conn: &mut SqliteConnection, new_oeuvre: NewOeuvre, imdb_id: &str) -> diesel::result::QueryResult<()> {
    // NOTE: This doesn't check for duplicates, it should be used as a seeding method only !
    // use add_imdb_oeuvre if the database isn't empty
    use crate::schema::oeuvres;

    // check if the imdb_id hasn't already been inserted 
    let inserted_oeuvre: Oeuvre = diesel::insert_into(oeuvres::table)
        .values(&new_oeuvre)
        .returning(Oeuvre::as_returning())
        .get_result(conn)?;

    let imdb_map_entry = NewImdbMap {
        oeuvre_id: inserted_oeuvre.id,
        imdb_id
    };
    diesel::insert_into(imdb_map::table)
        .values(&imdb_map_entry)
        .execute(conn)?;

    diesel::result::QueryResult::Ok(())
}

pub fn add_imdb_tag(conn: &mut SqliteConnection, imdb_id: String, tag_label: String) -> diesel::result::QueryResult<()> {
    use crate::schema::*;

    let imdb_oeuvre: ImdbMap = imdb_map::table.filter(imdb_map::imdb_id.eq(imdb_id)).first(conn)?;
    
    let tag: Tag = if let Some(tag) = tags::table.filter(tags::label.eq(tag_label.clone())).first(conn).optional()? {
        tag
    } else {
        let new_tag = NewTag { label: tag_label };
        diesel::insert_into(tags::table)
            .values(&new_tag)
            .returning(Tag::as_returning())
            .get_result(conn)?
    };

    let tagged_oeuvre = OeuvreTag {
        oeuvre_id: imdb_oeuvre.oeuvre_id,
        tag_id: tag.id,
    };

    diesel::insert_into(oeuvre_tags::table)
        .values(&tagged_oeuvre)
        .execute(conn)?;

    diesel::result::QueryResult::Ok(())
}
