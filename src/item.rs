use serde::{Deserialize, Serialize};

#[serde(rename_all = "camelCase")]
#[derive(Serialize, Deserialize, Debug)]
pub struct Item {
    pub id: String,
    pub title: String,
    pub fields: Vec<Field>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Field {
    Username,
    Password,
    Totp,
    Other(String),
}
