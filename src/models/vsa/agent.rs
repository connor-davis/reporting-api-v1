use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct VsaAgent {
    pub id: String,
    pub agent_name: Option<String>,
    pub computer_name: Option<String>,
    pub ip_address: Option<String>,
    pub anti_virus: Option<bool>,
    pub system_serial_number: Option<String>,
    pub system_age: Option<String>,
    pub free_space_in_gbytes: Option<f64>,
    pub used_space_in_gbytes: Option<f64>,
    pub total_size_in_gbytes: Option<f64>,
    pub group_id: Option<String>,
    pub organization_name: Option<String>,
    pub os_name: Option<String>,
    pub total_patches: Option<f64>,
    pub installed_patches: Option<f64>,
    pub last_patch: Option<DateTime<Utc>>,
    pub next_patch: Option<DateTime<Utc>>,
}
