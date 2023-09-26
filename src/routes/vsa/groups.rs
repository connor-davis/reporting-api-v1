use axum::{extract::State, response::IntoResponse, Json};
use reqwest::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use crate::{
    functions::vsa::groups::{groups, VsaGroup},
    models::vsa::agent::VsaAgent,
};

pub async fn index() -> impl IntoResponse {
    let groups = groups().await;

    Json(groups)
}

pub async fn import(State(pool): State<PgPool>) -> impl IntoResponse {
    let groups: Vec<VsaGroup> = groups().await;

    let mut updated = 0;
    let mut skipped = 0;

    for group in groups {
        let group_agents_result = sqlx::query_as!(
            VsaAgent,
            "SELECT * FROM vsa_agents WHERE group_id = $1;",
            group.reverse_group_id.unwrap()
        )
        .fetch_all(&pool)
        .await;

        match group_agents_result {
            Ok(group_agents) => {
                for group_agent in group_agents {
                    sqlx::query_as!(
                        VsaAgent,
                        "UPDATE vsa_agents SET organization_name = $1 WHERE id = $2",
                        group.organization_name,
                        group_agent.id
                    )
                    .execute(&pool)
                    .await
                    .expect("Failed to update vsa agent in postgres database.");

                    updated += 1;
                }
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
