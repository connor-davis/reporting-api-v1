use axum::{extract::Query, response::IntoResponse, Json};
use reqwest::StatusCode;
use serde_json::json;
use tokio::fs::read_dir;

pub async fn index(Query(params): Query<Vec<(String, String)>>) -> impl IntoResponse {
    let file_name = &params[0].1;

    // find the report in the reports directory between the start and end date
    let mut dir = read_dir("scans").await.unwrap();
    let mut file_path = String::new();

    while let Ok(Some(entry)) = dir.next_entry().await {
        if entry.file_name().to_str() == Some(file_name) {
            file_path = entry.path().to_str().unwrap().to_string();
        }
    }

    // read the file
    let file = tokio::fs::read_to_string(file_path).await.unwrap();

    // conver the file to json
    let file: serde_json::Value = serde_json::from_str(&file).unwrap();

    // return the file
    Json(json!({
        "status": StatusCode::OK.as_u16(),
        "data": file
    }))
}
