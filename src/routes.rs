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
    let port = 8050;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let tcp = TcpListener::bind(&addr).await.unwrap();

    axum::serve(tcp, router).await.unwrap();

    println!(
        "Serving metrics on IP: {}:{}/metrics",
        addr.ip(),
        addr.port()
    );
}
