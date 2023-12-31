use axum::{extract::State, response::IntoResponse, Json};
use reqwest::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use crate::{
    functions::cybercns::vulnerabilities::{vulnerabilities, CyberVulnerabilityScore},
    models::cybercns::vulnerability::CyberVulnerability,
};

pub async fn index(State(pool): State<PgPool>) -> impl IntoResponse {
    let vulnerabilities: Vec<CyberVulnerability> = sqlx::query_as!(
        CyberVulnerability,
        "SELECT * FROM cybercns_vulnerabilities;"
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to get rocketcyber agents from postgres.");

    Json(vulnerabilities)
}

pub async fn import(State(pool): State<PgPool>) -> impl IntoResponse {
    let vulnerabilities = vulnerabilities()
        .await
        .expect("Failed to get cybercns vulnerabilities from cybercns api.");

    let mut inserted = 0;
    let mut skipped = 0;

    for vulnerability in vulnerabilities {
        let vulnerability_id = vulnerability._id;

        if vulnerability_id.is_some() {
            let vulnerability_id = vulnerability_id.to_owned().unwrap();

            let existing_vulnerability_result = sqlx::query_as!(
                CyberVulnerability,
                "SELECT * FROM cybercns_vulnerabilities WHERE id = $1",
                vulnerability_id.clone().to_string()
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
                            vulnerability_id.clone().to_string(),
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
                        .expect("Failed to insert cybercns company into postgres database.");

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

    Json(json!({
        "status": StatusCode::OK.as_u16(),
        "inserted": inserted,
        "skipped": skipped,
    }))
}
