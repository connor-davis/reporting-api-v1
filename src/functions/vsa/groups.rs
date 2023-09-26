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
pub struct VsaGroup {
    pub reverse_group_id: Option<String>,
    pub organization_name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VsaGroupsResponse {
    pub value: Vec<VsaGroup>,
}

/**
Fetch the groups data from Kaseya VSA using the reqwest HTTP Client crate.
*/
pub async fn groups() -> Vec<VsaGroup> {
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
        .get("https://vsa.thusa.co.za/api/odata/1.0/MachineGroups")
        .bearer_auth(api_token)
        .send()
        .await
        .expect("Failed to retrieve Kaseya VSA groups data.");

    let body = response
        .json::<VsaGroupsResponse>()
        .await
        .expect("Failed to retrieve Kaseya VSA groups data.");

    body.value
}
