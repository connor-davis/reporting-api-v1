use std::time::Duration;

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use dotenv::dotenv;
use serde_json::json;
use sqlx::{postgres::PgPoolOptions, PgPool};

use super::{
    cybercns, logos, reports, rocketcyber, scans, spanning, statistics, table, tenants, vsa,
};

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
            "/tenants",
            Router::new()
                .route(
                    "/",
                    get(tenants::find::all)
                        .post(tenants::add::add_tenant)
                        .put(tenants::update::update_tenant)
                        .delete(tenants::remove::delete_tenant),
                )
                .route("/smart", get(tenants::find::smart_find))
                .route(
                    "/external-scan-hostname",
                    get(tenants::external_scan_hostname::find::index)
                        .post(
                            tenants::external_scan_hostname::add::add_tenant_external_scan_hostname,
                        )
                        .delete(
                            tenants::external_scan_hostname::remove::remove_external_scan_hostname,
                        ),
                ),
        )
        .nest(
            "/logos",
            Router::new()
                .route("/", get(logos::find::index).delete(logos::remove::index))
                .route("/view", get(logos::find::get_file))
                .route("/upload", post(logos::upload::index)),
        )
        .nest(
            "/reports",
            Router::new()
                .route(
                    "/",
                    get(reports::find::index).delete(reports::remove::delete_report),
                )
                .route("/view", get(reports::view::index))
                .route("/generate", get(reports::generate::index)),
        )
        .nest(
            "/scans",
            Router::new()
                .route("/", get(scans::find::index))
                .route("/view", get(scans::view::index)),
        )
        .nest(
            "/statistics",
            Router::new()
                .route("/vsa", get(statistics::vsa::index))
                .route("/vsa-patching", get(statistics::vsa_patching::index))
                .route("/rocket-cyber", get(statistics::rocket_cyber::index)),
        )
        .nest(
            "/table",
            Router::new()
                .route("/vsa", get(table::vsa::index))
                .route("/vsa/patching", get(table::vsa_patching::index))
                .route("/cns", get(table::cns_assets::index))
                .route(
                    "/cns-vulnerabilities",
                    get(table::cns_vulnerabilities::index),
                )
                .route("/rocket-cyber", get(table::rocket_cyber::index))
                .route(
                    "/rocket-cyber-agents",
                    get(table::rocket_cyber_agents::index),
                )
                .route("/spanning-backups", get(table::spanning_backups::index)),
        )
        .nest(
            "/vsa",
            Router::new()
                .route("/agents", get(vsa::agents::index).post(vsa::agents::import))
                .route(
                    "/security-products",
                    get(vsa::security_products::index).post(vsa::security_products::import),
                )
                .route(
                    "/devices",
                    get(vsa::devices::index).post(vsa::devices::import),
                )
                .route("/disks", get(vsa::disks::index).post(vsa::disks::import))
                .route("/groups", get(vsa::groups::index).post(vsa::groups::import))
                .route(
                    "/patches",
                    get(vsa::patches::index).post(vsa::patches::import),
                ),
        )
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
                )
                .route(
                    "/vulnerabilities",
                    get(cybercns::vulnerabilities::index).post(cybercns::vulnerabilities::import),
                ),
        )
        .nest(
            "/rocket-cyber",
            Router::new()
                .route(
                    "/accounts",
                    get(rocketcyber::accounts::index).post(rocketcyber::accounts::import),
                )
                .route(
                    "/agents",
                    get(rocketcyber::agents::index).post(rocketcyber::agents::import),
                )
                .route(
                    "/incidents",
                    get(rocketcyber::incidents::index).post(rocketcyber::incidents::import),
                ),
        )
        .nest(
            "/spanning",
            Router::new().route(
                "/backups",
                get(spanning::spanning_backups::index).post(spanning::spanning_backups::import),
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
