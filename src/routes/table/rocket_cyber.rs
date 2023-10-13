use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use math::round::ceil;
use reqwest::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use crate::models::rocketcyber::{account::RocketAccount, incident::RocketIncident};

pub async fn index(
    Query(params): Query<Vec<(String, String)>>,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    let tenant = &params[0].1;
    let offset = (&params[1].1.parse::<i64>().unwrap_or(1) - 1) * 10;

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
                    ORDER BY incident.title OFFSET $2 LIMIT 10;
                "#,
                account.account_id,
                offset
            )
            .fetch_all(&pool)
            .await
            .expect("Failed to get total results from postgres.");

            println!("Incidents: {:?}", incidents_result);

            let mut total_pages_result = sqlx::query_scalar!(
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

            if total_pages_result.unwrap_or(10) < 10 {
                total_pages_result = Some(10);
            }

            let total_pages = ceil(total_pages_result.unwrap_or(10) as f64 / 10 as f64, 0);

            Json(json!({
                "status": StatusCode::OK.as_u16(),
                "tenant": tenant,
                "results": incidents_result,
                "total_pages": total_pages
            }))
        }
        Err(_) => Json(json!({
            "status": StatusCode::OK.as_u16(),
            "tenant": tenant,
            "results": [],
            "total_pages": 0
        })),
    }
}
