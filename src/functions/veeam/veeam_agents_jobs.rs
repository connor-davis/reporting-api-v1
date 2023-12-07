use std::{collections::HashMap, time::Duration};

use anyhow::Result;
use chrono::{DateTime, Utc};
use dotenv::dotenv;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;

#[derive(Debug, Deserialize, Serialize)]
pub struct Tenant {
    pub id: i64,
    pub tenant_name: String,
    pub veeam_url: Option<String>,
    pub veeam_key: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "camelCase", serialize = "camelCase"))]
pub struct VeeamAgentJob {
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
    pub restore_points: Option<i64>,
    pub last_run: Option<DateTime<Utc>>,
    pub last_end_time: Option<DateTime<Utc>>,
    pub last_duration: Option<i64>,
    pub next_run: Option<DateTime<Utc>>,
    pub avg_duration: Option<i64>,
    pub backup_mode: Option<String>,
    pub target_type: Option<String>,
    pub is_enabled: Option<bool>,
    pub schedule_type: Option<String>,
    pub failure_message: Option<String>,
    pub backed_up_size: Option<i64>,
    pub company_name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VeeamPagingInfo {
    pub total: i32,
    pub count: i32,
    pub offset: i32,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "camelCase", serialize = "camelCase"))]
pub struct VeeamMeta {
    pub paging_info: VeeamPagingInfo,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VeeamAgentsJobsResponse {
    pub data: Vec<VeeamAgentJob>,
    pub meta: VeeamMeta,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "camelCase", serialize = "camelCase"))]
pub struct VeeamOrganization {
    pub instance_uid: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VeeamOrganizationsResponse {
    pub data: Vec<VeeamOrganization>,
    pub meta: VeeamMeta,
}

pub async fn veeam_agents_jobs() -> Result<Vec<VeeamAgentJob>> {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost".into());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&database_url)
        .await
        .expect("Failed to connect to postgres.");

    let tenants = sqlx::query_as!(
        Tenant,
        "SELECT id, tenant_name, veeam_url, veeam_key FROM tenants WHERE veeam_url IS NOT NULL AND veeam_key IS NOT NULL;"
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to get tenants from postgres.");

    let mut veeam_agent_jobs: Vec<VeeamAgentJob> = Vec::new();

    for tenant in tenants {
        let veeam_organizations = get_veeam_organizations(
            format!("{}/organizations", tenant.veeam_url.clone().unwrap()),
            tenant.tenant_name.clone(),
            tenant.veeam_key.clone().unwrap(),
        )
        .await
        .expect("Failed to get Veeam Organizations.");

        let mut organizations_map: HashMap<String, String> = HashMap::new();

        for organization in veeam_organizations {
            organizations_map.insert(
                organization.instance_uid.clone().unwrap(),
                organization.name.clone().unwrap(),
            );
        }

        let veeam_windows_jobs = get_veeam_agents_jobs(
            format!(
                "{}/infrastructure/backupAgents/windows/jobs",
                tenant.veeam_url.clone().unwrap()
            ),
            tenant.tenant_name.clone(),
            tenant.veeam_key.clone().unwrap(),
            "Windows".to_string(),
        )
        .await
        .expect("Failed to get Veeam Windows Jobs.");

        for windows_job in veeam_windows_jobs {
            let mut new_backup = windows_job;

            new_backup.company_name = Some(
                organizations_map
                    .get(new_backup.organization_uid.clone().unwrap().as_str())
                    .unwrap_or(&tenant.tenant_name.clone())
                    .clone(),
            );

            veeam_agent_jobs.push(new_backup);
        }

        let veeam_linux_jobs = get_veeam_agents_jobs(
            "https://veeam.thusacloud.co.za:1280/api/v3/infrastructure/backupAgents/linux/jobs"
                .to_string(),
            tenant.tenant_name.clone(),
            tenant.veeam_key.clone().unwrap(),
            "Linux".to_string(),
        )
        .await
        .expect("Failed to get Veeam Linux Jobs.");

        for linux_job in veeam_linux_jobs {
            let mut new_backup = linux_job;

            new_backup.company_name = Some(
                organizations_map
                    .get(new_backup.organization_uid.clone().unwrap().as_str())
                    .unwrap()
                    .clone(),
            );

            veeam_agent_jobs.push(new_backup);
        }

        let veeam_mac_jobs = get_veeam_agents_jobs(
            "https://veeam.thusacloud.co.za:1280/api/v3/infrastructure/backupAgents/mac/jobs"
                .to_string(),
            tenant.tenant_name.clone(),
            tenant.veeam_key.clone().unwrap(),
            "Mac".to_string(),
        )
        .await
        .expect("Failed to get Veeam Mac Jobs.");

        for mac_job in veeam_mac_jobs {
            let mut new_backup = mac_job;

            new_backup.company_name = Some(
                organizations_map
                    .get(new_backup.organization_uid.clone().unwrap().as_str())
                    .unwrap()
                    .clone(),
            );

            veeam_agent_jobs.push(new_backup);
        }
    }

    println!("Agent Jobs: {}", veeam_agent_jobs.len());

    Ok(veeam_agent_jobs)
}

pub async fn get_veeam_organizations(
    url: String,
    tenant_name: String,
    veeam_key: String,
) -> Result<Vec<VeeamOrganization>> {
    println!("Fetching Veeam Organizations for {}", tenant_name);

    let client = Client::builder()
        .http1_title_case_headers()
        .build()
        .expect("Failed to create reqwest client.");

    let response = client
        .get(url)
        .header("x-api-version", "1.0-rev0")
        .bearer_auth(&veeam_key)
        .send()
        .await
        .expect("Failed to get response from Veeam Backup API");

    let body = response.json::<VeeamOrganizationsResponse>().await.unwrap();

    Ok(body.data)
}

pub async fn get_veeam_agents_jobs(
    url: String,
    tenant_name: String,
    veeam_key: String,
    job_platform: String,
) -> Result<Vec<VeeamAgentJob>> {
    println!("Fetching Veeam {} Jobs for {}", job_platform, tenant_name);

    let initial_offset = 0;
    let initial_limit = 1;

    let client = Client::builder()
        .http1_title_case_headers()
        .build()
        .expect("Failed to create reqwest client.");

    let query_url = url.clone();

    let response = client
        .get(format!(
            "{}?offset={}limit={}",
            query_url, initial_offset, initial_limit
        ))
        .header("x-api-version", "1.0-rev0")
        .bearer_auth(&veeam_key)
        .send()
        .await
        .expect("Failed to get response from Veeam Backup API");

    let body = response.json::<VeeamAgentsJobsResponse>().await.unwrap();

    let remote_offset = body.meta.paging_info.offset;
    let remote_limit = 100;
    let remote_total = body.meta.paging_info.total;

    let pages = (remote_total as f32 / remote_limit as f32).ceil() as i32;

    println!("Pages: {}", pages);

    let mut veeam_agents_jobs: Vec<VeeamAgentJob> = Vec::new();

    for page in 0..pages {
        let response = client
            .get(format!(
                "{}?offset={}limit={}",
                query_url,
                remote_offset + page * remote_limit,
                remote_limit
            ))
            .header("x-api-version", "1.0-rev0")
            .bearer_auth(&veeam_key)
            .send()
            .await
            .expect("Failed to get response from Veeam Backup API");

        let body = response.json::<VeeamAgentsJobsResponse>().await.unwrap();

        println!("Page: {}", page);

        veeam_agents_jobs.extend(body.data);
    }

    Ok(veeam_agents_jobs)
}
