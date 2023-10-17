use std::env;

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
pub struct VsaDevice {
    pub agent_id: Option<String>,
    pub system_serial_number: Option<String>,
    pub bios_release_date: Option<String>,
    pub cpu_speed: Option<f64>,
    pub cpu_count: Option<f64>,
    pub ram_size_in_mbytes: Option<f64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VsaDevicesResponse {
    pub value: Vec<VsaDevice>,
}

/**
Fetch the anti-virus data from Kaseya VSA using the reqwest HTTP Client crate.
*/
pub async fn devices() -> Vec<VsaDevice> {
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

    let response = client
        .get("https://vsa.thusa.co.za/api/odata/1.0/AuditMachineSummary")
        .bearer_auth(api_token)
        .send()
        .await
        .expect("Failed to retrieve Kaseya VSA devices data.");

    let body = response
        .json::<VsaDevicesResponse>()
        .await
        .expect("Failed to retrieve Kaseya VSA devices data.");

    body.value
}
