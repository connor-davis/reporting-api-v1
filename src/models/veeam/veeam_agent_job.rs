use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct VeeamAgentJob {
    pub id: i32,
    pub instance_uid: Option<String>,
    pub backup_agent_uid: Option<String>,
    pub organization_uid: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub config_uid: Option<String>,
    pub system_type: Option<String>,
    pub backup_policy_uid: Option<String>,
    pub backup_policy_failure_message: Option<String>,
    pub status: Option<String>,
    pub operation_mode: Option<String>,
    pub destination: Option<String>,
    pub restore_points: Option<i32>,
    pub last_run: Option<DateTime<Utc>>,
    pub last_end_time: Option<DateTime<Utc>>,
    pub last_duration: Option<i32>,
    pub next_run: Option<DateTime<Utc>>,
    pub avg_duration: Option<i32>,
    pub backup_mode: Option<String>,
    pub target_type: Option<String>,
    pub is_enabled: Option<bool>,
    pub schedule_type: Option<String>,
    pub failure_message: Option<String>,
    pub backed_up_size: Option<i64>,
    pub company_name: Option<String>,
}
