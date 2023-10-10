use std::collections::HashMap;

use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Datelike, Utc};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "PascalCase", serialize = "PascalCase"))]
pub struct VsaPatch {
    pub id: String,
    pub organization_name: Option<String>,
    pub total_patches: Option<f64>,
    pub installed_patches: Option<f64>,
    pub last_patch: Option<DateTime<Utc>>,
    pub next_patch: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "PascalCase", serialize = "PascalCase"))]
pub struct VsaPatchResult {
    pub total: Option<f64>,
    pub outstanding: Option<f64>,
    pub date: Option<String>,
}

pub async fn index(
    Query(params): Query<Vec<(String, String)>>,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    let tenant = &params[0].1;

    let current_date = Utc::now();

    let mut total_patches_current_month = 0 as f64;
    let mut total_outstanding_patches_current_month = 0 as f64;

    let vsa_patches_result =
        sqlx::query_as!(
            VsaPatch,
            "SELECT id, organization_name, total_patches, installed_patches, last_patch, next_patch FROM vsa_agents WHERE LOWER(TRIM(TRAILING ' (Pty) Ltd' FROM organization_name)) = LOWER($1) ORDER BY computer_name;",
            tenant
        )
            .fetch_all(&pool)
            .await
            .expect("Failed to get vsa organization names from postgres.");

    let mut patch_results: HashMap<String, VsaPatchResult> = HashMap::new();

    for result in &vsa_patches_result {
        if !result.last_patch.is_none() {
            let last_patch = result.last_patch.unwrap();

            if last_patch.month() == current_date.month() {
                println!("There was a patch this month: {:?}", result);

                total_patches_current_month += result.total_patches.unwrap();
                total_outstanding_patches_current_month +=
                    result.total_patches.unwrap() - result.installed_patches.unwrap();
            }

            if !patch_results.contains_key(
                format!(
                    "{}-{}-{}",
                    last_patch.year(),
                    last_patch.month(),
                    last_patch.day()
                )
                .as_str(),
            ) {
                patch_results.insert(
                    format!(
                        "{}-{}-{}",
                        last_patch.year(),
                        last_patch.month(),
                        last_patch.day()
                    ),
                    VsaPatchResult {
                        total: Some(result.total_patches.unwrap()),
                        outstanding: Some(
                            result.total_patches.unwrap() - result.installed_patches.unwrap(),
                        ),
                        date: Some(format!(
                            "{}-{}-{}",
                            last_patch.year(),
                            last_patch.month(),
                            last_patch.day()
                        )),
                    },
                );
            } else {
                let found_result = patch_results.get(
                    format!(
                        "{}-{}-{}",
                        last_patch.year(),
                        last_patch.month(),
                        last_patch.day()
                    )
                    .as_str(),
                );

                if !found_result.is_none() {
                    let unwrapped_result = found_result.unwrap();
                    let mut total = unwrapped_result.total.unwrap();
                    let mut outstanding = unwrapped_result.outstanding.unwrap();

                    total += result.total_patches.unwrap();
                    outstanding +=
                        result.total_patches.unwrap() - result.installed_patches.unwrap();

                    patch_results.insert(
                        format!(
                            "{}-{}-{}",
                            last_patch.year(),
                            last_patch.month(),
                            last_patch.day()
                        ),
                        VsaPatchResult {
                            total: Some(total),
                            outstanding: Some(outstanding),
                            date: Some(format!(
                                "{}-{}-{}",
                                last_patch.year(),
                                last_patch.month(),
                                last_patch.day()
                            )),
                        },
                    );
                }
            }
        }
    }

    let mut patch_results_array: Vec<VsaPatchResult> = Vec::new();

    for result in patch_results {
        patch_results_array.push(result.1)
    }

    Json(json!({
        "status": StatusCode::OK.as_u16(),
        "tenant": tenant,
        "total_patches": total_patches_current_month,
        "outstanding_patches": total_outstanding_patches_current_month,
        "results": patch_results_array
    }))
}
