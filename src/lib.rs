pub mod models;
pub mod api_models;
pub mod jwt_auth;
pub use models::*;
use dotenvy::dotenv;
use anyhow::Result;
use rusqlite::{ffi, Connection, ErrorCode};
use std::{env, fs};

pub enum DatabaseKind {
    TEST,
    PROD,
}

impl DatabaseKind {
    pub fn env_var(&self) -> String {
        match self {
            DatabaseKind::TEST => "TEST_DATABASE_URL".to_string(),
            DatabaseKind::PROD => "DATABASE_URL".to_string()
        }
    }
}

pub fn setup_db(db_kind: DatabaseKind) -> Result<()> {
    dotenv().ok();

    let database_url = env::var(db_kind.env_var()).expect(&format!("{} must be set", db_kind.env_var()));
    let _ = fs::remove_file(&database_url);
    let conn = Connection::open(database_url)?;
    conn.execute_batch(&fs::read_to_string("what_next_setup.sql")?)?;
    Ok(())
}

pub fn establish_connection(db_kind: DatabaseKind) -> Result<Connection> {
    dotenv().ok();

    let database_url = env::var(db_kind.env_var()).expect(&format!("{} must be set", db_kind.env_var()));
    Ok(Connection::open(database_url)?)
}

pub fn is_constraint_violation<T>(res: &Result<T, rusqlite::Error>) -> bool {
    match res {
        Err(rusqlite::Error::SqliteFailure(ffi::Error { code: ErrorCode::ConstraintViolation, extended_code: _}, _))
            => true,
        _ => false
    }
}

// Don't run the test in parallel, they operate on the same DB
#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use super::*;

    fn assert_constraint_violation<T: Debug>(res: Result<T, rusqlite::Error>) {
        match res {
            _ if is_constraint_violation(&res) => (),
            x => panic!("Expected a Constraint Violation, got {:?}", x)
        }
    }

    #[test]
    fn test_auth() {
        setup_db(DatabaseKind::TEST).unwrap();
        let conn = establish_connection(DatabaseKind::TEST).unwrap();
        let user_id = add_user(&conn, "test", "aBadPassword").unwrap();
        // duplicated username should fail with a constraint violation
        assert_constraint_violation(add_user(&conn, "test", "whatever"));
        // bad username/password combo should err with QueryReturnedNoRows
        assert_eq!(check_credential(&conn, "test", "theWrongPassword"), Err(rusqlite::Error::QueryReturnedNoRows));
        assert_eq!(check_credential(&conn, "wrong_username", "aBadPassword"), Err(rusqlite::Error::QueryReturnedNoRows));
        // correct username/password combo should succeed and return the user id
        assert_eq!(check_credential(&conn, "test", "aBadPassword"), Ok(user_id));
        assert_eq!(change_password(&conn, "test", "aBadPassword", "whatever"), Ok(user_id));
    }

    #[test]
    fn test_reco() {
        setup_db(DatabaseKind::TEST).unwrap();
        let conn = establish_connection(DatabaseKind::TEST).unwrap();
        let outerstellar = add_oeuvre(&conn, NewOeuvre { 
            medium: Medium::Movie, title: "Outerstellar", rating: RatingOn100(85), 
            synopsis: "", picture: "" }).unwrap();
        let outception = add_oeuvre(&conn, NewOeuvre { 
            medium: Medium::Movie, title: "Outception", rating: RatingOn100(81), 
            synopsis: "", picture: "" }).unwrap();
        let _crap_movie = add_oeuvre(&conn, NewOeuvre { 
            medium: Medium::Movie, title: "Crap Movie", rating: RatingOn100(20), 
            synopsis: "", picture: "" }).unwrap();
        let barry_motter = add_oeuvre(&conn, NewOeuvre { 
            medium: Medium::Book, title: "Barry Motter", rating: RatingOn100(85), 
            synopsis: "", picture: "" }).unwrap();
        let _game_of_chairs = add_oeuvre(&conn, NewOeuvre { 
            medium: Medium::Book, title: "Game of Chairs", rating: RatingOn100(78), 
            synopsis: "", picture: "" }).unwrap();
        let main_user = add_user(&conn, "main_user", "doesn'tMatter").unwrap();
        let similar_user = add_user(&conn, "similar_user", "whatev").unwrap();
        let dissimilar_user = add_user(&conn, "dissimilar_user", "azndlaknd l").unwrap();
        // With no data on the user the reco should be the most popular oeuvre in the medium
        assert_eq!(get_reco(&conn, main_user, Medium::Movie), Ok(Some(Reco { oeuvre_id: outerstellar, score: RatingOn100(85)})));
        // Make one user similar and the other dissimilar to the "main user"
        update_user_rating(&conn, main_user, barry_motter, AppRating(2)).unwrap();
        update_user_rating(&conn, similar_user, barry_motter, AppRating(2)).unwrap();
        update_user_rating(&conn, dissimilar_user, barry_motter, AppRating(-2)).unwrap();
        // Make up Movie ratings
        // The similar user has a preference for outception but also likes outerstellar
        update_user_rating(&conn, similar_user, outception, AppRating(2)).unwrap();
        update_user_rating(&conn, similar_user, outerstellar, AppRating(1)).unwrap();
        // The dissimilar user hates outception and loves outerstellar
        update_user_rating(&conn, dissimilar_user, outception, AppRating(-2)).unwrap();
        update_user_rating(&conn, dissimilar_user, outerstellar, AppRating(2)).unwrap();
        // If we just went by overall rating + user rating, outerstellar should be the favorite
        // But since the most similar user to the main user prefers outception, outception should be recommended
        assert_eq!(get_reco(&conn, main_user, Medium::Movie), Ok(Some(Reco { oeuvre_id: outception, score: RatingOn100(94)})));
    }
}