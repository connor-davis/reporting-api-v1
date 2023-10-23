use axum::{extract::Query, response::IntoResponse, Json};
use reqwest::StatusCode;
use serde_json::json;
use tokio::fs::remove_file;

pub async fn index(Query(params): Query<Vec<(String, String)>>) -> impl IntoResponse {
    let file_name = &params[0].1;

    let delete_file_result = remove_file(format!("uploads/{}", file_name)).await;

    match delete_file_result {
        Ok(_) => {
            Json(json!({"status":StatusCode::OK.as_u16(),"message": "Successfully deleted file."}))
        }
        Err(err) => Json(json!({
            "status": StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            "message": "Failed to delete file.",
            "error": err.to_string()
        })),
    }
}
