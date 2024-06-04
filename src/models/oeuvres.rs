use rusqlite::{params, Connection, Result};
use crate::{Medium, NewOeuvre, Oeuvre, RatingOn100};

/// Returns oeuvre id if succesfull
pub fn add_oeuvre(conn: &Connection, new_oeuvre: NewOeuvre) -> Result<i32> {
    conn.prepare_cached("INSERT INTO oeuvres(medium, title, rating, synopsis, picture) VALUES(?1, ?2, ?3, ?4, ?5) RETURNING id")?
        .query_row(params![
            new_oeuvre.medium as i32, new_oeuvre.title, new_oeuvre.rating.0, 
            new_oeuvre.synopsis, new_oeuvre.picture
        ], |row| row.get::<usize, i32>(0))
}

pub fn update_oeuvre(conn: &Connection, oeuvre_id: i32, new_oeuvre: NewOeuvre) -> Result<()> {
    conn.execute("INSERT OR REPLACE INTO oeuvres(id, medium, title, rating, synopsis, picture) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            oeuvre_id, new_oeuvre.medium as i32, new_oeuvre.title, 
            new_oeuvre.rating.0, new_oeuvre.synopsis, new_oeuvre.picture
        ]
    )?;
    Ok(())
}

pub fn get_oeuvre(conn: &Connection, oeuvre_id: i32) -> Result<Oeuvre> {
    conn.prepare("SELECT medium, title, rating, synopsis, picture FROM oeuvres WHERE id = ?1")?
        .query_row([oeuvre_id], |row| Ok(Oeuvre {
            id: oeuvre_id,
            medium: Medium::from(row.get::<usize, i32>(0)?),
            title: row.get::<usize, String>(1)?,
            rating: RatingOn100(row.get::<usize, i32>(2)?),
            synopsis: row.get::<usize, String>(3)?,
            picture: row.get::<usize, String>(4)?
        }))
}


pub fn insert_or_replace_tag(conn: &Connection, tag_label: String) -> Result<i32> {
    conn.prepare_cached("INSERT INTO tags(label) VALUES(?1) ON CONFLICT(label) DO NOTHING RETURNING id")?
        .query_row([tag_label], |row| row.get::<usize, i32>(0))
}

pub fn add_tag(conn: &Connection, oeuvre_id: i32, tag_label: String) -> Result<()> {
    let tag_id = insert_or_replace_tag(conn, tag_label)?;
    conn.execute("INSERT INTO oeuvre_tags(oeuvre_id, tag_id) VALUES(?1, ?2)", [oeuvre_id, tag_id])?;
    Ok(())
}

pub fn update_rating(conn: &Connection, oeuvre_id: i32, rating: RatingOn100) -> Result<()> {
    conn.execute("UPDATE oeuvres SET rating = ?1 WHERE id = ?2", [rating.0, oeuvre_id])?;
    Ok(())
}