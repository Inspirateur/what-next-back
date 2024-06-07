use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHasher, SaltString
    }, Argon2, PasswordHash, PasswordVerifier
};
use rusqlite::{params, Connection, OptionalExtension, Result};
use crate::{on_rating_add, on_rating_remove, on_rating_update, AppRating, Medium, Oeuvre, RatingOn100};

/// Returns the newly created user id
pub fn add_user(conn: &Connection, username: &str, pwd: &str) -> Result<i32> {
    let salt = SaltString::generate(&mut OsRng);

    let phc = Argon2::default()
        .hash_password(pwd.as_bytes(), &salt)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?
        .serialize();

    conn.prepare("INSERT INTO users(username, phc) VALUES(?1, ?2) RETURNING id")?
        .query_row([username, phc.as_str()], |row| row.get::<usize, i32>(0))
}

/// Returns the corresponding user id if successful
pub fn check_credential(conn: &Connection, username: &str, pwd: &str) -> Result<i32> {
    let (user_id, phc_str) = conn.prepare_cached("SELECT id, phc FROM users WHERE username = ?1")?
        .query_row([username], |row| Ok((row.get::<usize, i32>(0)?, row.get::<usize, String>(1)?)))?;
    let phc = PasswordHash::new(&phc_str)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

    if Argon2::default().verify_password(pwd.as_bytes(), &phc).is_ok() {
        Ok(user_id)
    } else {
        Err(rusqlite::Error::QueryReturnedNoRows)
    }
}

pub fn change_password(conn: &Connection, username: &str, old_pwd: &str, new_pwd: &str) -> Result<i32> {
    let (user_id, phc_str) = conn.prepare("SELECT id, phc FROM users WHERE username = ?1")?
        .query_row([username], |row| Ok((row.get::<usize, i32>(0)?, row.get::<usize, String>(1)?)))?;

    let old_phc = PasswordHash::new(&phc_str)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

    if Argon2::default().verify_password(old_pwd.as_bytes(), &old_phc).is_err() {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    };

    let salt = SaltString::generate(&mut OsRng);
    let new_phc = Argon2::default()
        .hash_password(new_pwd.as_bytes(), &salt)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?
        .serialize();
    
    conn.execute("UPDATE users SET phc = ?1 WHERE id = ?2", params![user_id, new_phc.as_str()])?;
    Ok(user_id)
}

pub fn update_user_rating(conn: &Connection, user_id: i32, oeuvre_id: i32, rating: AppRating) 
    -> Result<()> 
{
    if let Ok(old_rating) = conn.prepare_cached("SELECT rating FROM user_ratings WHERE user_id = ?1 AND oeuvre_id = ?2")?
        .query_row([user_id, oeuvre_id], |row| row.get::<usize, i32>(0)) {
        conn.execute(
            "UPDATE user_ratings SET rating = ?1 WHERE user_id = ?2 AND oeuvre_id = ?3", 
            [rating.0, user_id, oeuvre_id])?;
        on_rating_update(conn, user_id, oeuvre_id, AppRating(old_rating), rating)
    } else {
        conn.execute(
            "INSERT INTO user_ratings(user_id, oeuvre_id, rating) VALUES(?1, ?2, ?3)", 
            [user_id, oeuvre_id, rating.0])?;
        on_rating_add(conn, user_id, oeuvre_id, rating)
    }
}

pub fn remove_user_rating(conn: &Connection, user_id: i32, oeuvre_id: i32) -> Result<()> {
    if let Some(old_rating) = conn.prepare_cached("DELETE FROM user_ratings WHERE user_id = ?1 AND oeuvre_id = ?2 RETURNING rating")?
        .query_row([user_id, oeuvre_id], |row| row.get::<usize, i32>(0).map(|r| AppRating(r)).optional())? {
        on_rating_remove(conn, user_id, oeuvre_id, old_rating)
    } else {
        Ok(())
    }
}

pub fn get_rated_oeuvres(conn: &Connection, user_id: i32) -> Result<Vec<Oeuvre>> {
    conn.prepare_cached(
        "SELECT oeuvres.id, oeuvres.medium, oeuvres.title, oeuvres.picture, user_ratings.rating 
        FROM user_ratings INNER JOIN oeuvres ON user_ratings.user_id = ?1 AND oeuvres.id = user_ratings.oeuvre_id")?
        .query_map([user_id], |row| Ok(Oeuvre {
            id: row.get::<usize, i32>(0)?,
            medium: Medium::from(row.get::<usize, i32>(1)?),
            title: row.get::<usize, String>(2)?,
            picture: row.get::<usize, String>(3)?,
            rating: RatingOn100(0),
            synopsis: String::new(),
            tags: Vec::new(),
            user_rating: Some(AppRating(row.get::<usize, i32>(4)?))
        }))?.collect::<Result<Vec<_>>>()
}

pub fn get_user_id(conn: &Connection, username: &str) -> Result<i32> {
    conn.prepare_cached("SELECT id FROM users WHERE username = ?1")?
        .query_row([username], |row| row.get::<usize, i32>(0))
}
