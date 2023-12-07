use axum::{extract::Query, response::IntoResponse, Json};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::fs::read_dir;

pub async fn index(Query(params): Query<Vec<(String, String)>>) -> impl IntoResponse {
    let tenant = &params[0].1;

    // List all of the .json files from the reports directory
    let mut dir = read_dir("scans").await.unwrap();

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Scan {
        pub tenant: String,
        pub date: String,
        pub time: String,
        pub host_name: String,
        pub file_name: String,
    }

    let mut found_scans: Vec<Scan> = Vec::new();

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
            let host_name = file_name_split[2].to_string();
            let date = file_name_split[3..6].join("-").to_string();
            let time = file_name_split[6..9].join(":").to_string();

            found_scans.push(Scan {
                tenant,
                date,
                time,
                host_name,
                file_name,
            });
        }
    }

    Json(json!({
        "status": StatusCode::OK.as_u16(),
        "scans": found_scans
    }))
}
