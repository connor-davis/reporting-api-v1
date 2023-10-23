use axum::{response::IntoResponse, Json};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;

pub async fn index() -> impl IntoResponse {
    Json(json!({
        "status": StatusCode::OK.as_u16()
    }))
}

pub async fn new_scan() -> impl IntoResponse {
    Json(json!({
        "status": StatusCode::OK.as_u16()
    }))
}

pub async fn show_scan() -> impl IntoResponse {
    Json(json!({
        "status": StatusCode::OK.as_u16()
    }))
}

pub async fn hide_scan() -> impl IntoResponse {
    Json(json!({
        "status": StatusCode::OK.as_u16()
    }))
}

pub async fn delete_scan() -> impl IntoResponse {
    Json(json!({
        "status": StatusCode::OK.as_u16()
    }))
}
