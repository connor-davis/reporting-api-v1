use std::{env, time::Duration};

use anyhow::Error;
use dotenv::dotenv;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::postgres::PgPoolOptions;

#[derive(Debug, Serialize, Deserialize)]
pub struct CyberVulnerabilityScore {
    pub base_score: Option<f64>,
    pub impact_score: Option<f64>,
    pub exploit_score: Option<f64>,
    pub cvss_score: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompanyRef {
    pub id: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentRef {
    pub id: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "agentType", skip_serializing_if = "Option::is_none")]
    pub agent_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetRef {
    pub id: Option<String>,
    pub name: Option<String>,
    pub platform: Option<String>,
    pub tags: Option<Vec<String>>,
}

// _id, product, category, title, severity, vector, cisa_vulnerabilityname, cisa, vuls_suppression, port, score, companyRef, agentRef, assetRef

#[derive(Debug, Serialize, Deserialize)]
pub struct CyberVulnerability {
    pub _id: Option<String>,
    pub product: Option<Vec<String>>,
    pub category: Option<String>,
    pub title: Option<String>,
    pub severity: Option<String>,
    pub vector: Option<String>,
    pub cisa_vulnerabilityname: Option<String>,
    pub cisa: Option<bool>,
    pub vuls_suppression: Option<bool>,
    pub port: Option<u32>,
    pub score: Option<CyberVulnerabilityScore>,
    #[serde(rename = "companyRef", skip_serializing_if = "Option::is_none")]
    pub company_ref: Option<CompanyRef>,
    #[serde(rename = "agentRef", skip_serializing_if = "Option::is_none")]
    pub agent_ref: Option<AgentRef>,
    #[serde(rename = "assetRef", skip_serializing_if = "Option::is_none")]
    pub asset_ref: Option<AssetRef>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub struct CyberVulnerabilityResponse {
    pub data: Option<Vec<CyberVulnerability>>,
    pub total: Option<i64>,
    pub scroll_id: Option<String>,
}

pub async fn vulnerabilities() -> Result<Vec<CyberVulnerability>, Error> {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost".into());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&database_url)
        .await
        .expect("Failed to connect to postgres.");

    #[derive(Debug, Deserialize, Serialize)]
    pub struct CyberCompany {
        id: Option<String>,
        name: Option<String>,
    }

    let companies_result = sqlx::query_as!(
        CyberCompany,
        r#"
            SELECT id, name FROM cybercns_companies;
        "#
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to find CyberCNS companies from postgres database.");

    let username = env::var("CYBER_CNS_CLIENT_ID").ok().unwrap();
    let password = env::var("CYBER_CNS_CLIENT_SECRET").ok().unwrap();

    let mut vulnerabilities: Vec<CyberVulnerability> = Vec::new();

    for company in companies_result {
        let company_name = company.name.unwrap();

        let client = Client::builder().http1_title_case_headers().build()?;

        println!(
            "Fetching critical severity vulnerabilities for {}",
            company_name
        );

        let response = client
        .get(format!(
            "https://portaleuwest2.mycybercns.com/api/vulnerability?q={}&limit=1",
            json!({"query":{"bool":{"must":[{"exists":{"field":"_id"}},{"exists":{"field":"vul_id"}},{"exists":{"field":"score"}},{"exists":{"field":"severity"}},{"exists":{"field":"companyRef"}},{"exists":{"field":"assetRef"}},{"match":{"companyRef.id.keyword":company.id}},{"match":{"severity.keyword":"Critical"}}]}}}).to_string(),
        ))
        .header("customerid", "clay")
        .header("User-Agent", "ra-v1")
        .basic_auth(&username, Some(&password))
        .send()
        .await?;

        let body = response
            .json::<CyberVulnerabilityResponse>()
            .await
            .expect("Failed to retrieve CyberCNS assets.");

        let total_assets = body.total.ok_or(0);

        println!(
            "Found {} vulnerabilities from CyberCNS for {}.",
            total_assets.unwrap(),
            company_name
        );

        let limit = 100;
        let total = total_assets.unwrap();
        let pages = total / limit;
        let mut current_page = 0;

        println!("Pages: {}", pages + 1);

        let company_name = company_name.clone();

        while current_page < pages + 1 {
            let response = client
            .get(format!(
                "https://portaleuwest2.mycybercns.com/api/vulnerability?q={}&limit=100&skip={}",
                json!({"query":{"bool":{"must":[{"exists":{"field":"_id"}},{"exists":{"field":"vul_id"}},{"exists":{"field":"score"}},{"exists":{"field":"severity"}},{"exists":{"field":"companyRef"}},{"exists":{"field":"assetRef"}},{"match":{"companyRef.id.keyword":company.id}},{"match":{"severity.keyword":"Critical"}}]}}}).to_string(),
                current_page * limit
            ))
            .header("customerid", "clay")
            .header("User-Agent", "ra-v1")
            .basic_auth(&username, Some(&password))
            .send()
            .await;

            match response {
                Ok(response) => {
                    let vulnerabilities_result =
                        response.json::<CyberVulnerabilityResponse>().await;

                    match vulnerabilities_result {
                        Ok(result) => {
                            if result.data.is_some() {
                                let page_vulnerabilities = result.data.unwrap();
                                let page_size = page_vulnerabilities.len();

                                if page_size == 0 {
                                    break;
                                }

                                vulnerabilities.extend(page_vulnerabilities);

                                println!(
                                    "Fetched page {}, current dataset: {}, total dataset: {}, company: {}",
                                    current_page + 1,
                                    page_size,
                                    vulnerabilities.len(),
                                    company_name
                                );
                                current_page += 1;
                                continue;
                            }
                        }
                        Err(error) => {
                            println!("Failed to get page {}: {}", current_page + 1, error);
                            current_page += 1;
                            continue;
                        }
                    }
                }
                Err(error) => {
                    println!("Failed to get page {}: {}", current_page + 1, error);
                    current_page += 1;
                    continue;
                }
            }
        }

        println!(
            "Fetching high severity vulnerabilities for {}",
            company_name
        );

        let response = client
        .get(format!(
            "https://portaleuwest2.mycybercns.com/api/vulnerability?q={}&limit=1",
            json!({"query":{"bool":{"must":[{"exists":{"field":"_id"}},{"exists":{"field":"vul_id"}},{"exists":{"field":"score"}},{"exists":{"field":"severity"}},{"exists":{"field":"companyRef"}},{"exists":{"field":"assetRef"}},{"match":{"companyRef.id.keyword":company.id}},{"match":{"severity.keyword":"High"}}]}}}).to_string(),
        ))
        .header("customerid", "clay")
        .header("User-Agent", "ra-v1")
        .basic_auth(&username, Some(&password))
        .send()
        .await?;

        let body = response
            .json::<CyberVulnerabilityResponse>()
            .await
            .expect("Failed to retrieve CyberCNS assets.");

        let total_assets = body.total.ok_or(0);

        println!(
            "Found {} vulnerabilities from CyberCNS for {}.",
            total_assets.unwrap(),
            company_name
        );

        let limit = 100;
        let total = total_assets.unwrap();
        let pages = total / limit;
        let mut current_page = 0;

        println!("Pages: {}", pages + 1);

        let company_name = company_name.clone();

        while current_page < pages + 1 {
            let response = client
            .get(format!(
                "https://portaleuwest2.mycybercns.com/api/vulnerability?q={}&limit=100&skip={}",
                json!({"query":{"bool":{"must":[{"exists":{"field":"_id"}},{"exists":{"field":"vul_id"}},{"exists":{"field":"score"}},{"exists":{"field":"severity"}},{"exists":{"field":"companyRef"}},{"exists":{"field":"assetRef"}},{"match":{"companyRef.id.keyword":company.id}},{"match":{"severity.keyword":"High"}}]}}}).to_string(),
                current_page * limit,
            ))
            .header("customerid", "clay")
            .header("User-Agent", "ra-v1")
            .basic_auth(&username, Some(&password))
            .send()
            .await;

            match response {
                Ok(response) => {
                    let vulnerabilities_result =
                        response.json::<CyberVulnerabilityResponse>().await;

                    match vulnerabilities_result {
                        Ok(result) => {
                            if result.data.is_some() {
                                let page_vulnerabilities = result.data.unwrap();
                                let page_size = page_vulnerabilities.len();

                                if page_size == 0 {
                                    break;
                                }

                                vulnerabilities.extend(page_vulnerabilities);

                                println!(
                                    "Fetched page {}, current dataset: {}, total dataset: {}, company: {}",
                                    current_page + 1,
                                    page_size,
                                    vulnerabilities.len(),
                                    company_name
                                );
                                current_page += 1;
                                continue;
                            }
                        }
                        Err(error) => {
                            println!("Failed to get page {}: {}", current_page + 1, error);
                            current_page += 1;
                            continue;
                        }
                    }
                }
                Err(error) => {
                    println!("Failed to get page {}: {}", current_page + 1, error);
                    current_page += 1;
                    continue;
                }
            }
        }
    }

    Ok(vulnerabilities)
}
