mod cors;
use std::{env, error::Error};
use cors::CORS;
use rocket::{get, http::Status, options, post, routes, serde::json::Json};
use serde_json::Map;
use strum::IntoEnumIterator;
use api_models::*;
use jwt_auth::*;
use what_next_back::*;

// CORS boilerplate that really should be handled by Rocket but is unfortunately not.
#[options("/signup")]
fn _sign_up() {}

#[post("/signup", format = "application/json", data = "<req>")]
fn sign_up(req: Json<CredentialRequest>) -> Result<String, Status> {
    let conn = establish_connection(DatabaseKind::PROD).map_err(|_| Status::InternalServerError)?;
    let user_id = add_user(&conn, &req.username, &req.password).map_err(|e| match e {
        _ if is_constraint_violation::<()>(&Err(e)) => Status::Conflict,
        _ => Status::InternalServerError
    })?;
    Ok(create_jwt(user_id).map_err(|_| Status::InternalServerError)?)
}

#[options("/login")]
fn _login() {}

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

#[options("/change_password")]
fn _change_pwd() {}

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
#[options("/media")]
fn _media() {}

#[get("/media")]
fn media() -> Result<String, Status> {
    serde_json::to_string(&Medium::iter().collect::<Vec<_>>()).map_err(|_| Status::InternalServerError)
}

async fn reco_worker(user_id: i32, medium: Medium) -> Result<String, Status> {
    let conn = establish_connection(DatabaseKind::PROD).map_err(|_| Status::InternalServerError)?;
    let Some(reco) = get_reco(&conn, user_id, medium).map_err(|_| Status::InternalServerError)? else {
        return Err(Status::NotFound);
    };
    if reco.score.0 < 50 {
        return Err(Status::NotFound);
    }
    let mut oeuvre: Oeuvre = get_oeuvre(&conn, reco.oeuvre_id).map_err(|_| Status::InternalServerError)?;
    if oeuvre.picture.len() == 0 && matches!(medium, Medium::Movie | Medium::Series | Medium::AnimationMovie) {
        // get imdb picture if possible
        // TODO: turn this into a chain of .map when async closure are stabilized
        if let Ok(imdb_id) = get_imdb_id(&conn, reco.oeuvre_id) {
            if let Ok(res) = reqwest::get(format!(
                "https://www.omdbapi.com/?i={}&apikey={}", imdb_id, 
                env::var("OMDB_KEY").expect("OMDB_KEY must be set"))
            ).await {
                if let Ok(body) = res.text().await {
                    if let Ok(map) = serde_json::from_str::<Map<String, serde_json::Value>>(&body) {
                        if let Some(pic_url_opt) = map.get("Poster") {
                            if let Some(pic_url) = pic_url_opt.as_str() {
                                oeuvre.picture = pic_url.to_string();
                                let _ = add_picture(&conn, reco.oeuvre_id, pic_url);
                            }
                        }
                    }
                }
            }
        }
    }
    oeuvre.rating = reco.score;
    Ok(serde_json::to_string(&oeuvre).map_err(|_| Status::InternalServerError)?)
}


#[options("/reco")]
fn _reco() {}

#[post("/reco", format = "application/json", data = "<medium>")]
async fn reco(jwt: JWT, medium: Json<Medium>) -> Result<String, Status> {
    reco_worker(jwt.claims.user_id, medium.0).await
}

#[options("/rate_reco")]
fn _rate_reco() {}

#[post("/rate_reco", format = "application/json", data = "<req>")]
async fn rate_reco(jwt: JWT, req: Json<RateRecoRequest>) -> Result<String, Status> {
    update_user_rating(
        &establish_connection(DatabaseKind::PROD).map_err(|_| Status::InternalServerError)?, 
        jwt.claims.user_id, req.oeuvre_id, req.rating
    ).map_err(|_| Status::InternalServerError)?;
    reco_worker(jwt.claims.user_id, req.medium).await
}

#[options("/rate")]
fn _rate() {}

#[post("/rate", format = "application/json", data = "<req>")]
fn rate(jwt: JWT, req: Json<RateRequest>) -> Result<(), Status> {
    let conn = &establish_connection(DatabaseKind::PROD).map_err(|_| Status::InternalServerError)?;
    update_user_rating(&conn, jwt.claims.user_id, req.oeuvre_id, req.rating)
        .map_err(|_| Status::InternalServerError)?;
    Ok(())
}

#[options("/unrate")]
fn _unrate() {}

#[post("/unrate", format = "application/json", data = "<oeuvre_id>")]
fn unrate(jwt: JWT, oeuvre_id: Json<i32>) -> Result<(), Status> {
    let conn = establish_connection(DatabaseKind::PROD).map_err(|_| Status::InternalServerError)?;
    remove_user_rating(&conn, jwt.claims.user_id, oeuvre_id.0)
        .map_err(|_| Status::InternalServerError)?;
    Ok(())
}

#[options("/rated")]
fn _rated() {}

#[get("/rated")]
fn rated(jwt: JWT) -> Result<String, Status> {
    // TODO: return all the oeuvre the user has rated, so they can see and possibily edit the ratings
    let conn = establish_connection(DatabaseKind::PROD).map_err(|_| Status::InternalServerError)?;
    let oeuvres = get_rated_oeuvres(&conn, jwt.claims.user_id).map_err(|_| Status::InternalServerError)?;
    serde_json::to_string(&oeuvres).map_err(|_| Status::InternalServerError)
}

#[options("/search/<_>/<_>")]
fn _search() {}

#[get("/search/<medium>/<query>")]
fn search(jwt: JWT, medium: &str, query: &str) -> Result<String, Status> {
    // TODO: return oeuvres from the medium that match the query, also provide rating info if the user has rated it 
    let medium: Medium = serde_json::from_str(medium).map_err(|_| Status::BadRequest)?;
    todo!()
}

#[rocket::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _ = rocket::build()
        .attach(CORS)
        .mount("/", routes![
            sign_up, login, change_pwd, reco, rate, rate_reco, unrate, media, rated,
            _sign_up, _login, _change_pwd, _reco, _rate, _rate_reco, _unrate, _media, _rated,
        ])
        .launch()
        .await?;

    Ok(())
}