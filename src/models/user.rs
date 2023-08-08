use std::fmt::{Display, Formatter};



#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct User {
    pub id: u32,
    pub nick: String,
    pub password: String,
    pub role: Roles
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Roles {
    Admin,
    Mod,
    User
}
impl Roles {
    pub fn from_str(role: &str) -> Roles {
        match role {
            "Admin" => Roles::Admin,
            "Mod" => Roles::Mod,
            _ => Roles::User,
        }
    }
}

impl User {
    #[must_use] pub fn new(id: u32, nick: String, password: String, role: Roles) -> Self {
        Self {
            id,
            nick,
            password,
            role
        }
    }
}


impl Display for Roles {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Admin => write!(f, "admin"),
            Self::Mod => write!(f, "mod"),
            Self::User => write!(f, "user")
        }
    }
}