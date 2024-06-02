use diesel::prelude::*;
use super::{ImdbMap, NewImdbMap, NewOeuvre, Oeuvre};

pub fn get_imdb_oeuvre_id(conn: &mut SqliteConnection, imdb_id: &str) -> diesel::result::QueryResult<i32> {
    use crate::schema::imdb_map;

    let imdb_oeuvre: ImdbMap = imdb_map::table.filter(imdb_map::imdb_id.eq(imdb_id)).first(conn)?;
    Ok(imdb_oeuvre.oeuvre_id)
}

pub fn add_imdb_oeuvre_no_check(conn: &mut SqliteConnection, new_oeuvre: NewOeuvre, imdb_id: &str) -> diesel::result::QueryResult<i32> {
    use crate::schema::{imdb_map, oeuvres};

    // NOTE: This doesn't check for duplicates, it should be used as a seeding method only !
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

    diesel::result::QueryResult::Ok(inserted_oeuvre.id)
}
