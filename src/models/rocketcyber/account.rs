use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "snake_case", serialize = "snake_case"))]
pub struct RocketAccount {
    pub id: i32,
    pub account_id: i64,
    pub account_name: String,
    pub account_path: String,
    pub status: String,
}
