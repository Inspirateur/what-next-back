use std::collections::HashMap;
use rusqlite::{Connection, Result, params};
use crate::{Medium, Oeuvre, RatingOn100};

pub fn add_token(conn: &Connection, oeuvre_id: i32, token: &str) -> Result<()> {
    conn.execute("INSERT OR IGNORE INTO search_tokens(oeuvre_id, token) VALUES (?1, ?2)", params![oeuvre_id, token])?;
    Ok(())
}

pub fn search_oeuvres(conn: &Connection, medium: Medium, tokens: Vec<&str>) -> Result<Vec<Oeuvre>> {
    let mut scores: HashMap<i32, i32> = HashMap::new();
    let mut oeuvres: HashMap<i32, Oeuvre> = HashMap::new();
    // Get oeuvres matching the token and medium
    let mut stmt = conn.prepare_cached(
        "SELECT oeuvres.id, oeuvres.title, oeuvres.picture, oeuvres.rating 
        FROM search_tokens 
        INNER JOIN oeuvres ON search_tokens.token = ?1 AND oeuvres.medium = ?2 AND search_tokens.oeuvre_id = oeuvres.id"
    )?;
    for token in tokens {
        for (id, title, picture, rating) in stmt.query_map(params![token, medium as i32], |row| Ok((
                row.get::<usize, i32>(0)?, row.get::<usize, String>(1)?, 
                row.get::<usize, String>(2)?, row.get::<usize, i32>(3)?))
            )?.filter_map(|r| r.ok())
        {
            oeuvres.insert(id, Oeuvre {
                id, medium, title, picture, rating: RatingOn100(rating), 
                synopsis: String::new(), tags: Vec::new(), user_rating: None
            });
            scores.entry(id).and_modify(|counter| *counter += rating).or_insert(rating);
        }
    }
    let mut res: Vec<_> = oeuvres.into_iter().filter(|(id, _)| *scores.get(id).unwrap() > 60).map(|(_, oeuvre)| oeuvre).collect();
    res.sort_by(|a, b| scores.get(&b.id).unwrap().cmp(scores.get(&a.id).unwrap()));
    Ok(res)
}