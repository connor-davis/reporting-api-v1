use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use reqwest::StatusCode;
use serde_json::json;
use sqlx::PgPool;

pub async fn remove_external_scan_hostname(
    Query(params): Query<Vec<(String, String)>>,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    let host_name = &params[0].1;

    let delete_hostname_result = sqlx::query!(
        "DELETE FROM tenants_external_scan_host_names WHERE host_name = $1",
        host_name
    )
    .execute(&pool)
    .await;

    match delete_hostname_result {
        Ok(_) => Json(json!({
            "status": StatusCode::OK.as_u16(),
            "host_name": host_name,
            "message": "Deleted hostname."
        })),
        Err(err) => Json(json!({
            "status": StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            "host_name": host_name,
            "message": "Failed to delete hostname.",
            "error": err.to_string()
        })),
    }
}
