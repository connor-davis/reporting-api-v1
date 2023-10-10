use axum::{extract::State, response::IntoResponse, Json};
use reqwest::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use crate::{
    functions::vsa::patches::{patches, VsaPatch},
    models::vsa::agent::VsaAgent,
};

pub async fn index() -> impl IntoResponse {
    let patches = patches().await;

    Json(patches)
}

pub async fn import(State(pool): State<PgPool>) -> impl IntoResponse {
    let patches: Vec<VsaPatch> = patches().await;

    let mut updated = 0;
    let mut skipped = 0;

    for patch in patches {
        let patch_agent_result = sqlx::query_as!(
            VsaAgent,
            "SELECT * FROM vsa_agents WHERE id = $1;",
            patch.agent_id
        )
        .fetch_one(&pool)
        .await;

        match patch_agent_result {
            Ok(patch_agent) => {
                sqlx::query_as!(
                    VsaAgent,
                    "UPDATE vsa_agents SET total_patches = $1, installed_patches = $2, last_patch = $3, next_patch = $4 WHERE id = $5;",
                    patch.total_patches_reported_by_scan,
                    patch.installed_patches_reported_by_scan,
                    patch.last_patch_scan_date,
                    patch.next_patch_scan_date,
                    patch_agent.id
                ).execute(&pool).await.expect("Failed to update vsa agent in postgres database.");

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
