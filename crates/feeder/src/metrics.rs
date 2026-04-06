use anyhow::{Context, Result};
use axum::{extract::State, response::IntoResponse, routing::get, Router};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::task::JoinHandle;

#[derive(Debug, Default)]
pub struct FeederMetrics {
    cycles_total: AtomicU64,
    failures_total: AtomicU64,
    submissions_total: AtomicU64,
    skipped_total: AtomicU64,
    last_sequence: AtomicU64,
    last_kzte_usd_price: AtomicI64,
}

impl FeederMetrics {
    pub fn record_cycle(&self) {
        self.cycles_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_failure(&self) {
        self.failures_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_submission(&self, sequence: u64, price: i64) {
        self.submissions_total.fetch_add(1, Ordering::Relaxed);
        self.last_sequence.store(sequence, Ordering::Relaxed);
        self.last_kzte_usd_price.store(price, Ordering::Relaxed);
    }

    pub fn record_skip(&self) {
        self.skipped_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn render(&self) -> String {
        format!(
            concat!(
                "# TYPE kzte_feeder_cycles_total counter\n",
                "kzte_feeder_cycles_total {}\n",
                "# TYPE kzte_feeder_failures_total counter\n",
                "kzte_feeder_failures_total {}\n",
                "# TYPE kzte_feeder_submissions_total counter\n",
                "kzte_feeder_submissions_total {}\n",
                "# TYPE kzte_feeder_skipped_total counter\n",
                "kzte_feeder_skipped_total {}\n",
                "# TYPE kzte_feeder_last_sequence gauge\n",
                "kzte_feeder_last_sequence {}\n",
                "# TYPE kzte_feeder_last_kzte_usd_price gauge\n",
                "kzte_feeder_last_kzte_usd_price {}\n"
            ),
            self.cycles_total.load(Ordering::Relaxed),
            self.failures_total.load(Ordering::Relaxed),
            self.submissions_total.load(Ordering::Relaxed),
            self.skipped_total.load(Ordering::Relaxed),
            self.last_sequence.load(Ordering::Relaxed),
            self.last_kzte_usd_price.load(Ordering::Relaxed),
        )
    }
}

pub async fn spawn_metrics_server(
    listen_addr: &str,
    metrics: Arc<FeederMetrics>,
) -> Result<JoinHandle<()>> {
    let app = Router::new()
        .route("/metrics", get(metrics_handler))
        .with_state(metrics);
    let socket: SocketAddr = listen_addr
        .parse()
        .with_context(|| format!("invalid metrics listen address {}", listen_addr))?;
    let listener = tokio::net::TcpListener::bind(socket)
        .await
        .with_context(|| format!("failed to bind metrics listener at {}", listen_addr))?;

    Ok(tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    }))
}

async fn metrics_handler(State(metrics): State<Arc<FeederMetrics>>) -> impl IntoResponse {
    metrics.render()
}
