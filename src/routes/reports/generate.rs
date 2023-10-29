use std::{collections::HashMap, fs::File, path::Path};

use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Datelike, Local, Utc};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use tokio::fs::{create_dir, try_exists};

use crate::{
    functions::cybercns::assets::{
        CyberCompanyRef, CyberHost, CyberSecurityReportCard, CyberSecurityReportCardEvidence,
    },
    models::{
        rocketcyber::{account::RocketAccount, incident::RocketIncident},
        vsa::agent::VsaAgent,
    },
};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "PascalCase", serialize = "PascalCase"))]
pub struct VsaPatch {
    pub id: String,
    pub organization_name: Option<String>,
    pub computer_name: Option<String>,
    pub total_patches: Option<f64>,
    pub installed_patches: Option<f64>,
    pub last_patch: Option<DateTime<Utc>>,
    pub next_patch: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VsaPatchResult {
    pub id: String,
    pub computer_name: Option<String>,
    pub patch_status: Option<String>,
    pub total_patches: Option<f64>,
    pub installed_patches: Option<f64>,
    pub last_patch: Option<DateTime<Utc>>,
    pub next_patch: Option<DateTime<Utc>>,
}

// Create the logic to save all the tables in the database to a file.
// The file will be a CSV file.
pub async fn index(
    Query(params): Query<Vec<(String, String)>>,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    let tenant = &params[0].1;

    // VSA Agents

    let vsa_table_result = sqlx::query_as!(
        VsaAgent,
        "SELECT * FROM vsa_agents WHERE similarity(LOWER(organization_name), LOWER($1)) >= 0.6 ORDER BY computer_name;",
        tenant
    ).fetch_all(&pool).await;

    let vsa_table = match vsa_table_result {
        Ok(vsa_table) => vsa_table,
        Err(_) => Vec::new(),
    };

    // VSA Patching

    let vsa_patching_table_result = sqlx::query_as!(
        VsaPatch,
        "SELECT id, organization_name, computer_name, total_patches, installed_patches, last_patch, next_patch FROM vsa_agents WHERE similarity(LOWER(organization_name), LOWER($1)) >= 0.6 ORDER BY computer_name;",
        tenant
    )
        .fetch_all(&pool)
        .await;

    let vsa_patching_table = match vsa_patching_table_result {
        Ok(vsa_patching_table) => vsa_patching_table,
        Err(_) => Vec::new(),
    };

    let mut patch_results_array: Vec<VsaPatchResult> = Vec::new();

    for result in vsa_patching_table {
        let mut patch_status = "OUTDATED".to_string();

        if result.total_patches == result.installed_patches {
            patch_status = "UP TO DATE".to_string();
        }

        patch_results_array.push(VsaPatchResult {
            id: result.id,
            computer_name: result.computer_name,
            patch_status: Some(patch_status),
            total_patches: result.total_patches,
            installed_patches: result.installed_patches,
            last_patch: result.last_patch,
            next_patch: result.next_patch,
        })
    }

    // Rocket Cyber
    let account = sqlx::query_as!(
        RocketAccount,
        r#"
            SELECT
                *
            FROM rocketcyber_accounts
            WHERE similarity(LOWER(account_name), LOWER($1)) >= 0.6
        "#,
        tenant
    )
    .fetch_one(&pool)
    .await;

    let incidents_result = match account {
        Ok(account) => {
            let incidents_result = sqlx::query_as!(
                RocketIncident,
                r#"
                    SELECT
                        *
                    FROM rocketcyber_incidents AS incident
                    WHERE account_id = $1
                    ORDER BY incident.title;
                "#,
                account.account_id
            )
            .fetch_all(&pool)
            .await
            .expect("Failed to get total results from postgres.");

            incidents_result
        }
        Err(_) => Vec::new(),
    };

    // CyberCNS
    #[derive(Debug, Deserialize, Serialize)]
    struct FullCyberAsset {
        pub id: String,
        pub host: Option<CyberHost>,
        pub security_report_card: Option<CyberSecurityReportCard>,
        pub company: Option<CyberCompanyRef>,
    }

    let full_assets_result = sqlx::query!(
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
            WHERE similarity(LOWER(c.name), LOWER($1)) >= 0.6
            ORDER BY h.host_name;
        "#,
        tenant
    )
    .fetch_all(&pool)
    .await;

    let full_assets = match full_assets_result {
        Ok(full_assets) => full_assets,
        Err(_) => Vec::new(),
    };

    let assets_result: Vec<FullCyberAsset> = full_assets
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
                id: Some(row.company_id),
                name: Some(row.company_name),
            }),
        })
        .collect();

    // Statistics
    // VSA

    let vsa_statistics_result =
        sqlx::query!(
            "SELECT organization_name, anti_virus, os_name FROM vsa_agents WHERE similarity(LOWER(organization_name), LOWER($1)) >= 0.6;",
            tenant
        )
            .fetch_all(&pool)
            .await
            .expect("Failed to get vsa organization names from postgres.");

    let mut agents_count = 0;
    let mut agents_with_av_count = 0;
    let mut win11_agents_count = 0;
    let mut win10_agents_count = 0;
    let mut win7_agents_count = 0;

    for record in vsa_statistics_result {
        let anti_virus = record.anti_virus.unwrap_or(false);
        let os_name = record.os_name.unwrap_or("Unknown".to_string());

        agents_count += 1;

        if anti_virus {
            agents_with_av_count += 1;
        }

        // Write a check to see if os_name is windows 11/10/7 else nothing
        if os_name.contains("7601") {
            win7_agents_count += 1;
        }

        // Check if os_name contains any of the windows 10 build numbers
        if os_name.contains("10240")
            || os_name.contains("10586")
            || os_name.contains("14393")
            || os_name.contains("15063")
            || os_name.contains("16299")
            || os_name.contains("17134")
            || os_name.contains("17763")
            || os_name.contains("18362")
            || os_name.contains("18363")
            || os_name.contains("19041")
            || os_name.contains("19042")
            || os_name.contains("19043")
            || os_name.contains("19044")
            || os_name.contains("19045")
        {
            win10_agents_count += 1;
        }

        // Check if os_name contains any of the windows 11 build numbers
        if os_name.contains("22000") || os_name.contains("22621") {
            win11_agents_count += 1;
        }
    }

    let vsa_statistics = json!({
        "agents": agents_count,
        "agents_with_anti_virus": agents_with_av_count,
        "win11_agents": win11_agents_count,
        "win10_agents": win10_agents_count,
        "win7_agents": win7_agents_count
    });

    // VSA Patching
    let current_date = Utc::now();

    let mut total_patches_current_month = 0 as f64;
    let mut total_outstanding_patches_current_month = 0 as f64;

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all(deserialize = "PascalCase", serialize = "PascalCase"))]
    pub struct VsaPatchStatistics {
        pub id: String,
        pub organization_name: Option<String>,
        pub total_patches: Option<f64>,
        pub installed_patches: Option<f64>,
        pub last_patch: Option<DateTime<Utc>>,
        pub next_patch: Option<DateTime<Utc>>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    #[serde(rename_all(deserialize = "PascalCase", serialize = "PascalCase"))]
    pub struct VsaPatchStatisticsResult {
        pub total: Option<f64>,
        pub outstanding: Option<f64>,
        pub date: Option<String>,
    }

    let vsa_patches_result =
        sqlx::query_as!(
            VsaPatchStatistics,
            "SELECT id, organization_name, total_patches, installed_patches, last_patch, next_patch FROM vsa_agents WHERE similarity(LOWER(organization_name), LOWER($1)) >= 0.6 ORDER BY computer_name;",
            tenant
        )
            .fetch_all(&pool)
            .await
            .expect("Failed to get vsa organization names from postgres.");

    let mut patch_results: HashMap<String, VsaPatchStatisticsResult> = HashMap::new();

    for result in &vsa_patches_result {
        if !result.last_patch.is_none() {
            let last_patch = result.last_patch.unwrap();

            if last_patch.month() == current_date.month() {
                println!("There was a patch this month: {:?}", result);

                total_patches_current_month += result.total_patches.unwrap();
                total_outstanding_patches_current_month +=
                    result.total_patches.unwrap() - result.installed_patches.unwrap();
            }

            if !patch_results.contains_key(
                format!(
                    "{}-{}-{}",
                    last_patch.year(),
                    last_patch.month(),
                    last_patch.day()
                )
                .as_str(),
            ) {
                patch_results.insert(
                    format!(
                        "{}-{}-{}",
                        last_patch.year(),
                        last_patch.month(),
                        last_patch.day()
                    ),
                    VsaPatchStatisticsResult {
                        total: Some(result.total_patches.unwrap()),
                        outstanding: Some(
                            result.total_patches.unwrap() - result.installed_patches.unwrap(),
                        ),
                        date: Some(format!(
                            "{}-{}-{}",
                            last_patch.year(),
                            last_patch.month(),
                            last_patch.day()
                        )),
                    },
                );
            } else {
                let found_result = patch_results.get(
                    format!(
                        "{}-{}-{}",
                        last_patch.year(),
                        last_patch.month(),
                        last_patch.day()
                    )
                    .as_str(),
                );

                if !found_result.is_none() {
                    let unwrapped_result = found_result.unwrap();
                    let mut total = unwrapped_result.total.unwrap();
                    let mut outstanding = unwrapped_result.outstanding.unwrap();

                    total += result.total_patches.unwrap();
                    outstanding +=
                        result.total_patches.unwrap() - result.installed_patches.unwrap();

                    patch_results.insert(
                        format!(
                            "{}-{}-{}",
                            last_patch.year(),
                            last_patch.month(),
                            last_patch.day()
                        ),
                        VsaPatchStatisticsResult {
                            total: Some(total),
                            outstanding: Some(outstanding),
                            date: Some(format!(
                                "{}-{}-{}",
                                last_patch.year(),
                                last_patch.month(),
                                last_patch.day()
                            )),
                        },
                    );
                }
            }
        }
    }

    let mut patch_statistics_results_array: Vec<VsaPatchStatisticsResult> = Vec::new();

    for result in patch_results {
        patch_statistics_results_array.push(result.1)
    }

    let vsa_patching_statistics = json!({
        "total_patches": total_patches_current_month,
        "outstanding_patches": total_outstanding_patches_current_month,
        "results": patch_statistics_results_array
    });

    // Rocket Cyber
    let account = sqlx::query_as!(
        RocketAccount,
        r#"
            SELECT
                *
            FROM rocketcyber_accounts
            WHERE similarity(LOWER(account_name), LOWER($1)) >= 0.6;
        "#,
        tenant
    )
    .fetch_one(&pool)
    .await;

    let rocketcyber_statistics = match account {
        Ok(account) => {
            let total_agents = sqlx::query_scalar!(
                r#"
                    SELECT
                        COUNT(*)
                    FROM rocketcyber_agents AS agent
                    WHERE customer_id = $1
                "#,
                account.account_id
            )
            .fetch_one(&pool)
            .await
            .expect("Failed to get total results from postgres.");

            let total_incidents = sqlx::query_scalar!(
                r#"
                    SELECT
                        COUNT(*)
                    FROM rocketcyber_incidents AS incident
                    WHERE account_id = $1
                "#,
                account.account_id
            )
            .fetch_one(&pool)
            .await
            .expect("Failed to get total results from postgres.");

            json!({
                "status": StatusCode::OK.as_u16(),
                "tenant": tenant,
                "total_agents": total_agents,
                "total_incidents": total_incidents
            })
        }
        Err(_) => json!({
            "status": StatusCode::OK.as_u16(),
            "tenant": tenant,
            "total_agents": 0,
            "total_incidents": 0
        }),
    };

    // Create the reports directory if it doesn't exist.
    let directory_exists = try_exists("reports").await;

    match directory_exists {
        Ok(directory) => {
            if directory {
                println!("directory found");
            } else {
                println!("directory not found. creating");

                let create_dir_result = create_dir("reports").await;

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

    let reports_dir = Path::new("reports");

    // Create the current date and time variable.
    let current_date_time = Local::now();

    // Create the file name which includes the tenant name and the current date and time.
    let file_name = format!(
        "report-{}-{}.json",
        tenant,
        current_date_time.format("%Y-%m-%d-%H-%M-%S")
    );

    // Create the file path.
    let file_path = reports_dir.join(file_name.clone());

    // Create the file.
    let mut file = File::create(&file_path).expect("Failed to create the file.");

    // Write the data to the file.
    match serde_json::to_writer(
        &mut file,
        &json!({
            "vsa": vsa_table,
            "vsa_statistics": vsa_statistics,
            "vsa_patching": patch_results_array,
            "vsa_patching_statistics": vsa_patching_statistics,
            "rocketcyber": incidents_result,
            "rocketcyber_statistics": rocketcyber_statistics,
            "cybercns": assets_result,
        }),
    ) {
        Ok(_) => Json(json!({ "status": StatusCode::OK.as_u16(), "report-filename": file_name })),
        Err(_) => Json(
            json!({ "status": StatusCode::INTERNAL_SERVER_ERROR.as_u16(), "error": "Failed to write the data to the file."}),
        ),
    }
}
