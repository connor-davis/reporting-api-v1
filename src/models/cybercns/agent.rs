use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CyberAgent {
    pub id: String,
    pub host_name: String,
}
