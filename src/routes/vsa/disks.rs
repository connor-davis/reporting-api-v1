use std::collections::HashMap;

use axum::{extract::State, response::IntoResponse, Json};
use reqwest::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use crate::{
    functions::vsa::disks::{disks, VsaDisk},
    models::vsa::agent::VsaAgent,
};

pub async fn index() -> impl IntoResponse {
    let disks = disks().await;

    Json(disks)
}

pub async fn import(State(pool): State<PgPool>) -> impl IntoResponse {
    let disks: Vec<VsaDisk> = disks().await;

    let mut updated = 0;
    let mut skipped = 0;

    let mut disks_map: HashMap<String, VsaDisk> = HashMap::new();

    for disk in disks {
        let agent_id = disk.agent_id.to_owned().unwrap();
        let agent_id_insert = agent_id.clone();

        let mapped_disk_found = disks_map.get_mut(&agent_id);

        if mapped_disk_found.is_some() {
            let mapped_disk = mapped_disk_found.unwrap();

            let current_free_space = mapped_disk.free_space_in_gbytes.unwrap();
            let new_free_space = disk.free_space_in_gbytes.unwrap();
            let calculated_free_space = current_free_space + new_free_space;

            let current_used_space = mapped_disk.used_space_in_gbytes.unwrap();
            let new_used_space = disk.used_space_in_gbytes.unwrap();
            let calculated_used_space = current_used_space + new_used_space;

            let current_total_size = mapped_disk.total_size_in_gbytes.unwrap();
            let new_total_size = disk.total_size_in_gbytes.unwrap();
            let calculated_total_size = current_total_size + new_total_size;

            disks_map.insert(
                agent_id,
                VsaDisk {
                    agent_id: Some(agent_id_insert),
                    free_space_in_gbytes: Some(calculated_free_space),
                    used_space_in_gbytes: Some(calculated_used_space),
                    total_size_in_gbytes: Some(calculated_total_size),
                },
            );
        } else {
            disks_map.insert(agent_id, disk);
        }
    }

    for disk in disks_map.values() {
        let agent_id = disk.agent_id.clone();

        let disk_agent_result = sqlx::query_as!(
            VsaAgent,
            "SELECT * FROM vsa_agents WHERE id = $1;",
            agent_id
        )
        .fetch_one(&pool)
        .await;

        match disk_agent_result {
            Ok(device_agent) => {
                sqlx::query_as!(
                        VsaAgent,
                        "UPDATE vsa_agents SET free_space_in_gbytes = $1, used_space_in_gbytes = $2, total_size_in_gbytes = $3 WHERE id = $4;",
                        disk.free_space_in_gbytes.unwrap_or_else(|| 0.0),
                        disk.used_space_in_gbytes.unwrap_or_else(|| 0.0),
                        disk.total_size_in_gbytes.unwrap_or_else(|| 0.0),
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
