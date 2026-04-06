pub mod business_day;
pub mod config;
pub mod math;
pub mod types;

pub use business_day::{
    carry_forward_decision, classify_staleness, CarryForwardDecision, StalenessTier,
};
pub use config::{
    load_toml_file, CliConfig, FeederConfig, MarketTwapSourceConfig, MetricsConfig,
    OfficialPageSourceConfig, OpenDataSourceConfig, OracleAddresses, PolicyConfig, RpcConfig,
};
pub use math::{
    calculate_deviation_bps, checked_mul_div_i64, confidence_from_bps, decimal_to_scaled_i64,
    derive_kzte_usd_from_kzt_per_usd, derive_usd_per_kzt_from_kzt_per_usd, scaled_to_f64_lossy,
    MathError, PRICE_SCALE,
};
pub use types::{
    AggregatedOfficialQuote, FeedStatus, HaltBehavior, RateConvention, ReferenceSubmission,
    SourceQuote,
};
