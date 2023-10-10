use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use reqwest::StatusCode;
use serde_json::json;
use sqlx::PgPool;

pub async fn index(
    Query(params): Query<Vec<(String, String)>>,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    let tenant = &params[0].1;

    let vsa_organizations_result =
        sqlx::query!(
            "SELECT organization_name, anti_virus, os_name FROM vsa_agents WHERE LOWER(TRIM(TRAILING ' (Pty) Ltd' FROM organization_name)) = LOWER($1);",
            tenant
        )
            .fetch_all(&pool)
            .await
            .expect("Failed to get vsa organization names from postgres.");

    let mut agents_count = 0;
    let mut agents_with_av_count = 0;
    let mut win11_agents_count = 0;
    let mut win10_agents_count = 0;
    let mut win7_agents_count = 0;

    for record in vsa_organizations_result {
        let anti_virus = record.anti_virus.unwrap_or(false);
        let os_name = record.os_name.unwrap_or("Unknown".to_string());

        agents_count += 1;

        if anti_virus {
            agents_with_av_count += 1;
        }

        // Write a check to see if os_name is windows 11/10/7 else nothing
        if os_name.contains("7601") {
            win7_agents_count += 1;
        }

        // Check if os_name contains any of the windows 10 build numbers
        if os_name.contains("10240")
            || os_name.contains("10586")
            || os_name.contains("14393")
            || os_name.contains("15063")
            || os_name.contains("16299")
            || os_name.contains("17134")
            || os_name.contains("17763")
            || os_name.contains("18362")
            || os_name.contains("18363")
            || os_name.contains("19041")
            || os_name.contains("19042")
            || os_name.contains("19043")
            || os_name.contains("19044")
            || os_name.contains("19045")
        {
            win10_agents_count += 1;
        }

        // Check if os_name contains any of the windows 11 build numbers
        if os_name.contains("22000") || os_name.contains("22621") {
            win11_agents_count += 1;
        }
    }

    Json(json!({
        "status": StatusCode::OK.as_u16(),
        "tenant": tenant,
        "agents": agents_count,
        "agents_with_anti_virus": agents_with_av_count,
        "win11_agents": win11_agents_count,
        "win10_agents": win10_agents_count,
        "win7_agents": win7_agents_count
    }))
}
