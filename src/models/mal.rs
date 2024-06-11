use rusqlite::Result;
use rusqlite::Connection;
use crate::add_oeuvre;
use crate::update_oeuvre;
use super::NewOeuvre;

pub fn get_mal_id(conn: &Connection, oeuvre_id: i32) -> Result<i32> {
    conn.prepare_cached("SELECT mal_id FROM mal_map WHERE oeuvre_id = ?1")?
        .query_row([oeuvre_id], |row| row.get::<usize, i32>(0))
}

pub fn get_mal_oeuvre_id(conn: &Connection, mal_id: i32) -> Result<i32> {
    conn.prepare_cached("SELECT oeuvre_id FROM mal_map WHERE mal_id = ?1")?
        .query_row([mal_id], |row| row.get::<usize, i32>(0))
}

pub fn add_mal_oeuvre(conn: &Connection, new_oeuvre: NewOeuvre, mal_id: i32) -> Result<i32> {
    let oeuvre_id = if let Ok(oeuvre_id) = get_mal_oeuvre_id(conn, mal_id) {
        update_oeuvre(conn, oeuvre_id, new_oeuvre)?;
        oeuvre_id
    } else {
        let oeuvre_id = add_oeuvre(conn, new_oeuvre)?;
        conn.execute("INSERT INTO mal_map(oeuvre_id, mal_id) VALUES(?1, ?2)", [oeuvre_id, mal_id])?;
        oeuvre_id
    };
    Ok(oeuvre_id)
}