use axum::{extract::State, response::IntoResponse, Json};
use reqwest::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use crate::{
    functions::vsa::devices::{devices, VsaDevice},
    models::vsa::agent::VsaAgent,
};

pub async fn index() -> impl IntoResponse {
    let devices = devices().await;

    Json(devices)
}

pub async fn import(State(pool): State<PgPool>) -> impl IntoResponse {
    let devices: Vec<VsaDevice> = devices().await;

    let mut updated = 0;
    let mut skipped = 0;

    for device in devices {
        let device_agent_result = sqlx::query_as!(
            VsaAgent,
            "SELECT * FROM vsa_agents WHERE id = $1;",
            device.agent_id.unwrap()
        )
        .fetch_one(&pool)
        .await;

        match device_agent_result {
            Ok(device_agent) => {
                sqlx::query_as!(
                        VsaAgent,
                        "UPDATE vsa_agents SET system_serial_number = $1, system_age = $2 WHERE id = $3",
                        device.system_serial_number.unwrap_or_else(|| "Unknown".to_string()),
                        device.bios_release_date.unwrap_or_else(|| "Unknown".to_string()),
                        device_agent.id
                    )
                    .execute(&pool)
                    .await
                    .expect("Failed to update vsa agent in postgres database.");

                updated += 1;
            }
            Err(_) => {
                skipped += 1;
            }
        }
    }

    Json(json!({
        "status": StatusCode::OK.as_u16(),
        "updated": updated,
        "skipped": skipped
    }))
}
