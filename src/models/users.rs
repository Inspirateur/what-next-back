use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHasher, SaltString
    }, Argon2, PasswordHash, PasswordVerifier
};
use diesel::prelude::*;
use super::{User, UserRating, NewUser};

/// Returns the newly created user id
pub fn add_user(conn: &mut SqliteConnection, username: &str, pwd: &str) -> diesel::result::QueryResult<i32> {
    use crate::schema::users;

    println!("username: {username} | password: {pwd}");
    let salt = SaltString::generate(&mut OsRng);

    let phc = Argon2::default()
        .hash_password(pwd.as_bytes(), &salt)
        .map_err(|e| diesel::result::Error::DeserializationError(Box::new(e)))?
        .serialize();

    let new_user = NewUser {
        username: username,
        phc: phc.as_str(),
    };
    let inserted_user: User = diesel::insert_into(users::table)
        .values(&new_user)
        .returning(User::as_returning())
        .get_result(conn)?;

    diesel::result::QueryResult::Ok(inserted_user.id)
}

/// Returns the corresponding user id if successful
pub fn check_credential(conn: &mut SqliteConnection, username: &str, pwd: &str) -> diesel::result::QueryResult<Option<i32>> {
    use crate::schema::users;

    let user: User = users::table.filter(users::username.eq(username)).first(conn)?;
    let phc = PasswordHash::new(&user.phc)
        .map_err(|e| diesel::result::Error::DeserializationError(Box::new(e)))?;

    if Argon2::default().verify_password(pwd.as_bytes(), &phc).is_ok() {
        diesel::result::QueryResult::Ok(Some(user.id))
    } else {
        diesel::result::QueryResult::Ok(None)
    }
}

/// Returns the corresponding user id if successful
pub fn change_pwd(conn: &mut SqliteConnection, username: &str, old_pwd: &str, new_pwd: &str) -> diesel::result::QueryResult<Option<i32>> {
    use crate::schema::users;

    let user: User = users::table.filter(users::username.eq(username)).first(conn)?;
    let old_phc = PasswordHash::new(&user.phc)
        .map_err(|e| diesel::result::Error::DeserializationError(Box::new(e)))?;

    if Argon2::default().verify_password(old_pwd.as_bytes(), &old_phc).is_err() {
        return diesel::result::QueryResult::Ok(None);
    };

    let salt = SaltString::generate(&mut OsRng);
    let new_phc = Argon2::default()
        .hash_password(new_pwd.as_bytes(), &salt)
        .map_err(|e| diesel::result::Error::DeserializationError(Box::new(e)))?
        .serialize();
    
    diesel::update(users::table.find(user.id)).set(users::phc.eq(new_phc.as_str())).execute(conn)?;
    diesel::result::QueryResult::Ok(Some(user.id))
}

/// Returns the previous rating if there was any
pub fn update_user_rating(conn: &mut SqliteConnection, user_id: i32, oeuvre_id: i32, rating: i32) -> diesel::result::QueryResult<Option<i32>> {
    use crate::schema::user_ratings;
    if let Ok(user_rating) = user_ratings::table.filter(user_ratings::user_id.eq(user_id))
        .filter(user_ratings::oeuvre_id.eq(oeuvre_id))
        .first::<UserRating>(conn) 
    {
        diesel::update(user_ratings::table
            .filter(user_ratings::user_id.eq(user_id))
            .filter(user_ratings::oeuvre_id.eq(oeuvre_id)))
            .set(user_ratings::rating.eq(rating))
            .execute(conn)?;
        diesel::result::QueryResult::Ok(Some(user_rating.rating))
    } else {
        diesel::insert_into(user_ratings::table)
            .values(&UserRating {
                user_id, oeuvre_id, rating
            })
            .execute(conn)?;
        diesel::result::QueryResult::Ok(None)
    }
}

/// Returns the previous rating if there was any
pub fn remove_user_rating(conn: &mut SqliteConnection, user_id: i32, oeuvre_id: i32) -> diesel::result::QueryResult<Option<i32>> {
    use crate::schema::user_ratings;

    diesel::delete(
    user_ratings::table
            .filter(user_ratings::user_id.eq(user_id))
            .filter(user_ratings::oeuvre_id.eq(oeuvre_id))
    ).returning(user_ratings::rating).get_result(conn).optional()
    // diesel::result::QueryResult::Ok(())
}