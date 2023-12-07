use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;

use crate::models::spanning::spanning::{SpanningBackupSummary, SpanningUser};

#[derive(Debug, Serialize, Deserialize)]
pub struct Tenant {
    pub id: i64,
    pub spanning_name: Option<String>,
    pub spanning_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpanningData {
    pub created: Option<i64>,
    pub deleted: Option<i64>,
    pub failed: Option<i64>,
    pub total: Option<i64>,
    pub attempts: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpanningBackup {
    pub total: Option<String>,
    pub partial: Option<i64>,
    pub failed: Option<i64>,
    pub successful: Option<i64>,
    pub data: Option<SpanningData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpanningSummary {
    pub date: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub backup_type: Option<String>,
    pub backup: Option<SpanningBackup>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase", serialize = "camelCase"))]
pub struct SpanningUserBackup {
    pub user_principal_name: Option<String>,
    pub user_display_name: Option<String>,
    pub email: Option<String>,
    pub ms_id: Option<String>,
    pub assigned: Option<bool>,
    pub is_admin: Option<bool>,
    pub is_deleted: Option<bool>,
    pub backup_summary: Option<Vec<SpanningSummary>>,
    pub company_name: Option<String>,
}

pub async fn index(
    Query(params): Query<Vec<(String, String)>>,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    let tenant = &params[0].1;

    let mut spanning_backups: Vec<SpanningUserBackup> = Vec::new();

    let backups: Vec<SpanningUser> =
        sqlx::query_as!(SpanningUser, "SELECT * FROM spanning_backups WHERE similarity(LOWER(company_name), LOWER($1)) >= 0.6 ORDER BY email;", tenant)
            .fetch_all(&pool)
            .await
            .expect("Failed to get vsa agents from postgres.");

    for backup in backups {
        let backup_summaries: Vec<SpanningBackupSummary> = sqlx::query_as!(
            SpanningBackupSummary,
            "SELECT * FROM spanning_backups_summaries WHERE backup = $1;",
            backup.id
        )
        .fetch_all(&pool)
        .await
        .expect("Failed to get vsa agents from postgres.");

        let mut spanning_summaries: Vec<SpanningSummary> = Vec::new();

        for summary in backup_summaries {
            spanning_summaries.push(SpanningSummary {
                date: Some(summary.date),
                backup_type: Some(summary.backup_type),
                backup: Some(SpanningBackup {
                    total: Some(summary.total),
                    partial: Some(summary.partial),
                    failed: Some(summary.failed),
                    successful: Some(summary.successful),
                    data: Some(SpanningData {
                        created: Some(summary.data_created),
                        deleted: Some(summary.data_deleted),
                        failed: Some(summary.data_failed),
                        total: Some(summary.data_total),
                        attempts: Some(summary.data_attempts),
                    }),
                }),
            })
        }

        spanning_backups.push(SpanningUserBackup {
            user_principal_name: Some(backup.user_principal_name),
            user_display_name: Some(backup.user_display_name),
            email: Some(backup.email),
            ms_id: Some(backup.ms_id),
            assigned: Some(backup.assigned),
            is_admin: Some(backup.is_admin),
            is_deleted: Some(backup.is_deleted),
            backup_summary: Some(spanning_summaries),
            company_name: Some(backup.company_name),
        })
    }

    Json(json!({
        "status": StatusCode::OK.as_u16(),
        "tenant": tenant,
        "results": spanning_backups,
    }))
}
