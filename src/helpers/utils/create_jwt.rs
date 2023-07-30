use chrono::Utc;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use color_eyre::{Result, Help};
use crate::models::{user::Roles, error::Error};

const BEARER: &str = "Bearer ";
const JWT_SECRET: &[u8] = b"secret";



pub fn create_jwt(uid: u32, role: &Roles) -> Result<String> {
    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::seconds(60))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: uid,
        role: role.to_string(),
        exp: expiration as usize,
    };
    let header = Header::new(Algorithm::HS512);
    encode(&header, &claims, &EncodingKey::from_secret(JWT_SECRET))
    .map_err(|_| Error::JWTTokenCreationError)
    .suggestion("Try again later")
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    sub: u32,
    role: String,
    exp: usize,
}
