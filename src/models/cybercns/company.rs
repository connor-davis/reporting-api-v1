use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CyberCompany {
    pub id: String,
    pub name: String,
}
