use std::net::SocketAddr;

use axum::{extract::DefaultBodyLimit, Router};
use dotenv::dotenv;
use tokio_cron_scheduler::{Job, JobScheduler};
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    documentation::openapi::ApiDoc,
    functions::cybercns::external_scan::start_external_scan,
    jobs::{
        cyber_cns::sync_cybercns, rocket_cyber::sync_rocketcyber, spanning::sync_spanning,
        veeam::sync_veeam, vsa::sync_vsa,
    },
    routes::router::router,
};

mod documentation;
mod functions;
mod jobs;
mod models;
mod routes;

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "api=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let router = router().await;

    let app = Router::new()
        .nest_service("/", router)
        .merge(SwaggerUi::new("/docs").url("/docs/openapi.json", ApiDoc::openapi()))
        .layer(DefaultBodyLimit::max(100_000_000))
        .layer(CorsLayer::permissive())
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));

    let address = SocketAddr::from(([0, 0, 0, 0], 3000));

    println!("Server listening on {}", address);
    println!("Spawning cronjobs.");

    tokio::spawn(start_external_scan());

    tokio::spawn(cronjobs());
    tokio::spawn(sync_vsa());
    tokio::spawn(sync_cybercns());
    tokio::spawn(sync_rocketcyber());
    tokio::spawn(sync_spanning());
    tokio::spawn(sync_veeam());

    axum::Server::bind(&address)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn cronjobs() {
    let scheduler = JobScheduler::new().await;

    match scheduler {
        Ok(mut scheduler) => {
            let fifth_minute_job = Job::new(
                "0     1/5     *      *              *       *            ",
                |_uuid, _l| {
                    println!("Running 5th minute cronjobs.");
                },
            );

            match fifth_minute_job {
                Ok(fifth_minute_job) => {
                    let _ = scheduler.add(fifth_minute_job).await;
                }
                Err(err) => println!("Fifth Minute Job Failed: {:?}", err),
            }

            let hourly_job = Job::new(
                "0     0     *      *              *       *            ",
                |_uuid, _l| {
                    println!("Running hourly cronjobs.");

                    tokio::spawn(sync_vsa());
                    tokio::spawn(sync_cybercns());
                    tokio::spawn(sync_rocketcyber());
                    tokio::spawn(sync_spanning());
                    tokio::spawn(sync_veeam());
                },
            );

            match hourly_job {
                Ok(hourly_job) => {
                    let _ = scheduler.add(hourly_job).await;
                }
                Err(err) => println!("Hourly Job Failed: {:?}", err),
            }

            let daily_job = Job::new("0 0 4 * * * *", |_uuid, _l| {
                println!("Running daily cronjobs.");

                tokio::spawn(start_external_scan());
            });

            match daily_job {
                Ok(daily_job) => {
                    let _ = scheduler.add(daily_job).await;
                }
                Err(err) => println!("Daily Job Failed: {:?}", err),
            }

            // Add code to be run during/after shutdown
            scheduler.set_shutdown_handler(Box::new(|| {
                Box::pin(async move {
                    println!("Shut down done");
                })
            }));

            match scheduler.start().await {
                Ok(_) => println!("Scheduler started."),
                Err(err) => println!("Scheduler Failed: {:?}", err),
            }
        }
        Err(err) => println!("Job Scheduler Failed: {:?}", err),
    }
}
