use axum::{extract::State, response::IntoResponse, Json};
use reqwest::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use crate::{functions::vsa::agents::agents, models::vsa::agent::VsaAgent};

pub async fn index(State(pool): State<PgPool>) -> impl IntoResponse {
    let agents: Vec<VsaAgent> = sqlx::query_as!(VsaAgent, "SELECT * FROM vsa_agents;")
        .fetch_all(&pool)
        .await
        .expect("Failed to get vsa agents from postgres.");

    Json(agents)
}

pub async fn import(State(pool): State<PgPool>) -> impl IntoResponse {
    let agents = agents().await;

    let mut inserted = 0;
    let mut skipped = 0;

    for agent in agents {
        let agent_id = agent.agent_id;

        let existing_agent_result = sqlx::query_as!(
            VsaAgent,
            "SELECT * FROM vsa_agents WHERE id = $1;",
            agent_id
        )
        .fetch_one(&pool)
        .await;

        match existing_agent_result {
            Ok(_) => {
                skipped += 1;
            }
            Err(_) => {
                sqlx::query_as!(
                    VsaAgent,
                    "INSERT INTO vsa_agents (id, agent_name, computer_name, ip_address, os_name, group_id) VALUES ($1, $2, $3, $4, $5, $6)",
                    agent_id,
                    agent.agent_name,
                    agent.computer_name,
                    agent.ip_address,
                    agent.operating_system_info,
                    agent.machine_group
                )
                .execute(&pool)
                .await
                .expect("Failed to insert agent into postgres database.");

                inserted += 1;
            }
        }
    }

    Json(json!({
        "status": StatusCode::OK.as_u16(),
        "inserted": inserted,
        "skipped": skipped
    }))
}
