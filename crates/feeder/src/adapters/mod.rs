use anyhow::Result;
use async_trait::async_trait;
use kzte_common::SourceQuote;

pub mod nbk_official_page;
pub mod nbk_open_data;

#[cfg(feature = "market-twap")]
pub mod optional_market_twap;

pub use nbk_official_page::NbkOfficialPageAdapter;
pub use nbk_open_data::NbkOpenDataAdapter;

#[cfg(feature = "market-twap")]
pub use optional_market_twap::OptionalMarketTwapAdapter;

#[async_trait]
pub trait SourceAdapter: Send + Sync {
    async fn fetch(&self) -> Result<SourceQuote>;
    fn source_name(&self) -> &'static str;
}
