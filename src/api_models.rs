use rocket::serde::Deserialize;

#[derive(Deserialize)]
pub struct CredentialRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct RateRequest {
    pub oeuvre_id: i32,
    pub rating: i32,
}