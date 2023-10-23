use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use reqwest::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use crate::models::vsa::agent::VsaAgent;

pub async fn index(
    Query(params): Query<Vec<(String, String)>>,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    let tenant = &params[0].1;

    let vsa_organizations_result =
            sqlx::query_as!(
                VsaAgent,
                "SELECT * FROM vsa_agents WHERE similarity(LOWER(organization_name), LOWER($1)) >= 0.6 ORDER BY computer_name;",
                tenant
            )
                .fetch_all(&pool)
                .await
                .expect("Failed to get vsa organization names from postgres.");

    Json(json!({
        "status": StatusCode::OK.as_u16(),
        "tenant": tenant,
        "results": vsa_organizations_result,
    }))
}
