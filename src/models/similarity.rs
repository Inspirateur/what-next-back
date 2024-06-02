use diesel::prelude::*;
use super::{Medium, Oeuvre};

pub fn on_rating_update(conn: &mut SqliteConnection, user_id: i32, oeuvre_id: i32, old_rating: i32, new_rating: i32) -> diesel::result::QueryResult<()> {
    diesel::result::QueryResult::Ok(())
}

pub fn on_rating_add(conn: &mut SqliteConnection, user_id: i32, oeuvre_id: i32, rating: i32) -> diesel::result::QueryResult<()> {
    diesel::result::QueryResult::Ok(())
}

pub fn on_rating_remove(conn: &mut SqliteConnection, user_id: i32, oeuvre_id: i32, old_rating: i32) -> diesel::result::QueryResult<()> {
    diesel::result::QueryResult::Ok(())
}

pub fn get_reco(conn: &mut SqliteConnection, user_id: i32, medium: Medium) -> diesel::result::QueryResult<Oeuvre> {
    use crate::schema::oeuvres::dsl::*;
    
    oeuvres.order(rating.desc()).first(conn)
}