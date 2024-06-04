use rocket::serde::Deserialize;

use crate::AppRating;

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