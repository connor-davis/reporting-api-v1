use std::time::Duration;

use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use dotenv::dotenv;
use serde_json::json;
use sqlx::{postgres::PgPoolOptions, PgPool};

use super::{cybercns, rocketcyber};

pub async fn router() -> Router {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost".into());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&database_url)
        .await
        .expect("Failed to connect to postgres.");

    Router::new()
        .route("/", get(index))
        .nest(
            "/cyber-cns",
            Router::new()
                .route(
                    "/agents",
                    get(cybercns::agents::index).post(cybercns::agents::import),
                )
                .route(
                    "/assets",
                    get(cybercns::assets::index).post(cybercns::assets::import),
                ),
        )
        .nest(
            "/rocket-cyber",
            Router::new()
                .route(
                    "/agents",
                    get(rocketcyber::agents::index).post(rocketcyber::agents::import),
                )
                .route(
                    "/incidents",
                    get(rocketcyber::incidents::index).post(rocketcyber::incidents::import),
                ),
        )
        .fallback(fallback)
        .with_state(pool)
}

async fn index(State(pool): State<PgPool>) -> impl IntoResponse {
    let result = sqlx::query_scalar!("select 'Welcome to Reporting API'")
        .fetch_one(&pool)
        .await
        .expect("Failed to query postgres pool.")
        .unwrap();

    Json(json!({
        "status": StatusCode::OK.as_u16(),
        "message": result
    }))
}

async fn fallback() -> impl IntoResponse {
    Json(json!({
        "status": StatusCode::NOT_FOUND.as_u16(),
        "message": "Route not found. Please contact the developer."
    }))
}
