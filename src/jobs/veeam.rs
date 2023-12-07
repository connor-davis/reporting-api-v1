use std::time::Duration;

use anyhow::Result;
use chrono::Utc;
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;

use crate::functions::veeam::{
    veeam_agents::veeam_agents, veeam_agents_jobs::veeam_agents_jobs, veeam_servers::veeam_servers,
    veeam_servers_jobs::veeam_server_jobs,
};

pub async fn sync_veeam() -> Result<()> {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost".into());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&database_url)
        .await
        .expect("Failed to connect to postgres.");

    println!("Syncing Veeam backups.");

    let mut skipped = 0;
    let mut updated = 0;
    let mut inserted = 0;
    let veeam_agents = veeam_agents().await;

    match veeam_agents {
        Ok(veeam_agents) => {
            for veeam_agent in veeam_agents {
                let existing_veeam_agent = sqlx::query!(
                    "SELECT * FROM veeam_agents WHERE name = $1;",
                    veeam_agent.name
                )
                .fetch_optional(&pool)
                .await
                .expect("Failed to get user from postgres.");

                match existing_veeam_agent {
                    Some(existing_veeam_agent) => {
                        // Update the existing veeam agent with the new details
                        let update_result = sqlx::query!(
                            "UPDATE veeam_agents SET
                                instance_uid = $1,
                                agent_platform = $2,
                                status = $3,
                                management_agent_uid = $4,
                                site_uid = $5,
                                organization_uid = $6,
                                operation_mode = $7,
                                gui_mode = $8,
                                platform = $9,
                                version = $10,
                                activation_time = $11,
                                management_mode = $12,
                                installation_type = $13,
                                total_jobs_count = $14,
                                running_jobs_count = $15,
                                success_jobs_count = $16,
                                company_name = $17
                            WHERE name = $18;",
                            veeam_agent.instance_uid.unwrap_or("N/A".to_string()),
                            veeam_agent.agent_platform.unwrap_or("N/A".to_string()),
                            veeam_agent.status.unwrap_or("N/A".to_string()),
                            veeam_agent
                                .management_agent_uid
                                .unwrap_or("N/A".to_string()),
                            veeam_agent.site_uid.unwrap_or("N/A".to_string()),
                            veeam_agent.organization_uid.unwrap_or("N/A".to_string()),
                            veeam_agent.operation_mode.unwrap_or("N/A".to_string()),
                            veeam_agent.gui_mode.unwrap_or("N/A".to_string()),
                            veeam_agent.platform.unwrap_or("N/A".to_string()),
                            veeam_agent.version.unwrap_or("N/A".to_string()),
                            veeam_agent.activation_time.unwrap_or(Utc::now()),
                            veeam_agent.management_mode.unwrap_or("N/A".to_string()),
                            veeam_agent.installation_type.unwrap_or("N/A".to_string()),
                            veeam_agent.total_jobs_count.unwrap_or(0),
                            veeam_agent.running_jobs_count.unwrap_or(0),
                            veeam_agent.success_jobs_count.unwrap_or(0),
                            veeam_agent.company_name.unwrap_or("N/A".to_string()),
                            existing_veeam_agent.name
                        )
                        .execute(&pool)
                        .await;

                        match update_result {
                            Ok(_) => {
                                updated += 1;
                            }
                            Err(error) => {
                                println!("Failed to update Veeam Agent: {:?}", error);

                                skipped += 1;
                            }
                        }
                    }
                    None => {
                        if veeam_agent.name.is_none() {
                            skipped += 1;

                            continue;
                        }

                        let insert_result = sqlx::query!(
                            "INSERT INTO veeam_agents (
                                instance_uid,
                                agent_platform, 
                                status, 
                                management_agent_uid, 
                                site_uid, 
                                organization_uid, 
                                name, 
                                operation_mode, 
                                gui_mode, 
                                platform, 
                                version, 
                                activation_time, 
                                management_mode, 
                                installation_type, 
                                total_jobs_count, 
                                running_jobs_count, 
                                success_jobs_count, 
                                company_name
                            ) 
                            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18);",
                            veeam_agent.instance_uid.unwrap_or("N/A".to_string()),
                            veeam_agent.agent_platform.unwrap_or("N/A".to_string()),
                            veeam_agent.status.unwrap_or("N/A".to_string()),
                            veeam_agent.management_agent_uid.unwrap_or("N/A".to_string()),
                            veeam_agent.site_uid.unwrap_or("N/A".to_string()),
                            veeam_agent.organization_uid.unwrap_or("N/A".to_string()),
                            veeam_agent.name.unwrap(),
                            veeam_agent.operation_mode.unwrap_or("N/A".to_string()),
                            veeam_agent.gui_mode.unwrap_or("N/A".to_string()),
                            veeam_agent.platform.unwrap_or("N/A".to_string()),
                            veeam_agent.version.unwrap_or("N/A".to_string()),
                            veeam_agent.activation_time.unwrap_or(Utc::now()),
                            veeam_agent.management_mode.unwrap_or("N/A".to_string()),
                            veeam_agent.installation_type.unwrap_or("N/A".to_string()),
                            veeam_agent.total_jobs_count.unwrap_or(0),
                            veeam_agent.running_jobs_count.unwrap_or(0),
                            veeam_agent.success_jobs_count.unwrap_or(0),
                            veeam_agent.company_name.unwrap_or("N/A".to_string())
                        )
                        .execute(&pool)
                        .await;

                        match insert_result {
                            Ok(_) => {
                                inserted += 1;
                            }
                            Err(error) => {
                                println!("Failed to insert Veeam Agent: {:?}", error);

                                skipped += 1;
                            }
                        }
                    }
                }
            }
        }
        Err(error) => {
            println!("Failed to get Veeam Agents: {:?}", error);
        }
    }

    println!(
        "Finished syncing Veeam Agents. Inserted: {}, Skipped: {}, Updated: {}",
        inserted, skipped, updated
    );

    println!("Syncing Veeam Agent Jobs.");

    let mut skipped = 0;
    let mut updated = 0;
    let mut inserted = 0;

    let veeam_agent_jobs = veeam_agents_jobs().await;

    match veeam_agent_jobs {
        Ok(veeam_agents_jobs) => {
            for veeam_agent_job in veeam_agents_jobs {
                let existing_veeam_agent_job = sqlx::query!(
                    "SELECT * FROM veeam_agents_jobs WHERE name = $1;",
                    veeam_agent_job.name
                )
                .fetch_optional(&pool)
                .await
                .expect("Failed to get user from postgres.");

                match existing_veeam_agent_job {
                    Some(existing_veeam_agent_job) => {
                        // Update the existing veeam agent with the new details
                        let update_result = sqlx::query!(
                            "UPDATE veeam_agents_jobs SET 
                                instance_uid = $1,
                                backup_agent_uid = $2, 
                                organization_uid = $3,
                                description = $4,
                                config_uid = $5,
                                system_type = $6,
                                backup_policy_uid = $7,
                                backup_policy_failure_message = $8,
                                status = $9,
                                operation_mode = $10,
                                destination = $11,
                                restore_points = $12,
                                last_run = $13,
                                last_end_time = $14,
                                last_duration = $15,
                                next_run = $16,
                                avg_duration = $17,
                                backup_mode = $18,
                                target_type = $19,
                                is_enabled = $20,
                                schedule_type = $21,
                                failure_message = $22,
                                backed_up_size = $23,
                                company_name = $24 
                            WHERE name = $25;",
                            veeam_agent_job.instance_uid.unwrap_or("N/A".to_string()),
                            veeam_agent_job
                                .backup_agent_uid
                                .unwrap_or("N/A".to_string()),
                            veeam_agent_job
                                .organization_uid
                                .unwrap_or("N/A".to_string()),
                            veeam_agent_job.description.unwrap_or("N/A".to_string()),
                            veeam_agent_job.config_uid.unwrap_or("N/A".to_string()),
                            veeam_agent_job.system_type.unwrap_or("N/A".to_string()),
                            veeam_agent_job
                                .backup_policy_uid
                                .unwrap_or("N/A".to_string()),
                            veeam_agent_job
                                .backup_policy_failure_message
                                .unwrap_or("N/A".to_string()),
                            veeam_agent_job.status.unwrap_or("N/A".to_string()),
                            veeam_agent_job.operation_mode.unwrap_or("N/A".to_string()),
                            veeam_agent_job.destination.unwrap_or("N/A".to_string()),
                            veeam_agent_job.restore_points.unwrap_or(0),
                            veeam_agent_job.last_run.unwrap_or(Utc::now()),
                            veeam_agent_job.last_end_time.unwrap_or(Utc::now()),
                            veeam_agent_job.last_duration.unwrap_or(0),
                            veeam_agent_job.next_run.unwrap_or(Utc::now()),
                            veeam_agent_job.avg_duration.unwrap_or(0),
                            veeam_agent_job.backup_mode.unwrap_or("N/A".to_string()),
                            veeam_agent_job.target_type.unwrap_or("N/A".to_string()),
                            veeam_agent_job.is_enabled.unwrap_or(false),
                            veeam_agent_job.schedule_type.unwrap_or("N/A".to_string()),
                            veeam_agent_job.failure_message.unwrap_or("N/A".to_string()),
                            veeam_agent_job.backed_up_size.unwrap_or(0),
                            veeam_agent_job.company_name.unwrap_or("N/A".to_string()),
                            existing_veeam_agent_job.name
                        )
                        .execute(&pool)
                        .await;

                        match update_result {
                            Ok(_) => {
                                updated += 1;
                            }
                            Err(error) => {
                                println!("Failed to update Veeam Agent Job: {:?}", error);

                                skipped += 1;
                            }
                        }
                    }
                    None => {
                        if veeam_agent_job.name.is_none() {
                            skipped += 1;

                            continue;
                        }

                        let insert_result = sqlx::query!(
                            "INSERT INTO veeam_agents_jobs (
                                instance_uid,
                                backup_agent_uid, 
                                organization_uid, 
                                name, 
                                description, 
                                config_uid, 
                                system_type, 
                                backup_policy_uid, 
                                backup_policy_failure_message, 
                                status, 
                                operation_mode, 
                                destination, 
                                restore_points, 
                                last_run, 
                                last_end_time, 
                                last_duration, 
                                next_run, 
                                avg_duration, 
                                backup_mode, 
                                target_type, 
                                is_enabled, 
                                schedule_type, 
                                failure_message, 
                                backed_up_size, 
                                company_name
                            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 
                                $11, $12, $13, $14, $15, $16, $17, $18, $19, 
                                $20, $21, $22, $23, $24, $25);",
                            veeam_agent_job.instance_uid.unwrap_or("N/A".to_string()),
                            veeam_agent_job
                                .backup_agent_uid
                                .unwrap_or("N/A".to_string()),
                            veeam_agent_job
                                .organization_uid
                                .unwrap_or("N/A".to_string()),
                            veeam_agent_job.name.unwrap(),
                            veeam_agent_job.description.unwrap_or("N/A".to_string()),
                            veeam_agent_job.config_uid.unwrap_or("N/A".to_string()),
                            veeam_agent_job.system_type.unwrap_or("N/A".to_string()),
                            veeam_agent_job
                                .backup_policy_uid
                                .unwrap_or("N/A".to_string()),
                            veeam_agent_job
                                .backup_policy_failure_message
                                .unwrap_or("N/A".to_string()),
                            veeam_agent_job.status.unwrap_or("N/A".to_string()),
                            veeam_agent_job.operation_mode.unwrap_or("N/A".to_string()),
                            veeam_agent_job.destination.unwrap_or("N/A".to_string()),
                            veeam_agent_job.restore_points.unwrap_or(0),
                            veeam_agent_job.last_run.unwrap_or(Utc::now()),
                            veeam_agent_job.last_end_time.unwrap_or(Utc::now()),
                            veeam_agent_job.last_duration.unwrap_or(0),
                            veeam_agent_job.next_run.unwrap_or(Utc::now()),
                            veeam_agent_job.avg_duration.unwrap_or(0),
                            veeam_agent_job.backup_mode.unwrap_or("N/A".to_string()),
                            veeam_agent_job.target_type.unwrap_or("N/A".to_string()),
                            veeam_agent_job.is_enabled.unwrap_or(false),
                            veeam_agent_job.schedule_type.unwrap_or("N/A".to_string()),
                            veeam_agent_job.failure_message.unwrap_or("N/A".to_string()),
                            veeam_agent_job.backed_up_size.unwrap_or(0),
                            veeam_agent_job.company_name.unwrap_or("N/A".to_string())
                        )
                        .execute(&pool)
                        .await;

                        match insert_result {
                            Ok(_) => {
                                inserted += 1;
                            }
                            Err(error) => {
                                println!("Failed to insert Veeam Agent Job: {:?}", error);

                                skipped += 1;
                            }
                        }
                    }
                }
            }
        }
        Err(error) => {
            println!("Failed to get Veeam Agent Jobs: {:?}", error);
        }
    }

    println!(
        "Finished syncing Veeam Agent Jobs. Inserted: {}, Skipped: {}, Updated: {}",
        inserted, skipped, updated
    );

    println!("Syncing Veeam Servers.");

    let mut skipped = 0;
    let mut updated = 0;
    let mut inserted = 0;

    let veeam_servers = veeam_servers().await;

    match veeam_servers {
        Ok(veeam_servers) => {
            for veeam_server in veeam_servers {
                let existing_veeam_server = sqlx::query!(
                    "SELECT * FROM veeam_servers WHERE name = $1;",
                    veeam_server.name
                )
                .fetch_optional(&pool)
                .await
                .expect("Failed to get user from postgres.");

                match existing_veeam_server {
                    Some(existing_veeam_server) => {
                        // Update the existing veeam server with the new details
                        let update_result = sqlx::query!(
                            "UPDATE veeam_servers SET
                                instance_uid = $1,
                                organization_uid = $2,
                                location_uid = $3,
                                management_agent_uid = $4,
                                version = $5,
                                display_version = $6,
                                installation_uid = $7,
                                backup_server_role_type = $8,
                                status = $9,
                                company_name = $10
                            WHERE name = $11;",
                            veeam_server.instance_uid.unwrap_or("N/A".to_string()),
                            veeam_server.organization_uid.unwrap_or("N/A".to_string()),
                            veeam_server.location_uid.unwrap_or("N/A".to_string()),
                            veeam_server
                                .management_agent_uid
                                .unwrap_or("N/A".to_string()),
                            veeam_server.version.unwrap_or("N/A".to_string()),
                            veeam_server.display_version.unwrap_or("N/A".to_string()),
                            veeam_server.installation_uid.unwrap_or("N/A".to_string()),
                            veeam_server
                                .backup_server_role_type
                                .unwrap_or("N/A".to_string()),
                            veeam_server.status.unwrap_or("N/A".to_string()),
                            veeam_server.company_name.unwrap_or("N/A".to_string()),
                            existing_veeam_server.name
                        )
                        .execute(&pool)
                        .await;

                        match update_result {
                            Ok(_) => {
                                updated += 1;
                            }
                            Err(error) => {
                                println!("Failed to update Veeam Server: {:?}", error);

                                skipped += 1;
                            }
                        }
                    }
                    None => {
                        if veeam_server.name.is_none() {
                            skipped += 1;

                            continue;
                        }

                        let insert_result = sqlx::query!(
                            r#"
                                INSERT INTO
                                    veeam_servers
                                (
                                    instance_uid,
                                    name,
                                    organization_uid,
                                    location_uid,
                                    management_agent_uid,
                                    version,
                                    display_version,
                                    installation_uid,
                                    backup_server_role_type,
                                    status,
                                    company_name
                                )
                                VALUES
                                (
                                    $1,
                                    $2,
                                    $3,
                                    $4,
                                    $5,
                                    $6,
                                    $7,
                                    $8,
                                    $9,
                                    $10,
                                    $11
                                );
                            "#,
                            veeam_server.instance_uid.unwrap_or("N/A".to_string()),
                            veeam_server.name.unwrap(),
                            veeam_server.organization_uid.unwrap_or("N/A".to_string()),
                            veeam_server.location_uid.unwrap_or("N/A".to_string()),
                            veeam_server
                                .management_agent_uid
                                .unwrap_or("N/A".to_string()),
                            veeam_server.version.unwrap_or("N/A".to_string()),
                            veeam_server.display_version.unwrap_or("N/A".to_string()),
                            veeam_server.installation_uid.unwrap_or("N/A".to_string()),
                            veeam_server
                                .backup_server_role_type
                                .unwrap_or("N/A".to_string()),
                            veeam_server.status.unwrap_or("N/A".to_string()),
                            veeam_server.company_name.unwrap_or("N/A".to_string())
                        )
                        .execute(&pool)
                        .await;

                        match insert_result {
                            Ok(_) => {
                                inserted += 1;
                            }
                            Err(error) => {
                                println!("Failed to insert Veeam Server: {:?}", error);

                                skipped += 1;
                            }
                        }
                    }
                }
            }
        }
        Err(error) => {
            println!("Failed to get Veeam Servers: {:?}", error);
        }
    }

    println!(
        "Finished syncing Veeam Servers. Inserted: {}, Skipped: {}, Updated: {}",
        inserted, skipped, updated
    );

    println!("Syncing Veeam Server Jobs.");

    let mut skipped = 0;
    let mut updated = 0;
    let mut inserted = 0;

    let veeam_server_jobs = veeam_server_jobs().await;

    match veeam_server_jobs {
        Ok(veeam_server_jobs) => {
            for veeam_server_job in veeam_server_jobs {
                let existing_veeam_server_job = sqlx::query!(
                    "SELECT * FROM veeam_servers_jobs WHERE name = $1;",
                    veeam_server_job.name
                )
                .fetch_optional(&pool)
                .await
                .expect("Failed to get user from postgres.");

                match existing_veeam_server_job {
                    Some(existing_veeam_server_job) => {
                        // Update the existing veeam server with the new details
                        let update_result = sqlx::query!(
                            "UPDATE veeam_servers_jobs SET
                                instance_uid = $1,
                                backup_server_uid = $2,
                                location_uid = $3,
                                site_uid = $4,
                                organization_uid = $5,
                                status = $6,
                                type = $7,
                                last_run = $8,
                                last_end_time = $9,
                                last_duration = $10,
                                processing_rate = $11,
                                avg_duration = $12,
                                transferred_data = $13,
                                bottleneck = $14,
                                is_enabled = $15,
                                schedule_type = $16,
                                failure_message = $17,
                                target_type = $18,
                                destination = $19,
                                retention_limit = $20,
                                retention_limit_type = $21,
                                is_gfs_option_enabled = $22,
                                company_name = $23
                            WHERE name = $24;",
                            veeam_server_job.instance_uid.unwrap_or("N/A".to_string()),
                            veeam_server_job
                                .backup_server_uid
                                .unwrap_or("N/A".to_string()),
                            veeam_server_job.location_uid.unwrap_or("N/A".to_string()),
                            veeam_server_job.site_uid.unwrap_or("N/A".to_string()),
                            veeam_server_job
                                .organization_uid
                                .unwrap_or("N/A".to_string()),
                            veeam_server_job.status.unwrap_or("N/A".to_string()),
                            veeam_server_job.r#type.unwrap_or("N/A".to_string()),
                            veeam_server_job.last_run.unwrap_or(Utc::now()),
                            veeam_server_job.last_end_time.unwrap_or(Utc::now()),
                            veeam_server_job.last_duration.unwrap_or(0),
                            veeam_server_job.processing_rate.unwrap_or(0.0),
                            veeam_server_job.avg_duration.unwrap_or(0),
                            veeam_server_job.transferred_data.unwrap_or(0),
                            veeam_server_job.bottleneck.unwrap_or("N/A".to_string()),
                            veeam_server_job.is_enabled.unwrap_or(false),
                            veeam_server_job.schedule_type.unwrap_or("N/A".to_string()),
                            veeam_server_job
                                .failure_message
                                .unwrap_or("N/A".to_string()),
                            veeam_server_job.target_type.unwrap_or("N/A".to_string()),
                            veeam_server_job.destination.unwrap_or("N/A".to_string()),
                            veeam_server_job.retention_limit.unwrap_or(0),
                            veeam_server_job
                                .retention_limit_type
                                .unwrap_or("N/A".to_string()),
                            veeam_server_job.is_gfs_option_enabled.unwrap_or(false),
                            veeam_server_job.company_name.unwrap_or("N/A".to_string()),
                            existing_veeam_server_job.name
                        )
                        .execute(&pool)
                        .await;

                        match update_result {
                            Ok(_) => {
                                updated += 1;
                            }
                            Err(error) => {
                                println!("Failed to update Veeam Server Job: {:?}", error);

                                skipped += 1;
                            }
                        }
                    }
                    None => {
                        if veeam_server_job.name.is_none() {
                            skipped += 1;

                            continue;
                        }

                        let insert_result = sqlx::query!(
                            r#"
                                INSERT INTO
                                    veeam_servers_jobs
                                (
                                    instance_uid,
                                    name,
                                    backup_server_uid,
                                    location_uid,
                                    site_uid,
                                    organization_uid,
                                    status,
                                    type,
                                    last_run,
                                    last_end_time,
                                    last_duration,
                                    processing_rate,
                                    avg_duration,
                                    transferred_data,
                                    bottleneck,
                                    is_enabled,
                                    schedule_type,
                                    failure_message,
                                    target_type,
                                    destination,
                                    retention_limit,
                                    retention_limit_type,
                                    is_gfs_option_enabled,
                                    company_name
                                )
                                VALUES
                                (
                                    $1,
                                    $2,
                                    $3,
                                    $4,
                                    $5,
                                    $6,
                                    $7,
                                    $8,
                                    $9,
                                    $10,
                                    $11,
                                    $12,
                                    $13,
                                    $14,
                                    $15,
                                    $16,
                                    $17,
                                    $18,
                                    $19,
                                    $20,
                                    $21,
                                    $22,
                                    $23,
                                    $24
                                );
                            "#,
                            veeam_server_job.instance_uid.unwrap_or("N/A".to_string()),
                            veeam_server_job.name.unwrap_or("N/A".to_string()),
                            veeam_server_job
                                .backup_server_uid
                                .unwrap_or("N/A".to_string()),
                            veeam_server_job.location_uid.unwrap_or("N/A".to_string()),
                            veeam_server_job.site_uid.unwrap_or("N/A".to_string()),
                            veeam_server_job
                                .organization_uid
                                .unwrap_or("N/A".to_string()),
                            veeam_server_job.status.unwrap_or("N/A".to_string()),
                            veeam_server_job.r#type.unwrap_or("N/A".to_string()),
                            veeam_server_job.last_run.unwrap_or(Utc::now()),
                            veeam_server_job.last_end_time.unwrap_or(Utc::now()),
                            veeam_server_job.last_duration.unwrap_or(0),
                            veeam_server_job.processing_rate.unwrap_or(0.0),
                            veeam_server_job.avg_duration.unwrap_or(0),
                            veeam_server_job.transferred_data.unwrap_or(0),
                            veeam_server_job.bottleneck.unwrap_or("N/A".to_string()),
                            veeam_server_job.is_enabled.unwrap_or(false),
                            veeam_server_job.schedule_type.unwrap_or("N/A".to_string()),
                            veeam_server_job
                                .failure_message
                                .unwrap_or("N/A".to_string()),
                            veeam_server_job.target_type.unwrap_or("N/A".to_string()),
                            veeam_server_job.destination.unwrap_or("N/A".to_string()),
                            veeam_server_job.retention_limit.unwrap_or(0),
                            veeam_server_job
                                .retention_limit_type
                                .unwrap_or("N/A".to_string()),
                            veeam_server_job.is_gfs_option_enabled.unwrap_or(false),
                            veeam_server_job.company_name.unwrap_or("N/A".to_string())
                        )
                        .execute(&pool)
                        .await;

                        match insert_result {
                            Ok(_) => {
                                inserted += 1;
                            }
                            Err(error) => {
                                println!("Failed to insert Veeam Server Job: {:?}", error);

                                skipped += 1;
                            }
                        }
                    }
                }
            }
        }
        Err(error) => {
            println!("Failed to get Veeam Server Jobs: {:?}", error);
        }
    }

    println!(
        "Finished syncing Veeam Server Jobs. Inserted: {}, Skipped: {}, Updated: {}",
        inserted, skipped, updated
    );

    Ok(())
}
