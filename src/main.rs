use diesel::result::DatabaseErrorKind;
use rocket::{http::Status, launch, post, routes, serde::json::Json};
use log::warn;
use api_models::*;
use jwt_auth::*;
use what_next_back::*;

#[post("/signup", format = "application/json", data = "<user>")]
fn sign_up(user: Json<CredentialRequest>) -> Result<String, Status> {
    let conn = &mut establish_connection();
    let user_id = add_user(conn, &user.username, &user.password).map_err(|e| match e {
        diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => Status::Conflict, 
        _ => Status::InternalServerError
    })?;
    Ok(create_jwt(user_id).map_err(|_| Status::InternalServerError)?)
}

#[post("/login", format = "application/json", data = "<user>")]
fn login(user: Json<CredentialRequest>) -> Result<String, Status> {
    let conn = &mut establish_connection();
    let Some(user_id) = check_credential(conn, &user.username, &user.password)
        .map_err(|e| match e {
            diesel::result::Error::NotFound => Status::Unauthorized,
            _ => Status::InternalServerError
        })? else {
        return Err(Status::Unauthorized);
    };
    Ok(create_jwt(user_id).map_err(|_| Status::InternalServerError)?)
}

#[post("/reco", format = "application/json", data = "<medium>")]
fn reco(jwt: JWT, medium: Json<Medium>) -> Result<String, Status> {
    let connection: &mut diesel::prelude::SqliteConnection = &mut establish_connection();
    let reco: Oeuvre = get_reco(connection, jwt.claims.user_id, medium.0).map_err(|_| Status::InternalServerError)?;
    Ok(serde_json::to_string(&reco).map_err(|_| Status::InternalServerError)?)
}

#[post("/rate", format = "application/json", data = "<rating>")]
fn rate(jwt: JWT, rating: Json<RateRequest>) -> Result<String, Status> {
    let connection: &mut diesel::prelude::SqliteConnection = &mut establish_connection();
    if let Some(old_rating) = update_user_rating(connection, jwt.claims.user_id, rating.oeuvre_id, rating.rating)
        .map_err(|_| Status::InternalServerError)? 
    {
        on_rating_update(connection, jwt.claims.user_id, rating.oeuvre_id, old_rating, rating.rating)
            .map_err(|_| Status::InternalServerError)?;
    } else {
        on_rating_add(connection, jwt.claims.user_id, rating.oeuvre_id, rating.rating)
            .map_err(|_| Status::InternalServerError)?;
    }
    Ok(String::new())
}

#[post("/unrate", format = "application/json", data = "<oeuvre_id>")]
fn unrate(jwt: JWT, oeuvre_id: Json<i32>) -> Result<String, Status> {
    let connection: &mut diesel::prelude::SqliteConnection = &mut establish_connection();
    if let Some(old_rating) = remove_user_rating(connection, jwt.claims.user_id, oeuvre_id.0).map_err(|_| Status::InternalServerError)? {
        on_rating_remove(connection, jwt.claims.user_id, oeuvre_id.0, old_rating).map_err(|_| Status::InternalServerError)?;
    } else {
        warn!(target: "what-next", "User #{} attempted to remove rating for #{} which didn't exist", jwt.claims.user_id, oeuvre_id.0);
    }
    Ok(String::new())
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![sign_up, login, reco, rate, unrate])
}