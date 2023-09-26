use axum::{extract::State, response::IntoResponse, Json};
use reqwest::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use crate::{
    functions::vsa::security_products::{security_products, VsaSecurityProduct},
    models::vsa::agent::VsaAgent,
};

pub async fn index() -> impl IntoResponse {
    let products = security_products().await;

    Json(products)
}

pub async fn import(State(pool): State<PgPool>) -> impl IntoResponse {
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

    Json(json!({
        "status": StatusCode::OK.as_u16(),
        "updated": updated,
        "skipped": skipped
    }))
}
