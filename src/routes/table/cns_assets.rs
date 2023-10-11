use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use math::round::ceil;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;

use crate::functions::cybercns::assets::{
    CyberCompanyRef, CyberHost, CyberSecurityReportCard, CyberSecurityReportCardEvidence,
};

pub async fn index(
    Query(params): Query<Vec<(String, String)>>,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    let tenant = &params[0].1;
    let offset = (&params[1].1.parse::<i64>().unwrap_or(1) - 1) * 10;

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
            WHERE LOWER(TRIM(TRAILING ' (Pty) Ltd' FROM c.name)) = LOWER($1)
            ORDER BY h.host_name
            OFFSET $2 LIMIT 10;
        "#,
        tenant,
        offset
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to get vsa organization names from postgres.");

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

    let mut total_pages_result = sqlx::query_scalar!(
        r#"
            SELECT
                COUNT(*)
            FROM cybercns_assets AS a
            LEFT JOIN cybercns_companies AS c ON a.company = c.id
            WHERE LOWER(TRIM(TRAILING ' (Pty) Ltd' FROM c.name)) = LOWER($1)
        "#,
        tenant
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to get total results from postgres.");

    if total_pages_result.unwrap_or(10) < 10 {
        total_pages_result = Some(10);
    }

    let total_pages = ceil(total_pages_result.unwrap_or(10) as f64 / 10 as f64, 0);

    Json(json!({
        "status": StatusCode::OK.as_u16(),
        "tenant": tenant,
        "results": assets_result,
        "total_pages": total_pages
    }))
}
