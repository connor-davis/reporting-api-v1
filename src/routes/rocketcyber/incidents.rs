use axum::{extract::State, response::IntoResponse, Json};
use reqwest::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use crate::{
    functions::rocketcyber::incidents::incidents, models::rocketcyber::incident::RocketIncident,
};

pub async fn index(State(pool): State<PgPool>) -> impl IntoResponse {
    let incidents: Vec<RocketIncident> =
        sqlx::query_as!(RocketIncident, "SELECT * FROM rocketcyber_incidents;")
            .fetch_all(&pool)
            .await
            .expect("Failed to get rocketcyber agents from postgres.");

    Json(incidents)
}

pub async fn import(State(pool): State<PgPool>) -> impl IntoResponse {
    let incidents = incidents()
        .await
        .expect("Failed to get rocketcyber incidents from rocketcyber api.");

    sqlx::query_as!(RocketIncident, "TRUNCATE rocketcyber_incidents;")
        .execute(&pool)
        .await
        .expect("Failed to remove old incidents from postgres database.");

    let mut inserted = 0;

    for incident in incidents {
        let incident_id = incident.id.unwrap();

        sqlx::query_as!(
            RocketIncident,
            "INSERT INTO rocketcyber_incidents VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10);",
            incident_id,
            incident.title.unwrap(),
            incident.description.unwrap(),
            incident.remediation.unwrap(),
            incident.resolved_at,
            incident.published_at,
            incident.created_at,
            incident.status.unwrap(),
            incident.account_id.unwrap(),
            incident.event_count.unwrap()
        )
        .execute(&pool)
        .await
        .expect("Failed to insert rocketcyber incident into postgres database.");

        inserted += 1;
    }

    Json(json!({
        "status": StatusCode::OK.as_u16(),
        "inserted": inserted
    }))
}
