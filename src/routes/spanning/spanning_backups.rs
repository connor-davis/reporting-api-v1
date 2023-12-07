use axum::{extract::State, response::IntoResponse, Json};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;

use crate::{
    functions::spanning::spanning_backups::{self, spanning_backups, SpanningBackupData},
    models::spanning::spanning::{SpanningBackupSummary, SpanningUser},
};

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

pub async fn index(State(pool): State<PgPool>) -> impl IntoResponse {
    let mut spanning_backups: Vec<SpanningUserBackup> = Vec::new();

    let backups: Vec<SpanningUser> =
        sqlx::query_as!(SpanningUser, "SELECT * FROM spanning_backups;")
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

    Json(spanning_backups)
}

pub async fn import(State(pool): State<PgPool>) -> impl IntoResponse {
    let backups = spanning_backups().await;

    let mut skipped = 0;
    let mut inserted = 0;

    match backups {
        Ok(backups) => {
            for backup in backups {
                let existing_backup = sqlx::query_as!(
                    SpanningUser,
                    "SELECT * FROM spanning_backups WHERE email = $1;",
                    backup.email
                )
                .fetch_optional(&pool)
                .await
                .expect("Failed to get user from postgres.");

                match existing_backup {
                    Some(_) => {
                        skipped += 1;
                    }
                    None => {
                        let result = sqlx::query!(
                            "INSERT INTO spanning_backups (user_principal_name, user_display_name, email, ms_id, assigned, is_admin, is_deleted, company_name) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id;",
                            backup.user_principal_name.unwrap_or("N/A".to_string()),
                            backup.user_display_name.unwrap_or("N/A".to_string()),
                            backup.email.unwrap_or("N/A".to_string()),
                            backup.ms_id.unwrap_or("N/A".to_string()),
                            backup.assigned.unwrap_or(false),
                            backup.is_admin.unwrap_or(false),
                            backup.is_deleted.unwrap_or(false),
                            backup.company_name.unwrap_or("N/A".to_string())
                        )
                        .fetch_one(&pool)
                        .await;

                        match result {
                            Ok(result) => {
                                for backup_summary in backup.backup_summary.unwrap() {
                                    let backup_summary_backup = backup_summary.backup.unwrap_or(
                                        spanning_backups::SpanningBackup {
                                            total: Some("0".to_string()),
                                            partial: Some(0),
                                            failed: Some(0),
                                            successful: Some(0),
                                            data: Some(SpanningBackupData {
                                                created: Some(0),
                                                deleted: Some(0),
                                                failed: Some(0),
                                                total: Some(0),
                                                attempts: Some(0),
                                            }),
                                        },
                                    );
                                    let backup_data =
                                        backup_summary_backup.data.unwrap_or(SpanningBackupData {
                                            created: Some(0),
                                            deleted: Some(0),
                                            failed: Some(0),
                                            total: Some(0),
                                            attempts: Some(0),
                                        });

                                    sqlx::query_as!(
                                        SpanningBackupSummary,
                                        "INSERT INTO spanning_backups_summaries (backup, date, backup_type, total, partial, failed, successful, data_created, data_deleted, data_failed, data_total, data_attempts) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12);",
                                        result.id,
                                        backup_summary.date.unwrap_or("N/A".to_string()),
                                        backup_summary.backup_type.unwrap_or("N/A".to_string()),
                                        backup_summary_backup.total.unwrap_or("0".to_string()),
                                        backup_summary_backup.partial.unwrap_or(0),
                                        backup_summary_backup.failed.unwrap_or(0),
                                        backup_summary_backup.successful.unwrap_or(0),
                                        backup_data.created.unwrap_or(0),
                                        backup_data.deleted.unwrap_or(0),
                                        backup_data.failed.unwrap_or(0),
                                        backup_data.total.unwrap_or(0),
                                        backup_data.attempts.unwrap_or(0)
                                    )
                                    .execute(&pool)
                                    .await
                                    .expect("Failed to insert backup summary into postgres database.");
                                }

                                inserted += 1;
                            }
                            Err(error) => println!("Failed to insert Spanning backup: {}", error),
                        }
                    }
                }
            }
        }
        Err(error) => println!("Failed to sync Spanning backups: {}", error),
    }

    Json(json!({
        "status": StatusCode::OK.as_u16(),
        "inserted": inserted,
        "skipped": skipped
    }))
}
