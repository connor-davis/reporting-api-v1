use std::env;

use anyhow::Error;
use dotenv::dotenv;
use math::round::ceil;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub struct CyberVulnerabilityAsset {
    pub id: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub struct CyberVulnerabilityCompany {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CyberVulnerabilityScore {
    pub base_score: Option<f64>,
    pub impact_score: Option<f64>,
    pub exploit_score: Option<f64>,
    pub cvss_score: Option<f64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CyberVulnerability {
    #[serde(rename = "_id")]
    pub _id: String,
    pub title: Option<String>,
    pub severity: Option<String>,
    pub vector: Option<String>,
    pub product: Vec<String>,
    pub score: Option<CyberVulnerabilityScore>,
    #[serde(rename = "companyRef")]
    pub company_ref: CyberVulnerabilityCompany,
    #[serde(rename = "assetRef")]
    pub asset_ref: CyberVulnerabilityAsset,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub struct CyberVulnerabilityResponse {
    pub data: Option<Vec<CyberVulnerability>>,
    pub total: Option<f64>,
}

pub async fn vulnerabilities() -> Result<Vec<CyberVulnerability>, Error> {
    dotenv().ok();

    let username = env::var("CYBER_CNS_CLIENT_ID").ok().unwrap();
    let password = env::var("CYBER_CNS_CLIENT_SECRET").ok().unwrap();

    let client = Client::builder().http1_title_case_headers().build()?;

    let mut response = client
        .get("https://portaleuwest2.mycybercns.com/api/vulnerability?limit=1")
        .header("customerid", "clay")
        .header("User-Agent", "ra-v1")
        .basic_auth(&username, Some(&password))
        .send()
        .await?;

    let mut body = response
        .json::<CyberVulnerabilityResponse>()
        .await
        .expect("Failed to retrieve CyberCNS assets.");

    let total_assets = body.total.ok_or(0);

    println!(
        "Found {} vulnerabilities from CyberCNS.",
        total_assets.unwrap()
    );

    let pages = ceil(total_assets.unwrap() as f64 / 100 as f64, -2) as i64;
    let mut current_page = 0;

    println!("Pages: {:?}", pages);

    let mut vulnerabilities: Vec<CyberVulnerability> = Vec::new();

    while current_page < pages {
        response = client
            .get(format!(
                "https://portaleuwest2.mycybercns.com/api/vulnerability?skip={}",
                current_page * 100
            ))
            .header("customerid", "clay")
            .header("User-Agent", "ra-v1")
            .basic_auth(&username, Some(&password))
            .send()
            .await?;

        body = response
            .json::<CyberVulnerabilityResponse>()
            .await
            .expect("Failed to retrieve CyberCNS assets.");

        let pages_vulnerabilities = body.data.unwrap();
        let vulnerabilities_count = pages_vulnerabilities.len();

        if vulnerabilities_count == 0 {
            break;
        }

        for vulnerability in pages_vulnerabilities {
            vulnerabilities.push(vulnerability)
        }

        println!(
            "Page: {}, Vulnerabilities: {}",
            current_page, vulnerabilities_count
        );

        current_page += 1;
    }

    Ok(vulnerabilities)
}
