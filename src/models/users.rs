use argon2::{
    password_hash::{
        rand_core::{OsRng, RngCore},
        PasswordHasher, SaltString
    },
    Argon2
};
use diesel::prelude::*;
use super::{User, UserRating};
use crate::{schema::{user_ratings, users}, NewUser};

/// Returns the newly created user id
pub fn add_user(conn: &mut SqliteConnection, username: &str, pwd: &str) -> diesel::result::QueryResult<i32> {
    println!("username: {username} | password: {pwd}");
    let mut salt = [0u8; 16];
    (&mut OsRng).fill_bytes(&mut salt);
    let argon2 = Argon2::default();
    let mut password_hash;
    let output_len = self
            .params
            .output_len()
            .unwrap_or(Params::DEFAULT_OUTPUT_LEN);

    let password_hash = argon2.hash_password(pwd.as_bytes(), &salt)
        .map_err(|e| diesel::result::Error::DeserializationError(Box::new(e)))?
        .hash.unwrap().as_bytes();

    let new_user = NewUser {
        username: username,
        pwd_hash: &password_hash,
        pwd_salt: salt.into().as_bytes(),
    };
    let inserted_user: User = diesel::insert_into(users::table)
        .values(&new_user)
        .returning(User::as_returning())
        .get_result(conn)?;

    diesel::result::QueryResult::Ok(inserted_user.id)
}

/// Returns the corresponding user id if successful
pub fn check_credential(conn: &mut SqliteConnection, username: &str, pwd: &str) -> diesel::result::QueryResult<Option<i32>> {
    println!("username: {username} | password: {pwd}");
    let user: User = users::table.filter(users::username.eq(username)).first(conn)?;
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(
        pwd.as_bytes(), 
        SaltString::from(user.pwd_salt)
    );
    diesel::result::QueryResult::Ok(None)
}

/// Returns the corresponding user id if successful
pub fn change_pwd(conn: &mut SqliteConnection, username: &str, old_pwd: &str, new_pwd: &str) -> diesel::result::QueryResult<Option<i32>> {
    diesel::result::QueryResult::Ok(None)
}