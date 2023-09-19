use axum::{extract::State, response::IntoResponse, Json};
use reqwest::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use crate::{functions::rocketcyber::agents::agents, models::rocketcyber::agent::RocketAgent};

pub async fn index(State(pool): State<PgPool>) -> impl IntoResponse {
    let agents: Vec<RocketAgent> =
        sqlx::query_as!(RocketAgent, "SELECT * FROM rocketcyber_agents;")
            .fetch_all(&pool)
            .await
            .expect("Failed to get rocketcyber agents from postgres.");

    Json(agents)
}

pub async fn import(State(pool): State<PgPool>) -> impl IntoResponse {
    let agents = agents()
        .await
        .expect("Failed to get rocketcyber agents from rocketcyber api.");

    let mut inserted = 0;
    let mut skipped = 0;

    for agent in agents {
        let agent_id = agent.id.unwrap();

        let existing_agent_result = sqlx::query_as!(
            RocketAgent,
            "SELECT * FROM rocketcyber_agents WHERE id = $1;",
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
                    RocketAgent,
                    "INSERT INTO rocketcyber_agents VALUES ($1,$2,$3,$4,$5);",
                    agent_id,
                    agent.customer_id.unwrap(),
                    agent.hostname.unwrap(),
                    agent.account_path.unwrap(),
                    agent.agent_version.unwrap()
                )
                .execute(&pool)
                .await
                .expect("Failed to insert rocketcyber agent into postgres database.");

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
