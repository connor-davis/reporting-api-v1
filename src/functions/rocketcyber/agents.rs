use chrono::{DateTime, Utc};
use dotenv::dotenv;
use reqwest::{Client, Error};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub struct RocketAgent {
    pub id: Option<String>,
    pub customer_id: Option<i64>,
    pub hostname: Option<String>,
    pub platform: Option<String>,
    pub family: Option<String>,
    pub version: Option<String>,
    pub architecture: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub account_path: Option<String>,
    pub agent_version: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "camelCase", serialize = "camelCase"))]
pub struct RocketAgentResponse {
    pub data: Option<Vec<RocketAgent>>,
}

/*
Fetch the agents from RocketCyber using the reqwest HTTP Client crate.
*/
pub async fn agents() -> Result<Vec<RocketAgent>, Error> {
    dotenv().ok();

    let client = Client::builder().http1_title_case_headers().build()?;

    let access_token = std::env::var("ROCKET_CYBER_API_KEY").ok().unwrap();

    let response = client
        .get("https://api-eu.rocketcyber.com/v3/agents")
        .bearer_auth(access_token)
        .send()
        .await?;

    let body = response
        .json::<RocketAgentResponse>()
        .await
        .expect("Failed to retrieve Rocket Cyber agents.");

    Ok(body.data.unwrap())
}
