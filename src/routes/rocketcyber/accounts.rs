use axum::{extract::State, response::IntoResponse, Json};
use reqwest::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use crate::{
    functions::rocketcyber::accounts::accounts, models::rocketcyber::account::RocketAccount,
};

pub async fn index(State(pool): State<PgPool>) -> impl IntoResponse {
    let accounts: Vec<RocketAccount> =
        sqlx::query_as!(RocketAccount, "SELECT * FROM rocketcyber_accounts;")
            .fetch_all(&pool)
            .await
            .expect("Failed to get rocketcyber accounts from postgres.");

    Json(accounts)
}

pub async fn import(State(pool): State<PgPool>) -> impl IntoResponse {
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

    Json(json!({
        "status": StatusCode::OK.as_u16(),
        "inserted": inserted,
        "skipped": skipped
    }))
}
