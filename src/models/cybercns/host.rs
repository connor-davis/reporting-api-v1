use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CyberHost {
    pub id: i64,
    pub host_name: String,
}
