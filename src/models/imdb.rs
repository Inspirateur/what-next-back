use rusqlite::params;
use rusqlite::Result;
use rusqlite::Connection;
use crate::add_oeuvre;
use crate::update_oeuvre;
use super::NewOeuvre;

pub fn get_imdb_id(conn: &Connection, oeuvre_id: i32) -> Result<String> {
    conn.prepare_cached("SELECT imdb_id FROM imdb_map WHERE oeuvre_id = ?1")?
        .query_row([oeuvre_id], |row| row.get::<usize, String>(0))
}

pub fn get_imdb_oeuvre_id(conn: &Connection, imdb_id: &str) -> Result<i32> {
    conn.prepare_cached("SELECT oeuvre_id FROM imdb_map WHERE imdb_id = ?1")?
        .query_row([imdb_id], |row| row.get::<usize, i32>(0))
}

pub fn add_imdb_oeuvre(conn: &Connection, new_oeuvre: NewOeuvre, imdb_id: &str) -> Result<i32> {
    let oeuvre_id = if let Ok(oeuvre_id) = get_imdb_oeuvre_id(conn, imdb_id) {
        update_oeuvre(conn, oeuvre_id, new_oeuvre)?;
        oeuvre_id
    } else {
        let oeuvre_id = add_oeuvre(conn, new_oeuvre)?;
        conn.execute("INSERT INTO imdb_map(oeuvre_id, imdb_id) VALUES(?1, ?2)", params![oeuvre_id, imdb_id])?;
        oeuvre_id
    };
    Ok(oeuvre_id)
}