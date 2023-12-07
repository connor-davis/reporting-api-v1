use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct VeeamServerJob {
    pub id: i32,
    pub instance_uid: String,
    pub name: String,
    pub backup_server_uid: String,
    pub location_uid: String,
    pub site_uid: String,
    pub organization_uid: String,
    pub status: String,
    pub r#type: String,
    pub last_run: DateTime<Utc>,
    pub last_end_time: DateTime<Utc>,
    pub last_duration: i64,
    pub processing_rate: f64,
    pub avg_duration: i64,
    pub transferred_data: i64,
    pub bottleneck: String,
    pub is_enabled: bool,
    pub schedule_type: String,
    pub failure_message: String,
    pub target_type: String,
    pub destination: String,
    pub retention_limit: i64,
    pub retention_limit_type: String,
    pub is_gfs_option_enabled: bool,
    pub company_name: String,
}
