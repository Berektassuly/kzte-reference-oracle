use anyhow::{Context, Result};
use futures::future::join_all;
use kzte_common::FeederConfig;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{interval, sleep};
use tracing::{error, info, warn};

use crate::adapters::{NbkOfficialPageAdapter, NbkOpenDataAdapter, SourceAdapter};
use crate::aggregator::{aggregate_official_quotes, build_reference_submissions, should_skip_submission};
use crate::metrics::{spawn_metrics_server, FeederMetrics};
use crate::submitter::{parse_pubkey, OracleRpcClient};

#[derive(Debug, Clone)]
pub struct RunSummary {
    pub submitted: usize,
    pub skipped: usize,
    pub signatures: Vec<String>,
    pub used_official_sources: usize,
    pub carry_forward: bool,
}

pub async fn run_service(config: FeederConfig) -> Result<()> {
    let metrics = Arc::new(FeederMetrics::default());
    let _server = spawn_metrics_server(&config.metrics.listen_addr, metrics.clone()).await?;
    let mut ticker = interval(Duration::from_secs(config.policy.interval_seconds.max(1)));

    loop {
        ticker.tick().await;
        if let Err(error) = run_once(&config, &metrics).await {
            metrics.record_failure();
            error!(error = ?error, "feeder cycle failed");
        }
    }
}

pub async fn run_once(config: &FeederConfig, metrics: &Arc<FeederMetrics>) -> Result<RunSummary> {
    metrics.record_cycle();

    let rpc = OracleRpcClient::new(config)?;
    let (oracle_state, feed_set) = rpc.load_state(config).await?;
    let config_pubkey = parse_pubkey(&config.oracle.config_pubkey, "oracle config pubkey")?;
    let publisher_set_pubkey = parse_pubkey(&config.oracle.publisher_set_pubkey, "publisher set pubkey")?;

    let official_quotes = fetch_official_quotes(config).await;
    let aggregated = match official_quotes {
        Ok(quotes) if !quotes.is_empty() => Some(aggregate_official_quotes(quotes)?),
        Ok(_) => None,
        Err(error) => {
            warn!(error = ?error, "official source fetch failed, falling back to carry-forward");
            None
        }
    };
    let carry_forward = aggregated.is_none();

    let market_twap = fetch_market_twap(config).await;
    if let Some(error) = market_twap.as_ref().err() {
        warn!(error = ?error, "market TWAP fetch failed; continuing without sanity check");
    }
    let market_twap = market_twap.ok().flatten();

    let submissions = build_reference_submissions(
        aggregated.as_ref(),
        market_twap.as_ref(),
        &oracle_state,
        &feed_set,
        &config.policy,
    )?;

    let mut submitted = 0usize;
    let mut skipped = 0usize;
    let mut signatures = Vec::new();

    for submission in submissions {
        if should_skip_submission(submission.expected_status, &oracle_state, &config.policy) {
            skipped += 1;
            metrics.record_skip();
            warn!(
                symbol = %submission.feed_symbol,
                status = ?submission.expected_status,
                "skipping oracle update because halt policy forbids submission"
            );
            continue;
        }

        let feed_pubkey = feed_pubkey_for_symbol(config, &submission.feed_symbol)?;
        let signature = rpc
            .submit_update(&config_pubkey, &publisher_set_pubkey, &feed_pubkey, &submission)
            .await?;

        info!(
            symbol = %submission.feed_symbol,
            price = submission.price,
            conf = submission.conf,
            sequence = submission.sequence,
            status = ?submission.expected_status,
            signature = %signature,
            "submitted oracle update"
        );

        if submission.feed_symbol == "KZTE/USD" {
            metrics.record_submission(submission.sequence, submission.price);
        }
        submitted += 1;
        signatures.push(signature.to_string());
    }

    Ok(RunSummary {
        submitted,
        skipped,
        signatures,
        used_official_sources: aggregated.as_ref().map(|value| value.source_count as usize).unwrap_or(0),
        carry_forward,
    })
}

async fn fetch_official_quotes(config: &FeederConfig) -> Result<Vec<kzte_common::SourceQuote>> {
    let mut adapters: Vec<Box<dyn SourceAdapter>> = Vec::new();
    if config.source.official_page.enabled {
        adapters.push(Box::new(NbkOfficialPageAdapter::new(
            config.source.official_page.url.clone(),
            config.source.official_page.timeout_ms,
        )?));
    }
    if config.source.open_data.enabled && !config.source.open_data.url.trim().is_empty() {
        adapters.push(Box::new(NbkOpenDataAdapter::new(
            config.source.open_data.clone(),
        )?));
    }

    let results = join_all(adapters.into_iter().map(|adapter| async move {
        fetch_with_retry(adapter.as_ref(), config.policy.retry_count, config.policy.retry_backoff_ms).await
    }))
    .await;

    let mut quotes = Vec::new();
    for result in results {
        match result {
            Ok(quote) => quotes.push(quote),
            Err(error) => warn!(error = ?error, "official adapter failed"),
        }
    }

    if quotes.is_empty() {
        Err(anyhow::anyhow!("no official sources returned a usable quote"))
    } else {
        Ok(quotes)
    }
}

async fn fetch_market_twap(config: &FeederConfig) -> Result<Option<kzte_common::SourceQuote>> {
    #[cfg(feature = "market-twap")]
    {
        use crate::adapters::OptionalMarketTwapAdapter;

        if config.source.market_twap.enabled && !config.source.market_twap.url.trim().is_empty() {
            let adapter = OptionalMarketTwapAdapter::new(config.source.market_twap.clone())?;
            let quote =
                fetch_with_retry(&adapter, config.policy.retry_count, config.policy.retry_backoff_ms).await?;
            return Ok(Some(quote));
        }
    }

    #[cfg(not(feature = "market-twap"))]
    if config.source.market_twap.enabled {
        warn!("market TWAP is enabled in config but the binary was built without the market-twap feature");
    }

    Ok(None)
}

async fn fetch_with_retry(
    adapter: &dyn SourceAdapter,
    retry_count: usize,
    retry_backoff_ms: u64,
) -> Result<kzte_common::SourceQuote> {
    let mut attempts = 0usize;
    loop {
        attempts += 1;
        match adapter.fetch().await {
            Ok(value) => return Ok(value),
            Err(error) if attempts <= retry_count => {
                warn!(
                    source = adapter.source_name(),
                    attempt = attempts,
                    error = ?error,
                    "source adapter failed, retrying"
                );
                sleep(Duration::from_millis(retry_backoff_ms.saturating_mul(attempts as u64))).await;
            }
            Err(error) => {
                return Err(error).context(format!(
                    "source adapter {} exhausted {} retries",
                    adapter.source_name(),
                    retry_count
                ));
            }
        }
    }
}

fn feed_pubkey_for_symbol(config: &FeederConfig, symbol: &str) -> Result<Pubkey> {
    match symbol {
        "KZTE/KZT" => Pubkey::from_str(&config.oracle.feed_kzte_kzt)
            .with_context(|| format!("invalid feed pubkey for {}", symbol)),
        "KZTE/USD" => Pubkey::from_str(&config.oracle.feed_kzte_usd)
            .with_context(|| format!("invalid feed pubkey for {}", symbol)),
        "KZTE/USDC" => Pubkey::from_str(&config.oracle.feed_kzte_usdc)
            .with_context(|| format!("invalid feed pubkey for {}", symbol)),
        other => Err(anyhow::anyhow!("unknown feed symbol {}", other)),
    }
}
