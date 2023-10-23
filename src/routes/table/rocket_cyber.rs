use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use reqwest::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use crate::models::rocketcyber::{account::RocketAccount, incident::RocketIncident};

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
            WHERE similarity(LOWER(account_name), LOWER($1)) >= 0.6
        "#,
        tenant
    )
    .fetch_one(&pool)
    .await;

    match account {
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

            Json(json!({
                "status": StatusCode::OK.as_u16(),
                "tenant": tenant,
                "results": incidents_result
            }))
        }
        Err(_) => Json(json!({
            "status": StatusCode::OK.as_u16(),
            "tenant": tenant,
            "results": []
        })),
    }
}
