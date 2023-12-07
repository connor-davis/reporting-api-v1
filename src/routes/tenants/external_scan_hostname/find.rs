use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use reqwest::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use crate::models::tenant::tenant_external_scan_hostname::TenantExternalScanHostname;

pub async fn index(
    Query(params): Query<Vec<(String, String)>>,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    let tenant = &params[0].1;

    let hostnames_result = sqlx::query_as!(
        TenantExternalScanHostname,
        r#"
            SELECT
                e.id as id,
                e.tenant_id as tenant_id,
                e.host_name as host_name
            FROM tenants_external_scan_host_names AS e
            LEFT JOIN
                tenants AS t ON e.tenant_id = t.id
            WHERE
                similarity(LOWER(t.tenant_name), LOWER($1)) >= 0.6
        "#,
        tenant
    )
    .fetch_all(&pool)
    .await;

    match hostnames_result {
        Ok(hostnames) => Json(json!({
            "status": StatusCode::OK.as_u16(),
            "results": hostnames
        })),
        Err(_) => Json(json!({
            "status": StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            "message": "Failed to find external scan hostnames from database."
        })),
    }
}
