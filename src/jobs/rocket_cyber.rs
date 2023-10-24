use std::time::Duration;

use axum::Json;
use dotenv::dotenv;
use reqwest::StatusCode;
use serde_json::json;
use sqlx::postgres::PgPoolOptions;

use crate::{
    functions::rocketcyber::{accounts::accounts, agents::agents, incidents::incidents},
    models::rocketcyber::{account::RocketAccount, agent::RocketAgent},
};

pub async fn sync_rocketcyber() {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost".into());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&database_url)
        .await
        .expect("Failed to connect to postgres.");

    println!("Syncing RocketCyber accounts.");

    let accounts = accounts()
        .await
        .expect("Failed to get rocketcyber accounts from rocketcyber api.");

    let mut inserted = 0;
    let mut skipped = 0;

    for account in accounts {
        let account_id = account.account_id.unwrap();

        let existing_account_result = sqlx::query_as!(
            RocketAccount,
            "SELECT * FROM rocketcyber_accounts WHERE account_id = $1;",
            account_id
        )
        .fetch_one(&pool)
        .await;

        match existing_account_result {
            Ok(_) => {
                skipped += 1;
            }
            Err(_) => {
                sqlx::query_as!(
                    RocketAccount,
                    "INSERT INTO rocketcyber_accounts (account_id, account_name, account_path, status) VALUES ($1,$2,$3,$4);",
                    account_id,
                    account.account_name.unwrap(),
                    account.account_path.unwrap(),
                    account.status.unwrap(),
                )
                .execute(&pool)
                .await
                .expect("Failed to insert rocketcyber account into postgres database.");

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

    println!("Syncing RocketCyber agents.");

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

    println!(
        "{:?}",
        Json(json!({
            "status": StatusCode::OK.as_u16(),
            "inserted": inserted,
            "skipped": skipped
        }))
    );

    println!("Syncing RocketCyber incidents.");

    let incidents = incidents()
        .await
        .expect("Failed to get rocketcyber incidents from rocketcyber api.");

    sqlx::query_as!(RocketIncident, "TRUNCATE rocketcyber_incidents;")
        .execute(&pool)
        .await
        .expect("Failed to remove old incidents from postgres database.");

    let mut inserted = 0;

    for incident in incidents {
        let incident_id = incident.id.unwrap();

        sqlx::query_as!(
            RocketIncident,
            "INSERT INTO rocketcyber_incidents VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10);",
            incident_id,
            incident.title.unwrap(),
            incident.description.unwrap(),
            incident.remediation.unwrap(),
            incident.resolved_at,
            incident.published_at,
            incident.created_at,
            incident.status.unwrap(),
            incident.account_id.unwrap(),
            incident.event_count.unwrap()
        )
        .execute(&pool)
        .await
        .expect("Failed to insert rocketcyber incident into postgres database.");

        inserted += 1;
    }

    println!(
        "{:?}",
        Json(json!({
            "status": StatusCode::OK.as_u16(),
            "inserted": inserted
        }))
    );

    println!("Finished RocketCyber sync.");
}
