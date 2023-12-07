use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;

pub async fn index(
    Query(params): Query<Vec<(String, String)>>,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    let tenant = &params[0].1;

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Vulnerability {
        pub host_name: Option<String>,
        pub title: Option<String>,
        pub product: Option<String>,
        pub severity: Option<String>,
        pub vector: Option<String>,
        pub base_score: Option<f64>,
        pub exploit_score: Option<f64>,
        pub impact_score: Option<f64>,
        pub cvss_score: Option<f64>,
    }

    let vulnerabilities = sqlx::query_as!(
        Vulnerability,
        r#"
            SELECT
                v.base_score as base_score,
                v.exploit_score as exploit_score,
                v.impact_score as impact_score,
                v.cvss_score as cvss_score,
                v.title as title,
                v.product as product,
                v.severity as severity,
                v.vector as vector,
                h.host_name as host_name
            FROM cybercns_vulnerabilities AS v
            LEFT JOIN cybercns_assets AS a ON v.asset_id = a.id
            LEFT JOIN cybercns_hosts AS h ON a.host = h.id
            LEFT JOIN cybercns_companies AS c ON a.company = c.id
            WHERE similarity(LOWER(c.name), LOWER($1)) >= 0.6
            ORDER BY h.host_name;
        "#,
        tenant
    )
    .fetch_all(&pool)
    .await;

    match vulnerabilities {
        Ok(vulnerabilities) => Json(json!({
            "status": StatusCode::OK.as_u16(),
            "tenant": tenant,
            "results": vulnerabilities
        })),
        Err(_) => Json(json!({
            "status": StatusCode::OK.as_u16(),
            "tenant": tenant,
            "results": []
        })),
    }
}
