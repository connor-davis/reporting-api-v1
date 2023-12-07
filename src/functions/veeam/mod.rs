/*
dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost".into());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&database_url)
        .await
        .expect("Failed to connect to postgres.");

    let tenants = sqlx::query_as!(
        Tenant,
        "SELECT id, tenant_name, veeam_key FROM tenants WHERE veeam_key IS NOT NULL;"
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to get tenants from postgres.");

    println!("{:?}", tenants);

    Ok(())
*/
pub mod veeam_agents;
pub mod veeam_agents_jobs;
pub mod veeam_servers;
pub mod veeam_servers_jobs;
