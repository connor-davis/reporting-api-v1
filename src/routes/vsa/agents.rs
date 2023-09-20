use axum::{extract::State, response::IntoResponse, Json};
use reqwest::StatusCode;
use serde_json::json;
use sqlx::{PgPool, QueryBuilder};

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

    let mut new_agents: Vec<VsaAgent> = Vec::new();

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
                new_agents.push(VsaAgent {
                    id: agent_id,
                    agent_name: Some(agent.agent_name),
                    computer_name: Some(agent.computer_name),
                    ip_address: Some(agent.ip_address),
                    anti_virus: agent.anti_virus,
                    system_serial_number: agent.system_serial_number,
                    system_age: agent.system_age,
                    free_space_in_gbytes: agent.free_space_in_gbytes,
                    used_space_in_gbytes: agent.used_space_in_gbytes,
                    total_size_in_gbytes: agent.total_size_in_gbytes,
                });

                inserted += 1;
            }
        }
    }

    let mut query_builder =
        QueryBuilder::new("INSERT INTO vsa_agents (id, agent_name, computer_name, ip_address) ");

    query_builder.push_values(new_agents, |mut b, new_agent| {
        b.push_bind(new_agent.id)
            .push_bind(new_agent.agent_name)
            .push_bind(new_agent.computer_name)
            .push_bind(new_agent.ip_address);
    });

    let query = query_builder.build();

    query
        .execute(&pool)
        .await
        .expect("Failed to bulk insert new agents to postgres database.");

    Json(json!({
        "status": StatusCode::OK.as_u16(),
        "inserted": inserted,
        "skipped": skipped
    }))
}
