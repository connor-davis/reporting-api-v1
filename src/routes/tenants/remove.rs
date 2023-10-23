use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use reqwest::StatusCode;
use serde_json::json;
use sqlx::PgPool;

pub async fn delete_tenant(
    Query(params): Query<Vec<(String, String)>>,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    let tenant_id = &params[0].1.parse::<i32>().unwrap();

    let delete_tenant_result = sqlx::query!("DELETE FROM tenants WHERE id = $1", tenant_id)
        .execute(&pool)
        .await;

    match delete_tenant_result {
        Ok(_) => Json(json!({
            "status": StatusCode::OK.as_u16(),
            "tenant_id": tenant_id,
            "message": "Deleted tenant."
        })),
        Err(err) => Json(json!({
            "status": StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            "tenant_id": tenant_id,
            "message": "Failed to delete tenant.",
            "error": err.to_string()
        })),
    }
}
