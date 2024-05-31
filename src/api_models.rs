use rocket::serde::Deserialize;

#[derive(Deserialize)]
pub struct CredentialRequest {
    pub username: String,
    pub password: String,
}