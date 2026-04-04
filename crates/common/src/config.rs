use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::types::RateConvention;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcConfig {
    pub http_url: String,
    pub ws_url: String,
    pub keypair_path: String,
    pub program_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleAddresses {
    pub config_pubkey: String,
    pub publisher_set_pubkey: String,
    pub feed_kzte_kzt: String,
    pub feed_kzte_usd: String,
    #[serde(default)]
    pub feed_kzte_usdc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfficialPageSourceConfig {
    pub enabled: bool,
    pub url: String,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenDataSourceConfig {
    pub enabled: bool,
    pub url: String,
    pub timeout_ms: u64,
    pub rate_json_pointer: String,
    pub publish_time_json_pointer: String,
    pub rate_convention: RateConvention,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketTwapSourceConfig {
    pub enabled: bool,
    pub url: String,
    pub timeout_ms: u64,
    pub price_json_pointer: String,
    pub publish_time_json_pointer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConfig {
    pub interval_seconds: u64,
    pub retry_count: usize,
    pub retry_backoff_ms: u64,
    pub base_confidence_bps: u32,
    pub carry_forward_multiplier_bps: u32,
    pub soft_stale_multiplier_bps: u32,
    pub stale_multiplier_bps: u32,
    pub diverged_multiplier_bps: u32,
    pub minimum_confidence: u64,
    pub publish_kzte_usdc: bool,
    pub allow_submit_when_halted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub listen_addr: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceGroupConfig {
    pub official_page: OfficialPageSourceConfig,
    pub open_data: OpenDataSourceConfig,
    pub market_twap: MarketTwapSourceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeederConfig {
    pub rpc: RpcConfig,
    pub oracle: OracleAddresses,
    pub source: SourceGroupConfig,
    pub policy: PolicyConfig,
    pub metrics: MetricsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliOracleConfig {
    pub config_pubkey: String,
    pub publisher_set_pubkey: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliFeedsConfig {
    pub kzte_kzt: String,
    pub kzte_usd: String,
    #[serde(default)]
    pub kzte_usdc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliConfig {
    pub rpc: RpcConfig,
    pub oracle: CliOracleConfig,
    pub feeds: CliFeedsConfig,
}

pub fn load_toml_file<T>(path: impl AsRef<Path>) -> Result<T, ConfigError>
where
    T: DeserializeOwned,
{
    let raw = fs::read_to_string(path.as_ref())?;
    Ok(toml::from_str(&raw)?)
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to read config: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse config: {0}")]
    Toml(#[from] toml::de::Error),
}
