use axum::{extract::Query, response::IntoResponse, Json};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::fs::read_dir;

pub async fn index(Query(params): Query<Vec<(String, String)>>) -> impl IntoResponse {
    let tenant = &params[0].1;

    // List all of the .json files from the reports directory
    let mut dir = read_dir("reports").await.unwrap();

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Report {
        pub tenant: String,
        pub date: String,
        pub time: String,
    }

    let mut found_reports: Vec<Report> = Vec::new();

    while let Ok(Some(entry)) = dir.next_entry().await {
        if entry
            .file_name()
            .to_str()
            .unwrap()
            .to_string()
            .contains(tenant)
        {
            let mut file_name = entry.file_name().to_str().unwrap().to_string();
            file_name = file_name.replace(".json", "");

            let file_name_split: Vec<&str> = file_name.split("-").collect();
            let tenant = file_name_split[1].to_string();
            let date = file_name_split[2..5].join("-").to_string();
            let time = file_name_split[5..8].join(":").to_string();

            found_reports.push(Report { tenant, date, time });
        }
    }

    Json(json!({
        "status": StatusCode::OK.as_u16(),
        "reports": found_reports
    }))
}
