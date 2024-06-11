use rusqlite::Result;
use rusqlite::Connection;
use crate::add_oeuvre;
use crate::update_oeuvre;
use super::NewOeuvre;

pub fn get_steam_id(conn: &Connection, oeuvre_id: i32) -> Result<i32> {
    conn.prepare_cached("SELECT steam_id FROM steam_map WHERE oeuvre_id = ?1")?
        .query_row([oeuvre_id], |row| row.get::<usize, i32>(0))
}

pub fn get_steam_oeuvre_id(conn: &Connection, steam_id: i32) -> Result<i32> {
    conn.prepare_cached("SELECT oeuvre_id FROM steam_map WHERE steam_id = ?1")?
        .query_row([steam_id], |row| row.get::<usize, i32>(0))
}

pub fn add_steam_oeuvre(conn: &Connection, new_oeuvre: NewOeuvre, steam_id: i32) -> Result<i32> {
    let oeuvre_id = if let Ok(oeuvre_id) = get_steam_oeuvre_id(conn, steam_id) {
        update_oeuvre(conn, oeuvre_id, new_oeuvre)?;
        oeuvre_id
    } else {
        let oeuvre_id = add_oeuvre(conn, new_oeuvre)?;
        conn.execute("INSERT INTO steam_map(oeuvre_id, steam_id) VALUES(?1, ?2)", [oeuvre_id, steam_id])?;
        oeuvre_id
    };
    Ok(oeuvre_id)
}