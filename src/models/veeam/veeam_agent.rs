use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "camelCase", serialize = "camelCase"))]
pub struct VeeamAgent {
    pub id: i32,
    pub instance_uid: Option<String>,
    pub agent_platform: Option<String>,
    pub status: Option<String>,
    pub management_agent_uid: Option<String>,
    pub site_uid: Option<String>,
    pub organization_uid: Option<String>,
    pub name: Option<String>,
    pub operation_mode: Option<String>,
    pub gui_mode: Option<String>,
    pub platform: Option<String>,
    pub version: Option<String>,
    pub activation_time: Option<DateTime<Utc>>,
    pub management_mode: Option<String>,
    pub installation_type: Option<String>,
    pub total_jobs_count: Option<i32>,
    pub running_jobs_count: Option<i32>,
    pub success_jobs_count: Option<i32>,
    pub company_name: Option<String>,
}