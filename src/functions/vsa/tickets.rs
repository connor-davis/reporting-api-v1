use std::env;

use anyhow::Result;
use chrono::{DateTime, Utc};
use dotenv::dotenv;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "PascalCase", serialize = "PascalCase"))]
pub struct VsaAuthResult {
    pub api_token: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "PascalCase", serialize = "PascalCase"))]
pub struct VsaAuthResponse {
    pub result: Option<VsaAuthResult>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VsaTicket {
    pub partition_id: Option<String>,
    pub asset_agent_id: Option<String>,
    pub asset_machine_group_name: Option<String>,
    pub ticket_id: Option<String>,
    pub ticket_number: Option<String>,
    pub service_desk: Option<String>,
    pub summary: Option<String>,
    pub status: Option<String>,
    pub stage: Option<String>,
    pub priority: Option<String>,
    pub severity: Option<String>,
    pub category: Option<String>,
    pub resolution: Option<String>,
    pub resolution_description: Option<String>,
    pub submitter_type: Option<String>,
    pub submitter_name: Option<String>,
    pub submitter_email: Option<String>,
    pub submitter_phone: Option<String>,
    pub contact_name: Option<String>,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub assignee: Option<String>,
    pub owner: Option<String>,
    pub organization: Option<String>,
    pub creation_date_time: Option<DateTime<Utc>>,
    pub modification_date_time: Option<DateTime<Utc>>,
    pub closed_date_time: Option<DateTime<Utc>>,
    pub due_date: Option<DateTime<Utc>>,
    pub projected_date: Option<DateTime<Utc>>,
    pub locked_by: Option<String>,
    pub locked_on_date_time: Option<DateTime<Utc>>,
    pub source_type: Option<String>,
    pub last_public_update_date: Option<DateTime<Utc>>,
    pub resolution_date: Option<DateTime<Utc>>,
    pub policy: Option<String>,
    pub description: Option<String>,
    pub is_archived: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VsaTicketsResponse {
    pub status: u16,
    pub message: String,
    pub tickets: Vec<VsaTicket>,
}

// Get the database url from the environment
// Get the database pool
// Get the list of tenants from the database
pub async fn get_vsa_tickets() -> Result<Vec<VsaTicket>> {
    dotenv().ok();

    let username = env::var("VSA_USERNAME").ok().unwrap();
    let password = env::var("VSA_API_KEY").ok().unwrap();

    let client = Client::builder()
        .http1_title_case_headers()
        .build()
        .expect("Failed to create reqwest client.");

    let auth_response = client
        .get("https://vsa.thusa.co.za/api/v1.0/auth")
        .basic_auth(&username, Some(&password))
        .send()
        .await
        .expect("Failed to make reqwest to Kaseya VSA.");

    let auth_body = auth_response
        .json::<VsaAuthResponse>()
        .await
        .expect("Failed to retrieve authentication token from Kaseya VSA.");

    let api_token = &auth_body.result.unwrap().api_token.unwrap();

    let vsa_tickets_response = client
        .get("https://vsa.thusa.co.za/api/odata/1.0/ServiceDeskTickets")
        .bearer_auth(api_token)
        .send()
        .await
        .expect("Failed to make reqwest to Kaseya VSA.");

    let vsa_tickets_body = vsa_tickets_response
        .json::<Value>()
        .await
        .expect("Failed to get VSA tickets from api.");

    println!("{:?}", vsa_tickets_body);

    Ok(Vec::new())
}
