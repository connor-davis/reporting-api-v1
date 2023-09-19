use chrono::{DateTime, Days, Local};
use dotenv::dotenv;
use reqwest::{Client, Error};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "camelCase", serialize = "camelCase"))]
pub struct RocketIncident {
    pub id: Option<i64>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub remediation: Option<String>,
    pub resolved_at: Option<DateTime<Local>>,
    pub published_at: Option<DateTime<Local>>,
    pub created_at: Option<DateTime<Local>>,
    pub status: Option<String>,
    pub account_id: Option<i64>,
    pub event_count: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "camelCase", serialize = "camelCase"))]
pub struct RocketIncidentsResponse {
    pub data: Option<Vec<RocketIncident>>,
}

/*
Fetch the last 30 days incidents from RocketCyber using the reqwest HTTP Client crate.
*/
pub async fn incidents() -> Result<Vec<RocketIncident>, Error> {
    dotenv().ok();

    let client = Client::builder().http1_title_case_headers().build()?;

    let access_token = std::env::var("ROCKET_CYBER_API_KEY").ok().unwrap();

    let local: DateTime<Local> = Local::now();
    let start_date = local
        .checked_sub_days(Days::new(30))
        .unwrap()
        .format("%Y-%m-%d")
        .to_string();
    let end_date = local.format("%Y-%m-%d").to_string();

    let response = client
        .get(format!(
            "https://api-eu.rocketcyber.com/v3/incidents?createdAt={}|{}",
            start_date, end_date
        ))
        .bearer_auth(access_token)
        .send()
        .await?;

    let body = response
        .json::<RocketIncidentsResponse>()
        .await
        .expect("Failed to retrieve Rocket Cyber agents.");

    Ok(body.data.unwrap())
}
