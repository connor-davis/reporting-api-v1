use axum::{extract::State, response::IntoResponse, Json};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;

use crate::{
    functions::cybercns::assets::{
        assets, CyberCompanyRef, CyberHost, CyberSecurityReportCard,
        CyberSecurityReportCardEvidence,
    },
    models::cybercns::asset::CyberAsset,
};

pub async fn index(State(pool): State<PgPool>) -> impl IntoResponse {
    #[derive(Debug, Deserialize, Serialize)]
    struct FullCyberAsset {
        pub id: String,
        pub host: Option<CyberHost>,
        pub security_report_card: Option<CyberSecurityReportCard>,
        pub company: Option<CyberCompanyRef>,
    }

    let full_assets = sqlx::query!(
        r#"
        SELECT
            a.id AS asset_id,
            h.host_name AS host_name,
            sr.id AS security_report_card_id,
            sr.anti_virus AS anti_virus,
            sr.local_firewall AS local_firewall,
            sr.insecure_listening_ports AS insecure_listening_ports,
            sr.failed_login AS failed_login,
            sr.network_vulnerabilities AS network_vulnerabilities,
            sr.system_aging AS system_aging,
            sr.supported_os AS supported_os,
            sr.backup_softwares AS backup_softwares,
            sre.anti_virus AS evidence_anti_virus,
            sre.local_firewall AS evidence_local_firewall,
            sre.insecure_listening_ports AS evidence_insecure_listening_ports,
            sre.failed_login AS evidence_failed_login,
            sre.network_vulnerabilities AS evidence_network_vulnerabilities,
            sre.system_aging AS evidence_system_aging,
            sre.supported_os AS evidence_supported_os,
            sre.backup_softwares AS evidence_backup_softwares,
            c.id AS company_id,
            c.name AS company_name
        FROM cybercns_assets AS a
        LEFT JOIN cybercns_hosts AS h ON a.host = h.id
        LEFT JOIN cybercns_security_report_card AS sr ON a.security_report_card = sr.id
        LEFT JOIN cybercns_security_report_card_evidence AS sre ON sr.evidence = sre.id
        LEFT JOIN cybercns_companies AS c ON a.company = c.id
        "#
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to get data from postgres.");

    let result: Vec<FullCyberAsset> = full_assets
        .into_iter()
        .map(|row| FullCyberAsset {
            id: row.asset_id,
            host: Some(CyberHost {
                host_name: row.host_name,
            }),
            security_report_card: Some(CyberSecurityReportCard {
                anti_virus: row.anti_virus,
                local_firewall: row.local_firewall,
                insecure_listening_ports: row.insecure_listening_ports,
                failed_login: row.failed_login,
                network_vulnerabilities: row.network_vulnerabilities,
                system_aging: row.system_aging,
                supported_os: row.supported_os,
                backup_softwares: row.backup_softwares,
                evidence: Some(CyberSecurityReportCardEvidence {
                    anti_virus: row.evidence_anti_virus,
                    local_firewall: row.evidence_local_firewall,
                    insecure_listening_ports: row.evidence_insecure_listening_ports,
                    failed_login: row.evidence_failed_login,
                    network_vulnerabilities: row.evidence_network_vulnerabilities,
                    system_aging: row.evidence_system_aging,
                    supported_os: row.evidence_supported_os,
                    backup_softwares: row.evidence_backup_softwares,
                }),
            }),
            company: Some(CyberCompanyRef {
                id: row.company_id,
                name: row.company_name,
            }),
        })
        .collect();

    Json(result)
}

pub async fn import(State(pool): State<PgPool>) -> impl IntoResponse {
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

    Json(json!({
        "status": StatusCode::OK.as_u16(),
        "inserted": inserted,
        "skipped": skipped
    }))
}
