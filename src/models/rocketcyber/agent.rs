use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "snake_case", serialize = "snake_case"))]
pub struct RocketAgent {
    pub id: String,
    pub customer_id: i64,
    pub hostname: String,
    pub account_path: String,
    pub agent_version: String,
}
