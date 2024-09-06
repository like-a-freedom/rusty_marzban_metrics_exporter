mod api;
mod metrics;
mod routes;
use api::APIError;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;

#[tokio::main]
async fn main() -> Result<(), APIError> {
    dotenv::dotenv().ok();

    // Initialize API client and metrics registry
    println!("Starting fetch Marzban URL: {}", env::var("URL").unwrap());
    let api_client = Arc::new(api::MarzbanAPI::new().await?);
    let registry = Arc::new(metrics::setup_metrics_registry());

    // Register and create all metrics only once
    let all_metrics = Arc::new(metrics::create_metrics(&registry));

    // Get update interval from environment variable or use a default value
    let update_interval_secs: u64 = env::var("UPDATE_INTERVAL")
        .unwrap_or_else(|_| "60".to_string()) // default to 60 seconds if not set
        .parse()
        .expect("UPDATE_INTERVAL must be a valid u64");

    // Start the metrics update loop
    let api_client_clone = Arc::clone(&api_client);
    let all_metrics_clone = Arc::clone(&all_metrics);

    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(update_interval_secs));
        loop {
            interval.tick().await;
            if let Err(e) = update_metrics_periodically(&api_client_clone, &all_metrics_clone).await
            {
                eprintln!("Failed to update metrics: {}", e);
            }
        }
    });

    // Start HTTP server to serve metrics
    routes::serve_metrics(Arc::clone(&registry)).await;

    Ok(())
}

async fn update_metrics_periodically(
    api_client: &Arc<api::MarzbanAPI>,
    metrics: &Arc<metrics::Metrics>,
) -> Result<(), APIError> {
    // Fetch data
    let nodes = api_client.fetch_nodes_data().await?;
    let node_usages = api_client.fetch_nodes_usage_data().await?;
    let system_data = api_client.fetch_system_data().await?;
    let core_data = api_client.fetch_core_data().await?;
    let users = api_client.fetch_users_data().await?;

    // Update metrics
    metrics::update_metrics(
        metrics,
        &nodes,
        &node_usages,
        &system_data,
        &core_data,
        &users,
    );

    Ok(())
}
