use std::{collections::HashMap, time::Duration};

use axum::Json;
use dotenv::dotenv;
use reqwest::StatusCode;
use serde_json::json;
use sqlx::postgres::PgPoolOptions;

use crate::{
    functions::vsa::{
        agents::agents,
        devices::{devices, VsaDevice},
        disks::{disks, VsaDisk},
        groups::{groups, VsaGroup},
        patches::{patches, VsaPatch},
        security_products::{security_products, VsaSecurityProduct},
    },
    models::vsa::agent::VsaAgent,
};

pub async fn sync_vsa() {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost".into());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&database_url)
        .await
        .expect("Failed to connect to postgres.");

    println!("Syncing VSA agents.");

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

    println!(
        "{:?}",
        Json(json!({
            "status": StatusCode::OK.as_u16(),
            "inserted": inserted,
            "skipped": skipped
        }))
    );

    println!("Syncing VSA devices.");

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
                        "UPDATE vsa_agents SET system_serial_number = $1, system_age = $2, cpu_speed = $3, cpu_count = $4, ram_size_in_mbytes = $5 WHERE id = $6",
                        device.system_serial_number.unwrap_or_else(|| "Unknown".to_string()),
                        device.bios_release_date.unwrap_or_else(|| "Unknown".to_string()),
                        device.cpu_speed.unwrap_or_else(|| 0.0),
                        device.cpu_count.unwrap_or_else(|| 0.0),
                        device.ram_size_in_mbytes.unwrap_or_else(|| 0.0),
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

    println!(
        "{:?}",
        Json(json!({
            "status": StatusCode::OK.as_u16(),
            "updated": updated,
            "skipped": skipped
        }))
    );

    println!("Syncing VSA disks.");

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

    println!(
        "{:?}",
        Json(json!({
            "status": StatusCode::OK.as_u16(),
            "updated": updated,
            "skipped": skipped
        }))
    );

    println!("Syncing VSA groups.");

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

    println!(
        "{:?}",
        Json(json!({
            "status": StatusCode::OK.as_u16(),
            "updated": updated,
            "skipped": skipped
        }))
    );

    println!("Syncing VSA security-products.");

    let products: Vec<VsaSecurityProduct> = security_products().await;
    let mut valid_antivirus_products: Vec<VsaSecurityProduct> = Vec::new();

    for product in products {
        if product.product_type == "AntiVirus" && product.is_active == 1 {
            valid_antivirus_products.push(product)
        }
    }

    let mut updated = 0;
    let mut skipped = 0;

    for valid_product in valid_antivirus_products {
        let product_agent_result = sqlx::query_as!(
            VsaAgent,
            "SELECT * FROM vsa_agents WHERE id = $1;",
            valid_product.agent_id
        )
        .fetch_one(&pool)
        .await;

        match product_agent_result {
            Ok(product_agent) => {
                sqlx::query_as!(
                    VsaAgent,
                    "UPDATE vsa_agents SET anti_virus = $1 WHERE id = $2",
                    true,
                    product_agent.id
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

    println!(
        "{:?}",
        Json(json!({
            "status": StatusCode::OK.as_u16(),
            "updated": updated,
            "skipped": skipped
        }))
    );

    println!("Syncing VSA patches.");

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

    println!(
        "{:?}",
        Json(json!({
            "status": StatusCode::OK.as_u16(),
            "updated": updated,
            "skipped": skipped
        }))
    );

    println!("Finished VSA sync.");
}
