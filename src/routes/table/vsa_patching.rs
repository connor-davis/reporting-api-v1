use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use math::round::ceil;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "PascalCase", serialize = "PascalCase"))]
pub struct VsaPatch {
    pub id: String,
    pub organization_name: Option<String>,
    pub computer_name: Option<String>,
    pub total_patches: Option<f64>,
    pub installed_patches: Option<f64>,
    pub last_patch: Option<DateTime<Utc>>,
    pub next_patch: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VsaPatchResult {
    pub id: String,
    pub computer_name: Option<String>,
    pub patch_status: Option<String>,
    pub total_patches: Option<f64>,
    pub installed_patches: Option<f64>,
    pub last_patch: Option<DateTime<Utc>>,
    pub next_patch: Option<DateTime<Utc>>,
}

pub async fn index(
    Query(params): Query<Vec<(String, String)>>,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    let tenant = &params[0].1;
    let offset = (&params[1].1.parse::<i64>().unwrap_or(1) - 1) * 10;

    let vsa_patches_result =
        sqlx::query_as!(
            VsaPatch,
            "SELECT id, organization_name, computer_name, total_patches, installed_patches, last_patch, next_patch FROM vsa_agents WHERE LOWER(TRIM(TRAILING ' (Pty) Ltd' FROM organization_name)) = LOWER($1) ORDER BY computer_name OFFSET $2 LIMIT 10;",
            tenant,
            offset
        )
            .fetch_all(&pool)
            .await
            .expect("Failed to get vsa organization names from postgres.");

    let mut total_pages_result = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM vsa_agents WHERE LOWER(TRIM(TRAILING ' (Pty) Ltd' FROM organization_name)) = LOWER($1);",
        tenant
    )
        .fetch_one(&pool)
        .await
        .expect("Failed to get total results from postgres.");

    if total_pages_result.unwrap_or(10) < 10 {
        total_pages_result = Some(10);
    }

    let total_pages = ceil(total_pages_result.unwrap_or(10) as f64 / 10 as f64, 0);

    let mut patch_results_array: Vec<VsaPatchResult> = Vec::new();

    for result in vsa_patches_result {
        let mut patch_status = "OUTDATED".to_string();

        if result.total_patches == result.installed_patches {
            patch_status = "UP TO DATE".to_string();
        }

        patch_results_array.push(VsaPatchResult {
            id: result.id,
            computer_name: result.computer_name,
            patch_status: Some(patch_status),
            total_patches: result.total_patches,
            installed_patches: result.installed_patches,
            last_patch: result.last_patch,
            next_patch: result.next_patch,
        })
    }

    Json(json!({
        "status": StatusCode::OK.as_u16(),
        "tenant": tenant,
        "results": patch_results_array,
        "total_pages": total_pages
    }))
}
