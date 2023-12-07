use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct TenantExternalScanHostname {
    pub id: f64,
    pub tenant_id: f64,
    pub host_name: String,
}
