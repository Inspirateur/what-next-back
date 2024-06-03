use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHasher, SaltString
    }, Argon2, PasswordHash, PasswordVerifier
};
use rusqlite::{params, Connection, OptionalExtension, Result};
use crate::AppRating;

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
pub fn check_credential(conn: &Connection, username: &str, pwd: &str) -> Result<Option<i32>> {
    let (user_id, phc_str): (i32, String) = conn.prepare("SELECT (id, phc) FROM users WHERE username = ?1")?
        .query_row([username], |row| Ok((row.get::<usize, i32>(0)?, row.get::<usize, String>(1)?)))?;
    let phc = PasswordHash::new(&phc_str)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

    if Argon2::default().verify_password(pwd.as_bytes(), &phc).is_ok() {
        Ok(Some(user_id))
    } else {
        Ok(None)
    }
}

/// Returns the corresponding user id if successful
pub fn change_pwd(conn: &Connection, username: &str, old_pwd: &str, new_pwd: &str) -> Result<Option<i32>> {
    let (user_id, phc_str): (i32, String) = conn.prepare("SELECT (id, phc) FROM users WHERE username = ?1")?
        .query_row([username], |row| Ok((row.get::<usize, i32>(0)?, row.get::<usize, String>(1)?)))?;

    let old_phc = PasswordHash::new(&phc_str)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

    if Argon2::default().verify_password(old_pwd.as_bytes(), &old_phc).is_err() {
        return Ok(None);
    };

    let salt = SaltString::generate(&mut OsRng);
    let new_phc = Argon2::default()
        .hash_password(new_pwd.as_bytes(), &salt)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?
        .serialize();
    
    conn.execute("UPDATE users SET phc = ?1 WHERE id = ?2", params![user_id, new_phc.as_str()])?;
    Ok(Some(user_id))
}

/// Returns the previous rating if there was any
pub fn update_user_rating(conn: &Connection, user_id: i32, oeuvre_id: i32, rating: AppRating) 
    -> Result<Option<AppRating>> 
{
    if let Ok(user_rating) = conn.prepare("SELECT rating FROM user_ratings WHERE user_id = ?1 AND oeuvre_id = ?2")?
        .query_row([user_id, oeuvre_id], |row| row.get::<usize, i32>(0)) {
        conn.execute(
            "UPDATE user_ratings SET rating = ?1 WHERE user_id = ?2 AND oeuvre_id = ?3", 
            [rating.0, user_id, oeuvre_id])?;
        Ok(Some(AppRating(user_rating)))
    } else {
        conn.execute(
            "INSERT INTO user_ratings(user_id, oeuvre_id, rating) VALUES(?1, ?2, ?3)", 
            [user_id, oeuvre_id, rating.0])?;
        Ok(None)
    }
}

/// Returns the previous rating if there was any
pub fn remove_user_rating(conn: &Connection, user_id: i32, oeuvre_id: i32) -> Result<Option<AppRating>> {
    conn.prepare("DELETE FROM user_ratings WHERE user_id = ?1 AND oeuvre_id = ?2 RETURNING rating")?
        .query_row([user_id, oeuvre_id], |row| row.get::<usize, i32>(0).map(|r| AppRating(r)).optional())
}