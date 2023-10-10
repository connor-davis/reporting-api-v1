use std::env;

use chrono::{DateTime, Utc};
use dotenv::dotenv;
use reqwest::Client;
use serde::{Deserialize, Serialize};

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
#[serde(rename_all(deserialize = "PascalCase", serialize = "PascalCase"))]
pub struct VsaPatch {
    pub agent_id: String,
    pub total_patches_reported_by_scan: f64,
    pub installed_patches_reported_by_scan: f64,
    pub last_patch_scan_date: Option<DateTime<Utc>>,
    pub next_patch_scan_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VsaPatchResponse {
    pub value: Vec<VsaPatch>,
}

/**
Fetch the patching data from Kaseya VSA using the reqwest HTTP Client crate.
*/
pub async fn patches() -> Vec<VsaPatch> {
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

    let vsa_patches = get_patches(api_token).await;

    vsa_patches
}

async fn get_patches(api_token: &String) -> Vec<VsaPatch> {
    let client = Client::builder()
        .http1_title_case_headers()
        .build()
        .expect("Failed to create new reqwest client.");

    let response = client
        .get("https://vsa.thusa.co.za/api/odata/1.0/PatchAgentStatuses")
        .bearer_auth(api_token)
        .send()
        .await
        .expect("Failed to retrieve Kaseya VSA antivirus data.");

    let body = response
        .json::<VsaPatchResponse>()
        .await
        .expect("Failed to retrieve security products for Kaseya VSA.");

    body.value
}
