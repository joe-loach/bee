use serde::{Deserialize, Serialize};

use crate::database;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UserId(pub u64);

#[derive(Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub(crate) password_hash: String,
}

pub struct Get {
    pub username: String,
}

impl database::Query for Get {
    type Result = User;

    fn query(&self) -> &'static str {
        "SELECT * FROM users WHERE username = ?1"
    }

    fn bindings(&self) -> Vec<database::Binding> {
        vec![self.username.as_str().into()]
    }
}

pub struct Insert {
    pub username: String,
    pub password: String,
}

impl database::Query for Insert {
    type Result = ();

    fn query(&self) -> &'static str {
        "INSERT INTO users (username, password_hash) VALUES (?1, ?2)"
    }

    fn bindings(&self) -> Vec<database::Binding> {
        vec![self.username.as_str().into(), self.password.as_str().into()]
    }
}
