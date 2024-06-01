use diesel::{prelude::*, result::DatabaseErrorKind};
use rocket::{http::Status, launch, post, routes, serde::json::Json};
use what_next_back::{
    add_user, update_user_rating, api_models::{CredentialRequest, RateRequest}, check_credential, establish_connection, jwt_auth::{create_jwt, JWT}, models::Oeuvre, Medium
};

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
    // TODO: do a recommendation based on user id in jwt.claims.user_id & medium
    use what_next_back::schema::oeuvres::dsl::*;
    let connection: &mut diesel::prelude::SqliteConnection = &mut establish_connection();
    let reco: Oeuvre = oeuvres
        .order(rating.desc())
        .first(connection).map_err(|_| Status::InternalServerError)?;
    Ok(serde_json::to_string(&reco).map_err(|_| Status::InternalServerError)?)
}

#[post("/rate", format = "application/json", data = "<rating>")]
fn rate(jwt: JWT, rating: Json<RateRequest>) -> Result<String, Status> {
    let connection: &mut diesel::prelude::SqliteConnection = &mut establish_connection();
    update_user_rating(connection, jwt.claims.user_id, rating.oeuvre_id, rating.rating)
        .map_err(|_| Status::InternalServerError)?;
    Ok(String::new())
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![sign_up, login, reco])
}