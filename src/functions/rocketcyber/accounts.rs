use dotenv::dotenv;
use reqwest::{Client, Error};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub struct RocketAccount {
    pub account_id: Option<i64>,
    pub account_name: Option<String>,
    pub account_path: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all(deserialize = "camelCase", serialize = "camelCase"))]
pub struct RocketAccountResponse {
    pub customers: Option<Vec<RocketAccount>>,
}

/*
Fetch the accounts from RocketCyber using the reqwest HTTP Client crate.
*/
pub async fn accounts() -> Result<Vec<RocketAccount>, Error> {
    dotenv().ok();

    let client = Client::builder().http1_title_case_headers().build()?;

    let access_token = std::env::var("ROCKET_CYBER_API_KEY").ok().unwrap();

    let response = client
        .get("https://api-eu.rocketcyber.com/v3/account?details=true")
        .bearer_auth(access_token)
        .send()
        .await?;

    let body = response
        .json::<RocketAccountResponse>()
        .await
        .expect("Failed to retrieve Rocket Cyber accounts.");

    Ok(body.customers.unwrap())
}
