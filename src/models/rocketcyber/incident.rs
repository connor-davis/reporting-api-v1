use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "snake_case", serialize = "snake_case"))]
pub struct RocketIncident {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub remediation: String,
    pub resolved_at: Option<DateTime<Utc>>,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub status: String,
    pub account_id: i64,
    pub event_count: i64,
}
