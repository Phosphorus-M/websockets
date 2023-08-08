use color_eyre::{Result, Help};
use headers::HeaderMap;

use crate::models::{user::Roles, error::Error};

use super::create_jwt::{Claims, JWT_SECRET, BEARER};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
pub async fn authorize((role, headers): (Roles, HeaderMap)) -> Result<u32> {
    match jwt_from_header(headers) {
        Ok(jwt) => {


            let decoded = decode::<Claims>(
                &jwt,
                &DecodingKey::from_secret(JWT_SECRET),
                &Validation::new(Algorithm::HS512),
            )
            .map_err(|_| Error::JWTTokenError)?;

            if role == Roles::Admin && Roles::from_str(&decoded.claims.role) != Roles::Admin {
                return Err(Error::NoPermissionError).suggestion("do something");
            }

            Ok(decoded.claims.sub)
        }
        Err(e) => return Err(e)
    }
}

fn jwt_from_header(headers: HeaderMap) -> Result<String>{
    let header = match headers.get("authorization") {
        Some(v) => v,
        None => Err(Error::NoAuthHeaderError)?,
    };
    let auth_header = match std::str::from_utf8(header.as_bytes()) {
        Ok(v) => v,
        Err(_) => Err(Error::NoAuthHeaderError)?,
    };
    if !auth_header.starts_with(BEARER) {
        Err(Error::InvalidAuthHeaderError)?;
    }
    Ok(auth_header.trim_start_matches(BEARER).to_owned())
}