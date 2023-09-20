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
}
