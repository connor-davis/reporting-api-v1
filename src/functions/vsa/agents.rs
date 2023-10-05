use std::env;

use anyhow::Error;
use dotenv::dotenv;
use math::round::ceil;
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
pub struct VsaAgentsCountResponse {
    pub total_records: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "PascalCase", serialize = "PascalCase"))]
pub struct VsaAgent {
    pub agent_id: String,
    pub agent_name: String,
    pub computer_name: String,
    #[serde(rename = "IPAddress")]
    pub ip_address: String,
    pub anti_virus: Option<bool>,
    pub system_serial_number: Option<String>,
    pub system_age: Option<String>,
    pub free_space_in_gbytes: Option<f64>,
    pub used_space_in_gbytes: Option<f64>,
    pub total_size_in_gbytes: Option<f64>,
    pub operating_system: Option<String>,
    #[serde(rename = "OSType")]
    pub operating_system_type: Option<String>,
    #[serde(rename = "OSInfo")]
    pub operating_system_info: Option<String>,
    pub machine_group: Option<String>,
    pub organization_name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "PascalCase", serialize = "PascalCase"))]
pub struct VsaAgentsResponse {
    pub result: Option<Vec<VsaAgent>>,
    pub total_records: Option<u64>,
}

pub async fn agents() -> Vec<VsaAgent> {
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

    let vsa_agents_count = agents_count(api_token)
        .await
        .expect("Failed to get Kaseya VSA agents count.");

    let vsa_agents = agents_paged(api_token, vsa_agents_count)
        .await
        .expect("Failed to get Kaseya VSA agents.");

    vsa_agents
}

async fn agents_count(api_token: &String) -> Result<u64, Error> {
    let client = Client::builder().http1_title_case_headers().build()?;

    let response = client
        .get("https://vsa.thusa.co.za/api/v1.0/assetmgmt/agents?$top=1")
        .bearer_auth(api_token)
        .send()
        .await?;

    let body = response.json::<VsaAgentsCountResponse>().await?;

    Ok(body.total_records.unwrap())
}

async fn agents_paged(api_token: &String, vsa_agents_count: u64) -> Result<Vec<VsaAgent>, Error> {
    let client = Client::builder().http1_title_case_headers().build()?;

    let count: f64 = vsa_agents_count as f64;
    let pages = ceil(count / 100.0, 0);
    let mut current_page: f64 = 0 as f64;

    let mut new_agents: Vec<VsaAgent> = Vec::new();

    while current_page < pages {
        println!("Fetching agents page {}/{}", current_page + 1.0, pages);

        let response = client
            .get(format!(
                "https://vsa.thusa.co.za/api/v1.0/assetmgmt/agents?$skip={}&$top=100",
                current_page * 100.0
            ))
            .bearer_auth(api_token)
            .send()
            .await?;

        let body = response.json::<VsaAgentsResponse>().await?;

        let body_agents = body.result.unwrap();

        for agent in body_agents {
            new_agents.push(agent)
        }

        current_page += 1.0;
    }

    Ok(new_agents)
}
