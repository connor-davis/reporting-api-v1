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
    tenant_id: i32,
    tenant_name: String,
    vsa_name: Option<String>,
    cyber_cns_name: Option<String>,
    rocket_cyber_name: Option<String>,
}

pub async fn update_tenant(
    State(pool): State<PgPool>,
    Json(payload): Json<Tenant>,
) -> impl IntoResponse {
    let updated_tenant_result =
        sqlx::query!(
            "UPDATE tenants SET vsa_name = $1, cyber_cns_name = $2, rocket_cyber_name = $3, tenant_name = $4 WHERE id = $5;",
            payload.vsa_name,
            payload.cyber_cns_name,
            payload.rocket_cyber_name,
            payload.tenant_name,
            payload.tenant_id
        )
        .execute(&pool)
        .await;

    match updated_tenant_result {
        Ok(result) => Json(json!({
            "status": StatusCode::OK.as_u16(),
            "message": "Updated tenant.",
            "tenant": Tenant {
                tenant_id: payload.tenant_id,
                tenant_name: payload.tenant_name,
                vsa_name: payload.vsa_name,
                cyber_cns_name: payload.cyber_cns_name,
                rocket_cyber_name: payload.rocket_cyber_name
            },
            "result": result.rows_affected(),
        })),
        Err(err) => Json(json!({
            "status": StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            "message": "Failed to update tenant.",
            "error": err.to_string()
        })),
    }
}
