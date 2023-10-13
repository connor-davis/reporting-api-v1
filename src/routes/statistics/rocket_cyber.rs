use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use reqwest::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use crate::models::rocketcyber::account::RocketAccount;

pub async fn index(
    Query(params): Query<Vec<(String, String)>>,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    let tenant = &params[0].1;

    let account = sqlx::query_as!(
        RocketAccount,
        r#"
            SELECT
                *
            FROM rocketcyber_accounts
            WHERE LOWER(TRIM(TRAILING ' (Pty) Ltd' FROM account_name)) = LOWER($1)
        "#,
        tenant
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to find rocket cyber account in postgres.");

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

    Json(json!({
        "status": StatusCode::OK.as_u16(),
        "tenant": tenant,
        "total_agents": total_agents,
        "total_incidents": total_incidents
    }))
}
