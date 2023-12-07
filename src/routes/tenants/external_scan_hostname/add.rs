use axum::{extract::State, response::IntoResponse, Json};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;

#[derive(Debug, Deserialize, Serialize)]
pub struct AddExternalScanHostNamePayload {
    pub tenant: String,
    pub host_name: String,
}

pub async fn add_tenant_external_scan_hostname(
    State(pool): State<PgPool>,
    Json(payload): Json<AddExternalScanHostNamePayload>,
) -> impl IntoResponse {
    let tenant_result = sqlx::query!(
        r#"
        SELECT id FROM tenants WHERE similarity(LOWER(tenants.tenant_name), LOWER($1)) >= 0.6
    "#,
        payload.tenant
    )
    .fetch_one(&pool)
    .await;

    match tenant_result {
        Ok(tenant) => {
            let created_tenant_result = sqlx::query!(
                r#"
                    INSERT INTO
                        tenants_external_scan_host_names (tenant_id, host_name) 
                    VALUES ($1, $2);
                "#,
                tenant.id,
                payload.host_name
            )
            .execute(&pool)
            .await;

            match created_tenant_result {
                Ok(_) => Json(json!({
                    "status": StatusCode::OK.as_u16(),
                    "message": "Created a new hostname."
                })),
                Err(err) => Json(json!({
                    "status": StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                    "message": "Failed to create new hostname.",
                    "error": err.to_string()
                })),
            }
        }
        Err(_) => Json(json!({
            "status": StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            "message": "Failed to find tenant in database."
        })),
    }
}
