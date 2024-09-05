use crate::metrics;
use axum::{routing::get, Router};
use prometheus::Registry;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

pub async fn serve_metrics(registry: Arc<Registry>) {
    let router = Router::new().route(
        "/metrics",
        get(move || {
            let registry = Arc::clone(&registry);
            async move { metrics::gather_metrics_output(&registry) }
        }),
    );

    let addr = SocketAddr::from(([127, 0, 0, 1], 8050));
    let tcp = TcpListener::bind(&addr).await.unwrap();

    axum::serve(tcp, router).await.unwrap();
}
