// inspired by: https://github.com/BrookJeynes/rust-blog-using-rocket/blob/route-authentication/application/src/auth.rs
// but simplified
use chrono::Utc;
use rocket::http::Status;
use rocket::request::{Outcome, Request, FromRequest};
use rocket::serde::{Deserialize, Serialize};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use std::env;
use dotenvy::dotenv;

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub user_id: i32,
    exp: usize
}

#[derive(Debug)]
pub struct JWT {
    pub claims: Claims
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for JWT {
    type Error = jsonwebtoken::errors::Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        fn is_valid(key: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
            Ok(decode_jwt(String::from(key))?)
        }

        match req.headers().get_one("authorization") {
            None => Outcome::Error((Status::Unauthorized, jsonwebtoken::errors::ErrorKind::InvalidToken.into())),
            Some(key) => match is_valid(key) {
                Ok(claims) => Outcome::Success(JWT {claims}),
                Err(err) => Outcome::Error((Status::Unauthorized, err)),
            },
        }
    }
}

fn decode_jwt(token: String) -> Result<Claims, jsonwebtoken::errors::Error> {
    dotenv().ok();

    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set.");
    let token = token.trim_start_matches("Bearer").trim();

    decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS512),
    ).map(|token| token.claims)
}

pub fn create_jwt(id: i32) -> Result<String, jsonwebtoken::errors::Error> {
    dotenv().ok();

    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set.");
    // the token lasts half a year because stakes are low
    let expiration = Utc::now().checked_add_signed(chrono::Duration::weeks(26)).expect("Invalid timestamp").timestamp();
    let claims = Claims {
        user_id: id,
        exp: expiration as usize
    };

    let header = Header::new(Algorithm::HS512);

    encode(&header, &claims, &EncodingKey::from_secret(secret.as_bytes()))
}