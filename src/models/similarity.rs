use diesel::prelude::*;

pub fn on_rating_update(conn: &mut SqliteConnection, user_id: i32, oeuvre_id: i32, old_rating: i32, new_rating: i32) -> diesel::result::QueryResult<()> {
    diesel::result::QueryResult::Ok(())
}

pub fn on_rating_add(conn: &mut SqliteConnection, user_id: i32, oeuvre_id: i32, rating: i32) -> diesel::result::QueryResult<()> {
    diesel::result::QueryResult::Ok(())
}

pub fn on_rating_remove(conn: &mut SqliteConnection, user_id: i32, oeuvre_id: i32) -> diesel::result::QueryResult<()> {
    diesel::result::QueryResult::Ok(())
}