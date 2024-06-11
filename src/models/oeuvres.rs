use rusqlite::{params, Connection, Result};
use crate::{index_oeuvre, Medium, NewOeuvre, Oeuvre, RatingOn100};

/// Returns oeuvre id if succesfull
pub fn add_oeuvre(conn: &Connection, new_oeuvre: NewOeuvre) -> Result<i32> {
    let oeuvre_id = conn.prepare_cached("INSERT INTO oeuvres(medium, title, rating, synopsis, picture) VALUES(?1, ?2, ?3, ?4, ?5) RETURNING id")?
        .query_row(params![
            new_oeuvre.medium as i32, new_oeuvre.title, new_oeuvre.rating.0, 
            new_oeuvre.synopsis, new_oeuvre.picture
        ], |row| row.get::<usize, i32>(0))?;
    index_oeuvre(conn, oeuvre_id, new_oeuvre.title)?;
    Ok(oeuvre_id)
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
    let tags = conn.prepare_cached("SELECT tags.label FROM tags INNER JOIN oeuvre_tags ON oeuvre_tags.oeuvre_id = ?1")?
        .query_map([oeuvre_id], |row| row.get::<usize, String>(0))?.collect::<Result<Vec<_>>>()?;
    conn.prepare_cached("SELECT medium, title, rating, synopsis, picture FROM oeuvres WHERE id = ?1")?
        .query_row([oeuvre_id], |row| Ok(Oeuvre {
            id: oeuvre_id,
            medium: Medium::from(row.get::<usize, i32>(0)?),
            title: row.get::<usize, String>(1)?,
            rating: RatingOn100(row.get::<usize, i32>(2)?),
            synopsis: row.get::<usize, String>(3)?,
            picture: row.get::<usize, String>(4)?,
            tags,
            user_rating: None,
        }))
}

pub fn insert_or_replace_tag(conn: &Connection, tag_label: String) -> Result<i32> {
    if let Ok(tag_id) = conn.prepare_cached("SELECT id FROM tags WHERE label = ?1")?
        .query_row([&tag_label], |row| row.get::<usize, i32>(0)) 
    {
        Ok(tag_id)
    } else {
        conn.prepare_cached("INSERT INTO tags(label) VALUES(?1) RETURNING id")?
            .query_row([&tag_label], |row| row.get::<usize, i32>(0))
    }
}

pub fn add_tag(conn: &Connection, oeuvre_id: i32, tag_label: String) -> Result<()> {
    let tag_id = insert_or_replace_tag(conn, tag_label)?;
    conn.execute("INSERT INTO oeuvre_tags(oeuvre_id, tag_id) VALUES(?1, ?2) ON CONFLICT DO NOTHING", [oeuvre_id, tag_id])?;
    Ok(())
}

pub fn update_rating(conn: &Connection, oeuvre_id: i32, rating: RatingOn100) -> Result<()> {
    conn.execute("UPDATE oeuvres SET rating = ?1 WHERE id = ?2", [rating.0, oeuvre_id])?;
    Ok(())
}

pub fn add_picture(conn: &Connection, oeuvre_id: i32, picture: &str) -> Result<()> {
    conn.execute("UPDATE oeuvres SET picture = ?1 WHERE id = ?2", params![picture, oeuvre_id])?;
    Ok(())
}

pub fn add_synopsis(conn: &Connection, oeuvre_id: i32, synopsis: &str) -> Result<()> {
    conn.execute("UPDATE oeuvres SET synopsis = ?1 WHERE id = ?2", params![synopsis, oeuvre_id])?;
    Ok(())
}