use std::error::Error;
use rocket::{get, http::{Method, Status}, post, routes, serde::json::Json};
use rocket_cors::{AllowedHeaders, AllowedOrigins};
use rusqlite::Connection;
use strum::IntoEnumIterator;
use api_models::*;
use jwt_auth::*;
use what_next_back::*;

#[post("/signup", format = "application/json", data = "<req>")]
fn sign_up(req: Json<CredentialRequest>) -> Result<String, Status> {
    let conn = establish_connection(DatabaseKind::PROD).map_err(|_| Status::InternalServerError)?;
    let user_id = add_user(&conn, &req.username, &req.password).map_err(|e| match e {
        _ if is_constraint_violation::<()>(&Err(e)) => Status::Conflict,
        _ => Status::InternalServerError
    })?;
    Ok(create_jwt(user_id).map_err(|_| Status::InternalServerError)?)
}

#[post("/login", format = "application/json", data = "<req>")]
fn login(req: Json<CredentialRequest>) -> Result<String, Status> {
    let conn = establish_connection(DatabaseKind::PROD).map_err(|_| Status::InternalServerError)?;
    let user_id = check_credential(&conn, &req.username, &req.password)
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Status::Unauthorized,
            _ => Status::InternalServerError
        })?;
    Ok(create_jwt(user_id).map_err(|_| Status::InternalServerError)?)
}

#[post("/change_password", format = "application/json", data = "<req>")]
fn change_pwd(req: Json<ChangePasswordRequest>) -> Result<String, Status> {
    let conn = establish_connection(DatabaseKind::PROD).map_err(|_| Status::InternalServerError)?;
    let user_id = change_password(&conn, &req.username, &req.old_password, &req.new_password)
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Status::Unauthorized,
            _ => Status::InternalServerError
        })?;
    Ok(create_jwt(user_id).map_err(|_| Status::InternalServerError)?)
}

#[get("/media")]
fn media() -> Result<String, Status> {
    serde_json::to_string(&Medium::iter().collect::<Vec<_>>()).map_err(|_| Status::InternalServerError)
}

fn _reco(conn: &Connection, user_id: i32, medium: Medium) -> Result<String, Status> {
    let Some(reco) = get_reco(&conn, user_id, medium).map_err(|_| Status::InternalServerError)? else {
        return Err(Status::NotFound);
    };
    if reco.score.0 < 50 {
        return Err(Status::NotFound);
    }
    let mut oeuvre: Oeuvre = get_oeuvre(&conn, reco.oeuvre_id).map_err(|_| Status::InternalServerError)?;
    oeuvre.rating = reco.score;
    Ok(serde_json::to_string(&oeuvre).map_err(|_| Status::InternalServerError)?)
}

#[post("/reco", format = "application/json", data = "<medium>")]
fn reco(jwt: JWT, medium: Json<Medium>) -> Result<String, Status> {
    let conn = establish_connection(DatabaseKind::PROD).map_err(|_| Status::InternalServerError)?;
    _reco(&conn, jwt.claims.user_id, medium.0)
}

#[post("/rate_reco", format = "application/json", data = "<req>")]
fn rate_reco(jwt: JWT, req: Json<RateRequest>) -> Result<String, Status> {
    let conn = &establish_connection(DatabaseKind::PROD).map_err(|_| Status::InternalServerError)?;
    update_user_rating(&conn, jwt.claims.user_id, req.oeuvre_id, req.rating)
        .map_err(|_| Status::InternalServerError)?;
    let medium = get_oeuvre(conn, req.oeuvre_id).map_err(|_| Status::InternalServerError)?.medium;
    _reco(&conn, jwt.claims.user_id, medium)
}

#[post("/rate", format = "application/json", data = "<req>")]
fn rate(jwt: JWT, req: Json<RateRequest>) -> Result<(), Status> {
    let conn = &establish_connection(DatabaseKind::PROD).map_err(|_| Status::InternalServerError)?;
    update_user_rating(&conn, jwt.claims.user_id, req.oeuvre_id, req.rating)
        .map_err(|_| Status::InternalServerError)?;
    Ok(())
}

#[post("/unrate", format = "application/json", data = "<oeuvre_id>")]
fn unrate(jwt: JWT, oeuvre_id: Json<i32>) -> Result<(), Status> {
    let conn = establish_connection(DatabaseKind::PROD).map_err(|_| Status::InternalServerError)?;
    remove_user_rating(&conn, jwt.claims.user_id, oeuvre_id.0)
        .map_err(|_| Status::InternalServerError)?;
    Ok(())
}

#[get("/rated")]
fn rated(jwt: JWT) -> Result<String, Status> {
    // TODO: return all the oeuvre the user has rated, so they can see and possibily edit the ratings
    todo!()
}

#[get("/search/<medium>/<query>")]
fn search(jwt: JWT, medium: &str, query: &str) -> Result<String, Status> {
    // TODO: return oeuvres from the medium that match the query, also provide rating info if the user has rated it 
    let medium: Medium = serde_json::from_str(medium).map_err(|_| Status::BadRequest)?;
    todo!()
}

#[rocket::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let allowed_origins = AllowedOrigins::all();

    // You can also deserialize this
    let cors = rocket_cors::CorsOptions {
        allowed_origins,
        allowed_methods: vec![Method::Get, Method::Post].into_iter().map(From::from).collect(),
        allowed_headers: AllowedHeaders::all(),
        allow_credentials: true,
        ..Default::default()
    }.to_cors()?;

    let _ = rocket::build()
        .mount("/", routes![sign_up, login, change_pwd, reco, rate, rate_reco, unrate, media])
        .attach(cors)
        .launch()
        .await?;

    Ok(())
}