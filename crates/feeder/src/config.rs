use anyhow::{Context, Result};
use kzte_common::{load_toml_file, FeederConfig};
use std::env;
use std::path::{Path, PathBuf};

pub fn load_config(path: impl AsRef<Path>) -> Result<FeederConfig> {
    let mut config: FeederConfig = load_toml_file(path.as_ref())
        .with_context(|| format!("failed to load feeder config from {}", path.as_ref().display()))?;

    if let Ok(value) = env::var("SOLANA_RPC_URL") {
        config.rpc.http_url = value;
    }
    if let Ok(value) = env::var("SOLANA_WS_URL") {
        config.rpc.ws_url = value;
    }
    if let Ok(value) = env::var("SOLANA_KEYPAIR_PATH") {
        config.rpc.keypair_path = value;
    }
    if let Ok(value) = env::var("KZTE_ORACLE_PROGRAM_ID") {
        config.rpc.program_id = value;
    }
    if let Ok(value) = env::var("KZTE_ORACLE_CONFIG_PUBKEY") {
        config.oracle.config_pubkey = value;
    }
    if let Ok(value) = env::var("KZTE_ORACLE_PUBLISHER_SET_PUBKEY") {
        config.oracle.publisher_set_pubkey = value;
    }
    if let Ok(value) = env::var("KZTE_FEED_KZTE_KZT_PUBKEY") {
        config.oracle.feed_kzte_kzt = value;
    }
    if let Ok(value) = env::var("KZTE_FEED_KZTE_USD_PUBKEY") {
        config.oracle.feed_kzte_usd = value;
    }
    if let Ok(value) = env::var("KZTE_FEED_KZTE_USDC_PUBKEY") {
        config.oracle.feed_kzte_usdc = value;
    }
    if let Ok(value) = env::var("NBK_OFFICIAL_PAGE_URL") {
        config.source.official_page.url = value;
    }
    if let Ok(value) = env::var("NBK_OPEN_DATA_URL") {
        config.source.open_data.url = value;
    }
    if let Ok(value) = env::var("MARKET_TWAP_URL") {
        config.source.market_twap.url = value;
    }

    config.rpc.keypair_path = expand_tilde(&config.rpc.keypair_path)?.display().to_string();
    Ok(config)
}

fn expand_tilde(path: &str) -> Result<PathBuf> {
    if let Some(stripped) = path.strip_prefix("~/") {
        let home = env::var("USERPROFILE")
            .or_else(|_| env::var("HOME"))
            .context("could not resolve user home directory for keypair path")?;
        Ok(PathBuf::from(home).join(stripped))
    } else {
        Ok(PathBuf::from(path))
    }
}
