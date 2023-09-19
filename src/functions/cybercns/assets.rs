use std::env;

use dotenv::dotenv;
use reqwest::{Client, Error};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CyberHost {
    pub host_name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub struct CyberSecurityReportCardEvidence {
    pub anti_virus: Option<String>,
    pub local_firewall: Option<String>,
    pub insecure_listening_ports: Option<String>,
    pub failed_login: Option<String>,
    pub network_vulnerabilities: Option<String>,
    pub system_aging: Option<String>,
    #[serde(rename = "supportedOS", skip_serializing_if = "Option::is_none")]
    pub supported_os: Option<String>,
    #[serde(rename = "BackupSoftwares", skip_serializing_if = "Option::is_none")]
    pub backup_softwares: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub struct CyberSecurityReportCard {
    pub anti_virus: Option<f64>,
    pub local_firewall: Option<f64>,
    pub insecure_listening_ports: Option<f64>,
    pub failed_login: Option<f64>,
    pub network_vulnerabilities: Option<f64>,
    pub system_aging: Option<f64>,
    #[serde(rename = "supportedOS", skip_serializing_if = "Option::is_none")]
    pub supported_os: Option<f64>,
    #[serde(rename = "BackupSoftwares", skip_serializing_if = "Option::is_none")]
    pub backup_softwares: Option<f64>,
    pub evidence: Option<CyberSecurityReportCardEvidence>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub struct CyberCompanyRef {
    pub id: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CyberAsset {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub host: Option<CyberHost>,
    pub security_reportcard: Option<CyberSecurityReportCard>,
    #[serde(rename = "companyRef", skip_serializing_if = "Option::is_none")]
    pub company_ref: Option<CyberCompanyRef>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CyberAssetResponse {
    pub data: Option<Vec<CyberAsset>>,
    pub total: Option<u64>,
}

/**
Fetch the assets from CyberCNS using the reqwest HTTP Client crate.
*/
pub async fn assets() -> Result<Vec<CyberAsset>, Error> {
    dotenv().ok();

    let username = env::var("CYBER_CNS_CLIENT_ID").ok().unwrap();
    let password = env::var("CYBER_CNS_CLIENT_SECRET").ok().unwrap();

    let client = Client::builder().http1_title_case_headers().build()?;

    let mut response = client
        .get("https://portaleuwest2.mycybercns.com/api/asset?limit=1")
        .header("customerid", "clay")
        .header("User-Agent", "ra-v1")
        .basic_auth(&username, Some(&password))
        .send()
        .await?;

    let mut body = response
        .json::<CyberAssetResponse>()
        .await
        .expect("Failed to retrieve CyberCNS assets.");

    let total_assets = body.total.ok_or(0);

    println!("Found {} assets from CyberCNS.", total_assets.unwrap());

    response = client
        .get(format!(
            "https://portaleuwest2.mycybercns.com/api/asset?limit={}",
            total_assets.unwrap()
        ))
        .header("customerid", "clay")
        .header("User-Agent", "ra-v1")
        .basic_auth(&username, Some(&password))
        .send()
        .await?;

    body = response
        .json::<CyberAssetResponse>()
        .await
        .expect("Failed to retrieve CyberCNS assets.");

    Ok(body.data.unwrap())
}
