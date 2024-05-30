use rocket::serde::Deserialize;
use rocket::serde::Serialize;
use rocket::Responder;
use crate::models::*;

#[derive(Serialize)]
pub enum ResponseBody {
    AuthToken(String),
    Reco(Oeuvre),
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Response {
    pub body: ResponseBody,
}

#[derive(Responder, Debug)]
pub enum NetworkResponse {
    #[response(status = 201)]
    Created(String),
    #[response(status = 400)]
    BadRequest(String),
    #[response(status = 401)]
    Unauthorized(String),
    #[response(status = 404)]
    NotFound(String),
    #[response(status = 409)]
    Conflict(String),
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub user_name: String,
    pub password: String,
}