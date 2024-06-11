use rusqlite::params;
use rusqlite::Result;
use rusqlite::Connection;
use crate::add_oeuvre;
use crate::update_oeuvre;
use super::NewOeuvre;

pub fn get_isbn10(conn: &Connection, oeuvre_id: i32) -> Result<String> {
    conn.prepare_cached("SELECT isbn10 FROM isbn_map WHERE oeuvre_id = ?1")?
        .query_row([oeuvre_id], |row| row.get::<usize, String>(0))
}

pub fn get_isbn_oeuvre_id(conn: &Connection, isbn10: &str) -> Result<i32> {
    conn.prepare_cached("SELECT oeuvre_id FROM isbn_map WHERE isbn10 = ?1")?
        .query_row([isbn10], |row| row.get::<usize, i32>(0))
}

pub fn add_isbn_oeuvre(conn: &Connection, new_oeuvre: NewOeuvre, isbn10: &str) -> Result<i32> {
    let oeuvre_id = if let Ok(oeuvre_id) = get_isbn_oeuvre_id(conn, isbn10) {
        update_oeuvre(conn, oeuvre_id, new_oeuvre)?;
        oeuvre_id
    } else {
        let oeuvre_id = add_oeuvre(conn, new_oeuvre)?;
        conn.execute("INSERT INTO isbn_map(oeuvre_id, isbn10) VALUES(?1, ?2)", params![oeuvre_id, isbn10])?;
        oeuvre_id
    };
    Ok(oeuvre_id)
}