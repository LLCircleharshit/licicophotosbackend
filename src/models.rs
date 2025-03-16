// models.rs
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum RequestData {
    Auth { username: String, password: String, operation: String },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseData {
    pub message: String,
    pub username: String,
    pub password: String,
}
