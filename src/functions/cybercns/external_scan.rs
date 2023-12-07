use std::{env, fs::File, path::Path, time::Duration};

use anyhow::Error;
use chrono::Local;
use dotenv::dotenv;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::postgres::PgPoolOptions;
use tokio::{
    fs::{create_dir, try_exists},
    time::sleep,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ExternalScanHostname {
    pub tenant_id: i32,
    pub tenant_name: Option<String>,
    pub host_name: String,
    pub company_id: Option<String>,
}

pub async fn start_external_scan() -> Result<(), Error> {
    println!("Starting new external scan.");

    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost".into());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&database_url)
        .await
        .expect("Failed to connect to postgres.");

    let tenant_external_scan_hostnames = sqlx::query_as!(
        ExternalScanHostname,
        r#"
            SELECT
                h.tenant_id as tenant_id,
                h.host_name as host_name,
                t.tenant_name as tenant_name,
                c.id as company_id
            FROM tenants_external_scan_host_names as h
            LEFT JOIN tenants as t ON t.id = h.tenant_id
            LEFT JOIN cybercns_companies AS c ON LOWER(c.name) = LOWER(t.cyber_cns_name);
        "#
    )
    .fetch_all(&pool)
    .await;

    match tenant_external_scan_hostnames {
        Ok(tenant_external_scan_hostnames) => {
            for tenant_external_scan_hostname in tenant_external_scan_hostnames {
                let result = execute_quick_external_scan(tenant_external_scan_hostname).await;

                match result {
                    Ok(_) => {}
                    Err(error) => println!("External Scan Error: {}", error),
                }
            }
        }
        Err(_) => {}
    }

    Ok(())
}

pub async fn execute_quick_external_scan(data: ExternalScanHostname) -> Result<(), Error> {
    println!(
        "{}",
        format!(
            "External Scan for {} on hostname {}.",
            &data.tenant_name.clone().unwrap(),
            &data.host_name
        )
    );

    dotenv().ok();

    let company_id = data.company_id.clone().unwrap();

    let username = env::var("CYBER_CNS_CLIENT_ID").ok().unwrap();
    let password = env::var("CYBER_CNS_CLIENT_SECRET").ok().unwrap();

    let client = Client::builder().http1_title_case_headers().build()?;

    let req_body = json!({
        "hostname": data.host_name,
        "Scannow": true
    });

    let response = client
    .post(format!(
        "https://portaleuwest2.mycybercns.com/api/company/{}/quickExternalScan",
        company_id,
    ))
    .header("customerid", "clay")
    .header("User-Agent", "ra-v1")
    .header("Content-Type", "application/json")
    .basic_auth(&username, Some(&password))
    .json(&req_body)
    .send()
    .await?;

    println!("{:?}", response.text().await.unwrap());

    let response = client
        .post(format!(
            "https://portaleuwest2.mycybercns.com/api/company/{}/quickExternalScan",
            company_id,
        ))
        .header("customerid", "clay")
        .header("User-Agent", "ra-v1")
        .header("Content-Type", "application/json")
        .basic_auth(&username, Some(&password))
        .json(&req_body)
        .send()
        .await?;

    let body = response.json::<(bool, String)>().await;

    match body {
        Ok(body) => {
            println!("{}", body.1);

            if body.0 {
                loop {
                    sleep(Duration::from_secs(5)).await;

                    let results = execute_quick_external_scan_results(ExternalScanHostname {
                        tenant_id: data.tenant_id.clone(),
                        tenant_name: data.tenant_name.clone(),
                        host_name: data.host_name.clone(),
                        company_id: data.company_id.clone(),
                    })
                    .await?;

                    if !results.0 {
                        continue;
                    } else {
                        // Create the reports directory if it doesn't exist.
                        let directory_exists = try_exists("scans").await;

                        match directory_exists {
                            Ok(directory) => {
                                if directory {
                                    println!("directory found");
                                } else {
                                    println!("directory not found. creating");

                                    let create_dir_result = create_dir("scans").await;

                                    match create_dir_result {
                                        Ok(_) => {
                                            println!("directory created");
                                        }
                                        Err(_) => {
                                            println!("failed to create directory");
                                        }
                                    }
                                }
                            }
                            Err(_) => {
                                println!("unknown error when checking directory exists")
                            }
                        }

                        let reports_dir = Path::new("scans");

                        // Create the current date and time variable.
                        let current_date_time = Local::now();

                        // Create the file name which includes the tenant name and the current date and time.
                        let file_name = format!(
                            "scan-{}-{}-{}.json",
                            data.tenant_name.clone().unwrap(),
                            data.host_name.clone(),
                            current_date_time.format("%Y-%m-%d-%H-%M-%S")
                        );

                        // Create the file path.
                        let file_path = reports_dir.join(file_name.clone());

                        // Create the file.
                        let mut file =
                            File::create(&file_path).expect("Failed to create the file.");

                        // Write the data to the file.
                        match serde_json::to_writer(
                            &mut file,
                            &json!({
                                "scan_host_name": data,
                                "scan_results": results.1
                            }),
                        ) {
                            Ok(_) => println!("Generated new scan results: {:?}", file_path),
                            Err(error) => {
                                println!("External Scan Results Generation Error: {}", error)
                            }
                        }

                        break;
                    }
                }
            }
        }
        Err(error) => println!("External Scan Error: {}", error),
    }

    Ok(())
}

pub async fn execute_quick_external_scan_results(
    data: ExternalScanHostname,
) -> Result<(bool, Value), Error> {
    dotenv().ok();

    let company_id = data.company_id.clone().unwrap();

    let username = env::var("CYBER_CNS_CLIENT_ID").ok().unwrap();
    let password = env::var("CYBER_CNS_CLIENT_SECRET").ok().unwrap();

    let client = Client::builder().http1_title_case_headers().build()?;
    let req_body = json!({
        "hostname": data.host_name
    });

    let response = client
        .post(format!(
            "https://portaleuwest2.mycybercns.com/api/company/{}/quickExternalScanResults",
            company_id
        ))
        .header("customerid", "clay")
        .header("User-Agent", "ra-v1")
        .header("Content-Type", "application/json")
        .basic_auth(&username, Some(&password))
        .json(&req_body)
        .send()
        .await?;

    let body = response.json::<(bool, Value)>().await;

    match body {
        Ok(body) => Ok(body),
        Err(error) => Ok((
            false,
            json!(format!("External Scan Results Error: {}", error)),
        )),
    }
}
