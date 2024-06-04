use rocket::{http::Status, launch, post, routes, serde::json::Json};
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

#[post("/reco", format = "application/json", data = "<medium>")]
fn reco(jwt: JWT, medium: Json<Medium>) -> Result<String, Status> {
    let conn = establish_connection(DatabaseKind::PROD).map_err(|_| Status::InternalServerError)?;
    let Some(reco) = get_reco(&conn, jwt.claims.user_id, medium.0).map_err(|_| Status::InternalServerError)? else {
        return Err(Status::NotFound);
    };
    if reco.score.0 < 50 {
        return Err(Status::NotFound);
    }
    let mut oeuvre: Oeuvre = get_oeuvre(&conn, reco.oeuvre_id).map_err(|_| Status::InternalServerError)?;
    oeuvre.rating = reco.score;
    Ok(serde_json::to_string(&oeuvre).map_err(|_| Status::InternalServerError)?)
}

#[post("/rate", format = "application/json", data = "<rating>")]
fn rate(jwt: JWT, rating: Json<RateRequest>) -> Result<String, Status> {
    let conn = &establish_connection(DatabaseKind::PROD).map_err(|_| Status::InternalServerError)?;
    update_user_rating(&conn, jwt.claims.user_id, rating.oeuvre_id, rating.rating)
        .map_err(|_| Status::InternalServerError)?;
    Ok(String::new())
}

#[post("/unrate", format = "application/json", data = "<oeuvre_id>")]
fn unrate(jwt: JWT, oeuvre_id: Json<i32>) -> Result<String, Status> {
    let conn = establish_connection(DatabaseKind::PROD).map_err(|_| Status::InternalServerError)?;
    remove_user_rating(&conn, jwt.claims.user_id, oeuvre_id.0)
        .map_err(|_| Status::InternalServerError)?;
    Ok(String::new())
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![sign_up, login, change_pwd, reco, rate, unrate])
}