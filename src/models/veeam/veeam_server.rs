use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct VeeamServer {
    pub id: i32,
    pub instance_uid: String,
    pub name: String,
    pub organization_uid: String,
    pub location_uid: String,
    pub management_agent_uid: String,
    pub version: String,
    pub display_version: String,
    pub installation_uid: String,
    pub backup_server_role_type: String,
    pub status: String,
    pub company_name: String,
}
