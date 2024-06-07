use rocket::serde::Deserialize;
use serde::Serialize;

use crate::{AppRating, Medium, Oeuvre, RatingOn100};

#[derive(Deserialize)]
pub struct CredentialRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    pub username: String,
    pub old_password: String,
    pub new_password: String,
}

#[derive(Deserialize)]
pub struct RateRequest {
    pub oeuvre_id: i32,
    pub rating: AppRating,
}

#[derive(Deserialize)]
pub struct RateRecoRequest {
    pub oeuvre_id: i32,
    pub rating: AppRating,
    pub medium: Medium
}

#[derive(Serialize)]
pub struct ProfileResponse {
    pub oeuvres: Vec<Oeuvre>,
    pub similarity: RatingOn100,
}

#[derive(Serialize)]
pub struct InfoResponse {
    pub media: Vec<Medium>,
    pub usename: String
}