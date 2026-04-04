use clap::Parser;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::EnvFilter;

use kzte_feeder::{load_config, run_once, run_service};
use kzte_feeder::metrics::FeederMetrics;

#[derive(Debug, Parser)]
#[command(name = "kzte-feeder", about = "KZTE reference oracle feeder")]
struct Args {
    #[arg(long, env = "FEEDER_CONFIG_PATH", default_value = "config/feeder.example.toml")]
    config: String,
    #[arg(long)]
    once: bool,
    #[arg(long)]
    json_logs: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    init_tracing(args.json_logs)?;

    let config = load_config(&args.config)?;
    if args.once {
        let metrics = Arc::new(FeederMetrics::default());
        let summary = run_once(&config, &metrics).await?;
        info!(?summary, "single feeder run completed");
        Ok(())
    } else {
        run_service(config).await
    }
}

fn init_tracing(json_logs: bool) -> anyhow::Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let builder = tracing_subscriber::fmt().with_env_filter(filter);

    if json_logs {
        builder
            .json()
            .try_init()
            .map_err(|error| anyhow::anyhow!("failed to initialize JSON tracing: {}", error))?;
    } else {
        builder
            .compact()
            .try_init()
            .map_err(|error| anyhow::anyhow!("failed to initialize tracing: {}", error))?;
    }

    Ok(())
}
