use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CyberAsset {
    pub id: String,
    pub host: Option<i32>,
    pub security_report_card: Option<i32>,
    pub company: Option<String>,
}
