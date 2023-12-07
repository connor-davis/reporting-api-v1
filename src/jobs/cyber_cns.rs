use std::time::Duration;

use anyhow::Result;
use axum::Json;
use dotenv::dotenv;
use reqwest::StatusCode;
use serde_json::json;
use sqlx::postgres::PgPoolOptions;

use crate::{
    functions::cybercns::{
        agents::agents,
        assets::{
            assets, CyberCompanyRef, CyberHost, CyberSecurityReportCard,
            CyberSecurityReportCardEvidence,
        },
        vulnerabilities::{vulnerabilities, CyberVulnerabilityScore},
    },
    models::cybercns::{agent::CyberAgent, asset::CyberAsset, vulnerability::CyberVulnerability},
};

pub async fn sync_cybercns() -> Result<()> {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost".into());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&database_url)
        .await
        .expect("Failed to connect to postgres.");

    println!("Syncing CyberCNS agents.");

    let agents = agents()
        .await
        .expect("Failed to get cybercns agents from cybercns api.");

    let mut inserted = 0;
    let mut skipped = 0;

    for agent in agents {
        let agent_id = agent.id.unwrap();

        let existing_agent_result = sqlx::query_as!(
            CyberAgent,
            "SELECT * FROM cybercns_agents WHERE id = $1;",
            agent_id
        )
        .fetch_one(&pool)
        .await;

        match existing_agent_result {
            Ok(_) => {
                skipped += 1;
            }
            Err(_) => {
                if !agent.host_name.is_none() {
                    sqlx::query_as!(
                        CyberAgent,
                        "INSERT INTO cybercns_agents VALUES ($1,$2);",
                        agent_id,
                        agent.host_name.unwrap()
                    )
                    .execute(&pool)
                    .await
                    .expect("Failed to insert cybercns agent into postgres database.");

                    inserted += 1;
                } else {
                    sqlx::query_as!(
                        CyberAgent,
                        "INSERT INTO cybercns_agents VALUES ($1,$2);",
                        agent_id,
                        "Unknown"
                    )
                    .execute(&pool)
                    .await
                    .expect("Failed to insert cybercns agent into postgres database.");

                    inserted += 1;
                }
            }
        }
    }

    println!(
        "{:?}",
        Json(json!({
            "status": StatusCode::OK.as_u16(),
            "inserted": inserted,
            "skipped": skipped
        }))
    );

    println!("Syncing CyberCNS vulnerabilities.");

    let vulnerabilities = vulnerabilities()
        .await
        .expect("Failed to get cybercns vulnerabilities from cybercns api.");

    let mut inserted = 0;
    let mut skipped = 0;

    for vulnerability in vulnerabilities {
        let vulnerability_id = &vulnerability._id;

        if vulnerability_id.to_owned().is_some() {
            let vulnerability_id = vulnerability_id.to_owned().unwrap();

            let existing_vulnerability_result = sqlx::query_as!(
                CyberVulnerability,
                "SELECT * FROM cybercns_vulnerabilities WHERE id = $1",
                vulnerability_id.to_string()
            )
            .fetch_one(&pool)
            .await;

            match existing_vulnerability_result {
                Ok(_) => {
                    skipped += 1;
                }
                Err(_) => {
                    let score = vulnerability
                        .score
                        .unwrap_or_else(|| CyberVulnerabilityScore {
                            base_score: Some(0.0),
                            impact_score: Some(0.0),
                            exploit_score: Some(0.0),
                            cvss_score: Some(0.0),
                        });

                    let asset_ref = vulnerability.asset_ref;
                    let company_ref = vulnerability.company_ref;

                    if asset_ref.is_some() && company_ref.is_some() {
                        let asset_ref = asset_ref.unwrap();
                        let company_ref = company_ref.unwrap();
                        let product = vulnerability.product.unwrap_or(Vec::new());

                        sqlx::query!(
                            "INSERT INTO cybercns_vulnerabilities (id, title, severity, vector, product, base_score, impact_score, exploit_score, cvss_score, asset_id, company_id, company_name) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12);",
                            vulnerability._id,
                            vulnerability.title,
                            vulnerability.severity,
                            vulnerability.vector,
                            product.join(", "),
                            score.base_score,
                            score.impact_score,
                            score.exploit_score,
                            score.cvss_score,
                            asset_ref.id,
                            company_ref.id,
                            company_ref.name
                        )
                        .execute(&pool)
                        .await
                        .expect("Failed to insert cybercns vulnerability into postgres database.");

                        inserted += 1;
                    } else {
                        skipped += 1;
                    }
                }
            }
        } else {
            skipped += 1;
        }
    }

    println!(
        "{:?}",
        Json(json!({
            "status": StatusCode::OK.as_u16(),
            "inserted": inserted,
            "skipped": skipped,
        }))
    );

    println!("Syncing CyberCNS assets.");

    let assets = assets()
        .await
        .expect("Failed to get cybercns assets from cybercns api.");

    let mut inserted = 0;
    let mut skipped = 0;

    for asset in assets {
        let asset_id = asset.id.unwrap();

        let existing_asset_result = sqlx::query_as!(
            CyberAsset,
            "SELECT * FROM cybercns_assets WHERE id = $1;",
            asset_id
        )
        .fetch_one(&pool)
        .await;

        match existing_asset_result {
            Ok(_) => {
                skipped += 1;
            }
            Err(_) => {
                let asset_host = asset.host.unwrap_or_else(|| CyberHost {
                    host_name: Some("Data not found from CyberCNS.".to_string()),
                });
                let asset_security_report_card =
                    asset
                        .security_reportcard
                        .unwrap_or_else(|| CyberSecurityReportCard {
                            anti_virus: Some(0.0),
                            local_firewall: Some(0.0),
                            insecure_listening_ports: Some(0.0),
                            failed_login: Some(0.0),
                            network_vulnerabilities: Some(0.0),
                            system_aging: Some(0.0),
                            supported_os: Some(0.0),
                            backup_softwares: Some(0.0),
                            evidence: Some(CyberSecurityReportCardEvidence {
                                anti_virus: Some("Data not found from CyberCNS.".to_string()),
                                local_firewall: Some("Data not found from CyberCNS.".to_string()),
                                insecure_listening_ports: Some(
                                    "Data not found from CyberCNS.".to_string(),
                                ),
                                failed_login: Some("Data not found from CyberCNS.".to_string()),
                                network_vulnerabilities: Some(
                                    "Data not found from CyberCNS.".to_string(),
                                ),
                                system_aging: Some("Data not found from CyberCNS.".to_string()),
                                supported_os: Some("Data not found from CyberCNS.".to_string()),
                                backup_softwares: Some("Data not found from CyberCNS.".to_string()),
                            }),
                        });
                let asset_security_report_card_evidence = asset_security_report_card
                    .evidence
                    .unwrap_or_else(|| CyberSecurityReportCardEvidence {
                        anti_virus: Some("Data not found from CyberCNS.".to_string()),
                        local_firewall: Some("Data not found from CyberCNS.".to_string()),
                        insecure_listening_ports: Some("Data not found from CyberCNS.".to_string()),
                        failed_login: Some("Data not found from CyberCNS.".to_string()),
                        network_vulnerabilities: Some("Data not found from CyberCNS.".to_string()),
                        system_aging: Some("Data not found from CyberCNS.".to_string()),
                        supported_os: Some("Data not found from CyberCNS.".to_string()),
                        backup_softwares: Some("Data not found from CyberCNS.".to_string()),
                    });
                let asset_company_ref = asset.company_ref.unwrap_or_else(|| CyberCompanyRef {
                    id: Some("Data not found from CyberCNS.".to_string()),
                    name: Some("Data not found from CyberCNS.".to_string()),
                });

                let host_record = sqlx::query!(
                    "INSERT INTO cybercns_hosts (host_name) VALUES ($1) RETURNING id;",
                    asset_host.host_name,
                )
                .fetch_one(&pool)
                .await
                .expect("Failed to insert cybercns company into postgres database.");

                let evidence_record = sqlx::query!(
                    "INSERT INTO cybercns_security_report_card_evidence (anti_virus,local_firewall,insecure_listening_ports,network_vulnerabilities,system_aging,supported_os,backup_softwares) VALUES ($1,$2,$3,$4,$5,$6,$7) RETURNING id;",
                    asset_security_report_card_evidence.anti_virus,
                    asset_security_report_card_evidence.local_firewall,
                    asset_security_report_card_evidence.insecure_listening_ports,
                    asset_security_report_card_evidence.network_vulnerabilities,
                    asset_security_report_card_evidence.system_aging,
                    asset_security_report_card_evidence.supported_os,
                    asset_security_report_card_evidence.backup_softwares
                )
                .fetch_one(&pool)
                .await
                .expect("Failed to insert cybercns company into postgres database.");

                let evidence_id: i64 = evidence_record.id as i64;

                let security_report_card_record = sqlx::query!(
                    "INSERT INTO cybercns_security_report_card (anti_virus,local_firewall,insecure_listening_ports,network_vulnerabilities,system_aging,supported_os,backup_softwares,evidence) VALUES ($1,$2,$3,$4,$5,$6,$7,$8) RETURNING id;",
                    asset_security_report_card.anti_virus,
                    asset_security_report_card.local_firewall,
                    asset_security_report_card.insecure_listening_ports,
                    asset_security_report_card.network_vulnerabilities,
                    asset_security_report_card.system_aging,
                    asset_security_report_card.supported_os,
                    asset_security_report_card.backup_softwares,
                    evidence_id,
                )
                .fetch_one(&pool)
                .await
                .expect("Failed to insert cybercns company into postgres database.");

                let existing_company_result = sqlx::query!(
                    "SELECT id FROM cybercns_companies WHERE id = $1;",
                    asset_company_ref.id
                )
                .fetch_one(&pool)
                .await;

                match existing_company_result {
                    Ok(company_record) => {
                        sqlx::query_as!(
                            CyberAsset,
                            "INSERT INTO cybercns_assets (id,host,security_report_card,company) VALUES ($1,$2,$3,$4);",
                            asset_id,
                            host_record.id,
                            security_report_card_record.id,
                            company_record.id
                        )
                        .execute(&pool)
                        .await
                        .expect("Failed to insert cybercns asset into postgres database.");
                    }
                    Err(_) => {
                        let company_record = sqlx::query!(
                            "INSERT INTO cybercns_companies VALUES ($1, $2) RETURNING id;",
                            asset_company_ref.id,
                            asset_company_ref.name
                        )
                        .fetch_one(&pool)
                        .await
                        .expect("Failed to insert cybercns company into postgres database.");

                        sqlx::query_as!(
                            CyberAsset,
                            "INSERT INTO cybercns_assets (id,host,security_report_card,company) VALUES ($1,$2,$3,$4);",
                            asset_id,
                            host_record.id,
                            security_report_card_record.id,
                            company_record.id
                        )
                        .execute(&pool)
                        .await
                        .expect("Failed to insert cybercns asset into postgres database.");
                    }
                }

                inserted += 1;
            }
        }
    }

    println!(
        "{:?}",
        Json(json!({
            "status": StatusCode::OK.as_u16(),
            "inserted": inserted,
            "skipped": skipped
        }))
    );

    println!("Finished CyberCNS sync.");

    Ok(())
}
