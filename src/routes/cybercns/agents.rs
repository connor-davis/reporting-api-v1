use axum::{extract::State, response::IntoResponse, Json};
use reqwest::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use crate::{functions::cybercns::agents::agents, models::cybercns::agent::CyberAgent};

#[utoipa::path(get, path = "/cyber-cns/agents", responses((status = 200, description = "List all CyberCNS agents from api database.")), tag = "CyberCNS")]
pub async fn index(State(pool): State<PgPool>) -> impl IntoResponse {
    let agents: Vec<CyberAgent> = sqlx::query_as!(CyberAgent, "SELECT * FROM cybercns_agents;")
        .fetch_all(&pool)
        .await
        .expect("Failed to get rocketcyber agents from postgres.");

    Json(agents)
}

#[utoipa::path(post, path = "/cyber-cns/agents", responses((status = 200, description = "Import CyberCNS agents from the CyberCNS api.")), tag = "CyberCNS")]
pub async fn import(State(pool): State<PgPool>) -> impl IntoResponse {
    let agents = agents()
        .await
        .expect("Failed to get cybercns agents from cybercns api.");

    let mut inserted = 0;
    let mut skipped = 0;

    for agent in agents {
        let agent_id = agent.id.unwrap();

        let existing_agent_result = sqlx::query_as!(
            CyberAgent,
            "SELECT * FROM cybercns_agents WHERE id = $1;",
            agent_id
        )
        .fetch_one(&pool)
        .await;

        match existing_agent_result {
            Ok(_) => {
                skipped += 1;
            }
            Err(_) => {
                if !agent.host_name.is_none() {
                    sqlx::query_as!(
                        CyberAgent,
                        "INSERT INTO cybercns_agents VALUES ($1,$2);",
                        agent_id,
                        agent.host_name.unwrap()
                    )
                    .execute(&pool)
                    .await
                    .expect("Failed to insert cybercns agent into postgres database.");

                    inserted += 1;
                } else {
                    sqlx::query_as!(
                        CyberAgent,
                        "INSERT INTO cybercns_agents VALUES ($1,$2);",
                        agent_id,
                        "Unknown"
                    )
                    .execute(&pool)
                    .await
                    .expect("Failed to insert cybercns agent into postgres database.");

                    inserted += 1;
                }
            }
        }
    }

    Json(json!({
        "status": StatusCode::OK.as_u16(),
        "inserted": inserted,
        "skipped": skipped
    }))
}
