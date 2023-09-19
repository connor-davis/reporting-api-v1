use std::env;

use dotenv::dotenv;
use reqwest::{Client, Error};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CyberAgent {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub host_name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "camelCase", serialize = "camelCase"))]
pub struct CyberAgentResponse {
    pub data: Option<Vec<CyberAgent>>,
    pub total: Option<u64>,
}

pub async fn agents() -> Result<Vec<CyberAgent>, Error> {
    dotenv().ok();

    let username = env::var("CYBER_CNS_CLIENT_ID").ok().unwrap();
    let password = env::var("CYBER_CNS_CLIENT_SECRET").ok().unwrap();

    let client = Client::builder().http1_title_case_headers().build()?;

    let mut response = client
        .get("https://portaleuwest2.mycybercns.com/api/agent?limit=1")
        .header("customerid", "clay")
        .header("User-Agent", "ra-v1")
        .basic_auth(&username, Some(&password))
        .send()
        .await?;

    let mut body = response
        .json::<CyberAgentResponse>()
        .await
        .expect("Failed to retrieve CyberCNS agents.");

    let total_agents = body
        .total
        .expect("Failed to retrieve total CyberCNS agents.");

    response = client
        .get(format!(
            "https://portaleuwest2.mycybercns.com/api/agent?limit={}",
            total_agents
        ))
        .header("customerid", "clay")
        .header("User-Agent", "ra-v1")
        .basic_auth(&username, Some(&password))
        .send()
        .await?;

    body = response
        .json::<CyberAgentResponse>()
        .await
        .expect("Failed to retrieve CyberCNS agents.");

    Ok(body.data.unwrap())
}
