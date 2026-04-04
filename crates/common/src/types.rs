use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FeedStatus {
    Active,
    CarryForward,
    Stale,
    Diverged,
    Paused,
    Halted,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RateConvention {
    KztPerUsd,
    UsdPerKzt,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HaltBehavior {
    Reject,
    StoreHalted,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceQuote {
    pub pair: String,
    pub price: i64,
    pub publish_time: i64,
    pub observed_at: i64,
    pub source_name: String,
    pub raw_payload_hash: [u8; 32],
    pub confidence_hint: Option<u64>,
    pub convention: RateConvention,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AggregatedOfficialQuote {
    pub pair: String,
    pub price: i64,
    pub publish_time: i64,
    pub observed_at: i64,
    pub source_name: String,
    pub raw_payload_hash: [u8; 32],
    pub source_count: u8,
    pub confidence_hint: Option<u64>,
    pub convention: RateConvention,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReferenceSubmission {
    pub feed_symbol: String,
    pub base_symbol: String,
    pub quote_symbol: String,
    pub price: i64,
    pub conf: u64,
    pub expo: i32,
    pub publish_time: i64,
    pub observed_at: i64,
    pub source_count: u8,
    pub sequence: u64,
    pub peg_deviation_bps: u32,
    pub twap_price: Option<i64>,
    pub raw_payload_hash: [u8; 32],
    pub metadata_version: u32,
    pub expected_status: FeedStatus,
    pub is_reference_feed: bool,
}
