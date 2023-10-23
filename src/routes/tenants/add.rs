use axum::{
    extract::{Json, State},
    response::IntoResponse,
};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;

#[derive(Debug, Deserialize, Serialize)]
pub struct Tenant {
    tenant_name: String,
    vsa_name: Option<String>,
    cyber_cns_name: Option<String>,
    rocket_cyber_name: Option<String>,
}

pub async fn add_tenant(
    State(pool): State<PgPool>,
    Json(payload): Json<Tenant>,
) -> impl IntoResponse {
    let created_tenant_result = sqlx::query!("INSERT INTO tenants (tenant_name, vsa_name, cyber_cns_name, rocket_cyber_name) VALUES ($1, $2, $3, $4);", payload.tenant_name, payload.vsa_name, payload.cyber_cns_name, payload.rocket_cyber_name).execute(&pool).await;

    match created_tenant_result {
        Ok(_) => Json(json!({
            "status": StatusCode::OK.as_u16(),
            "message": "Created a new tenant."
        })),
        Err(err) => Json(json!({
            "status": StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            "message": "Failed to create new tenant.",
            "error": err.to_string()
        })),
    }
}
