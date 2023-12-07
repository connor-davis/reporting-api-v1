use std::time::Duration;

use anyhow::Error;
use dotenv::dotenv;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;

#[derive(Debug, Serialize, Deserialize)]
pub struct Tenant {
    pub id: i64,
    pub tenant_name: Option<String>,
    pub spanning_name: Option<String>,
    pub spanning_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpanningBackupData {
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
    pub data: Option<SpanningBackupData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpanningBackupSummary {
    pub date: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub backup_type: Option<String>,
    pub backup: Option<SpanningBackup>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase", serialize = "camelCase"))]
pub struct SpanningUser {
    pub user_principal_name: Option<String>,
    pub user_display_name: Option<String>,
    pub email: Option<String>,
    pub ms_id: Option<String>,
    pub assigned: Option<bool>,
    pub is_admin: Option<bool>,
    pub is_deleted: Option<bool>,
    pub backup_summary: Option<Vec<SpanningBackupSummary>>,
    pub company_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase", serialize = "camelCase"))]
pub struct SpanningResponse {
    pub next_link: Option<String>,
    pub users: Vec<SpanningUser>,
}

pub async fn spanning_backups() -> Result<Vec<SpanningUser>, Error> {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost".into());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&database_url)
        .await
        .expect("Failed to connect to postgres.");

    let tenants = sqlx::query_as!(Tenant, "SELECT id, tenant_name, spanning_name, spanning_key FROM tenants WHERE spanning_name IS NOT NULL AND spanning_key IS NOT NULL;")
        .fetch_all(&pool)
        .await
        .expect("Failed to get tenants from postgres.");

    let mut spanning_backups: Vec<SpanningUser> = Vec::new();

    for tenant in tenants {
        let tenant_name = tenant.tenant_name;
        let spanning_name = tenant.spanning_name.unwrap();
        let spanning_key = tenant.spanning_key.unwrap();

        let new_backups = get_spanning_backups(
            "https://o365-api-eu.spanningbackup.com/external/users".to_string(),
            tenant_name.clone().unwrap(),
            spanning_name,
            spanning_key,
        )
        .await;

        match new_backups {
            Ok(new_backups) => {
                let mut backups: Vec<SpanningUser> = Vec::new();

                for mut new_backup in new_backups {
                    new_backup.company_name = tenant_name.clone();

                    backups.push(new_backup);
                }

                spanning_backups.extend(backups)
            }
            Err(error) => {
                println!(
                    "Failed to get response from Spanning Backup API: {:?}",
                    error
                );

                continue;
            }
        }
    }

    let mut valid_backups: Vec<SpanningUser> = Vec::new();

    for backup in spanning_backups {
        if backup.assigned.unwrap() {
            valid_backups.push(backup);
        }
    }

    println!("Spanning Backups: {}", valid_backups.len());

    Ok(valid_backups)
}

pub async fn get_spanning_backups(
    url: String,
    tenant_name: String,
    spanning_name: String,
    spanning_key: String,
) -> Result<Vec<SpanningUser>, Error> {
    println!("Fetching Spanning Backups for {}", tenant_name);

    let client = Client::builder()
        .http1_title_case_headers()
        .build()
        .expect("Failed to create reqwest client.");

    let mut backups: Vec<SpanningUser> = Vec::new();
    let mut query_url = url.clone();

    let mut response = client
        .get(query_url)
        .basic_auth(&spanning_name, Some(&spanning_key))
        .send()
        .await
        .expect("Failed to get response from Spanning Backup API");

    let mut body = response.json::<SpanningResponse>().await.unwrap();

    backups.extend(body.users);

    while body.next_link.is_some() {
        query_url = body.next_link.unwrap();

        if query_url.len() == 0 {
            println!("Finished fetching Spanning Backups for {}", tenant_name);

            break;
        }

        println!(
            "Fetching Spanning Backups for {} with {}",
            tenant_name, query_url
        );

        response = client
            .get(query_url)
            .basic_auth(&spanning_name, Some(&spanning_key))
            .send()
            .await
            .expect("Failed to get response from Spanning Backup API");

        body = response.json::<SpanningResponse>().await.unwrap();

        backups.extend(body.users);

        if body.next_link.is_none() {
            println!("Finished fetching Spanning Backups for {}", tenant_name);

            break;
        }
    }

    Ok(backups)
}
