use std::{collections::{HashMap}};

use serde::{Serialize, Deserialize};
use serde_inline_default::serde_inline_default;
use serde_json::json;
use tokio::sync::MutexGuard;

use crate::{routes::ws::commands::pokemon_info::fetch_pokemon_info, helpers::utils::create_jwt::{create_jwt}};

use super::{user::{User}, login_response::LoginResponse};

#[serde_inline_default]
#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub command: Option<String>,
    pub arguments: Option<Vec<String>>,
    pub content: Option<String>
}

impl Message {

    #[must_use]
    pub fn has_enoughs_params(&self) -> bool{
        let Some(command) = &self.command else {
            return true;
        };
        let arguments = &self.arguments.clone().unwrap_or_default();

        match command.as_str() {
            "pokemon_info" => arguments.len() == 1,
            "message" => arguments.len() == 1,
            "nick" => arguments.len() == 1,
            "login" => arguments.len() == 2,
            _ => true
        }
    }

    fn require_be_logged(&self) -> bool {
        let Some(command) = &self.command else {
            return false;
        };

        match command.as_str() {
            "pokemon_info" => true,
            "message" => true,
            "nick" => false,
            "login" => false,
            "register" => false,
            _ => true
        }
    }


    pub async fn execute(&self, mut users: MutexGuard<'_, HashMap<String, User>>, jwt: &String) -> Option<String> {

        if self.require_be_logged() && jwt.is_empty() {
            return None;
        }

        if !self.has_enoughs_params() {
            return None;
        }

        let Some(command) = &self.command else {
            return None;
        };
        let Some(arguments) = &self.arguments.clone() else {
            return None;
        };

        match command.as_str() {
            "pokemon_info" => {
                let Some(pokemon) = fetch_pokemon_info(arguments[0].as_str()).await else {
                    return None;
                };
                serde_json::to_string(&pokemon).ok()
            },
            "message" => {
                let message = arguments[0].clone();
                Some(message)
            },
            "register" => {
                let username = arguments[0].clone();
                let password = arguments[1].clone();
                let id = users.len() + 1;
                let token = create_jwt(id as u32, &super::user::Roles::User).ok()?;
                users.insert(username.clone(), User::new(id as u32, username, password, super::user::Roles::User));

                Some(json!(&LoginResponse { token }).to_string())
            }
            "login" => {
                let username = arguments[0].clone();
                let password = arguments[1].clone();
                let Some(user) = users.get(&username) else {
                    return Some("Fallaste pa".to_string())
                };
                if password != user.password {
                    return Some("Fallaste pa".to_string())
                }
                let token = create_jwt(user.id, &user.role).ok()?;
                Some(json!(&LoginResponse { token }).to_string())
            },
            "nick" => {
                let nick = arguments[0].clone();

                // let decoded = decode::<Claims>(
                //     &jwt,
                //     &DecodingKey::from_secret(JWT_SECRET),
                //     &Validation::new(Algorithm::HS512),
                // )
                // .map_err(|_| reject::custom(Error::JWTTokenError))?;
    
                // if role == Role::Admin && Role::from_str(&decoded.claims.role) != Role::Admin {
                //     return Err(reject::custom(Error::NoPermissionError));
                // }


                Some(nick)
            }
            "algo" => todo!(),
            "otra cosa" => unimplemented!(),
            "Acá no llega" => unreachable!(),
            _ => None
        }
    }
}
