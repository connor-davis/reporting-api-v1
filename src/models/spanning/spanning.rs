use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SpanningBackupSummary {
    pub id: i32,
    pub date: String,
    pub backup_type: String,
    pub total: String,
    pub partial: i64,
    pub failed: i64,
    pub successful: i64,
    pub data_created: i64,
    pub data_deleted: i64,
    pub data_failed: i64,
    pub data_total: i64,
    pub data_attempts: i64,
    pub backup: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpanningUser {
    pub id: i32,
    pub user_principal_name: String,
    pub user_display_name: String,
    pub email: String,
    pub ms_id: String,
    pub assigned: bool,
    pub is_admin: bool,
    pub is_deleted: bool,
    pub company_name: String,
}
