use std::time::Duration;

use anyhow::Result;
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;

use crate::functions::spanning::spanning_backups::{self, spanning_backups, SpanningBackupData};
use crate::models::spanning::spanning::SpanningUser;

pub async fn sync_spanning() -> Result<()> {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost".into());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&database_url)
        .await
        .expect("Failed to connect to postgres.");

    println!("Syncing Spanning backups.");

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

    println!(
        "Finished syncing Spanning backups. Inserted: {}, Skipped: {}",
        inserted, skipped
    );

    Ok(())
}
