use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use math::round::ceil;
use reqwest::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use crate::models::vsa::agent::VsaAgent;

pub async fn index(
    Query(params): Query<Vec<(String, String)>>,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    let tenant = &params[0].1;
    let offset = (&params[1].1.parse::<i64>().unwrap_or(1) - 1) * 10;

    let vsa_organizations_result =
        sqlx::query_as!(
            VsaAgent,
            "SELECT * FROM vsa_agents WHERE LOWER(TRIM(TRAILING ' (Pty) Ltd' FROM organization_name)) = LOWER($1) ORDER BY computer_name OFFSET $2 LIMIT 10;",
            tenant,
            offset
        )
            .fetch_all(&pool)
            .await
            .expect("Failed to get vsa organization names from postgres.");

    let mut total_pages_result = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM vsa_agents WHERE LOWER(TRIM(TRAILING ' (Pty) Ltd' FROM organization_name)) = LOWER($1);",
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
        "results": vsa_organizations_result,
        "total_pages": total_pages
    }))
}
