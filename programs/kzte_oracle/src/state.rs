use anchor_lang::prelude::*;

use crate::constants::{MAX_ASSET_SYMBOL_LEN, MAX_FEED_SYMBOL_LEN, MAX_PUBLISHERS};
use crate::error::OracleError;

#[account]
pub struct OracleConfig {
    pub admin: Pubkey,
    pub pending_admin: Option<Pubkey>,
    pub paused: bool,
    pub publisher_set: Pubkey,
    pub soft_stale_seconds: i64,
    pub hard_stale_seconds: i64,
    pub warn_deviation_bps: u32,
    pub halt_deviation_bps: u32,
    pub price_scale: u64,
    pub last_sequence: u64,
    pub halt_behavior: HaltBehavior,
}

impl OracleConfig {
    pub const LEN: usize = 8 + 32 + 33 + 1 + 32 + 8 + 8 + 4 + 4 + 8 + 8 + 1;
}

#[account]
pub struct PublisherSet {
    pub config: Pubkey,
    pub publishers: Vec<Pubkey>,
}

impl PublisherSet {
    pub const SEED_PREFIX: &'static [u8] = b"publisher-set";
    pub const LEN: usize = 8 + 32 + 4 + (MAX_PUBLISHERS * 32);

    pub fn replace_publishers(&mut self, config: Pubkey, publishers: Vec<Pubkey>) -> Result<()> {
        require!(publishers.len() <= MAX_PUBLISHERS, OracleError::TooManyPublishers);
        if self.config != Pubkey::default() && self.config != config {
            return err!(OracleError::PublisherSetAlreadyBound);
        }

        self.config = config;
        self.publishers = publishers;
        Ok(())
    }

    pub fn contains(&self, candidate: &Pubkey) -> bool {
        self.publishers.iter().any(|publisher| publisher == candidate)
    }
}

#[account]
pub struct FeedAccount {
    pub config: Pubkey,
    pub symbol: [u8; MAX_FEED_SYMBOL_LEN],
    pub base_symbol: [u8; MAX_ASSET_SYMBOL_LEN],
    pub quote_symbol: [u8; MAX_ASSET_SYMBOL_LEN],
    pub price: i64,
    pub conf: u64,
    pub expo: i32,
    pub publish_time: i64,
    pub prev_publish_time: i64,
    pub status: FeedStatus,
    pub source_count: u8,
    pub last_good_price: i64,
    pub peg_deviation_bps: u32,
    pub sequence: u64,
    pub is_reference_feed: bool,
    pub twap_price: Option<i64>,
    pub metadata_hash: [u8; 32],
    pub metadata_version: u32,
}

impl FeedAccount {
    pub const SEED_PREFIX: &'static [u8] = b"feed";
    pub const LEN: usize = 8 + 32 + 16 + 8 + 8 + 8 + 8 + 4 + 8 + 8 + 1 + 1 + 8 + 4 + 8 + 1 + 9 + 32 + 4;

    pub fn set_symbols(&mut self, symbol: &str, base_symbol: &str, quote_symbol: &str) -> Result<()> {
        self.symbol = encode_fixed::<MAX_FEED_SYMBOL_LEN>(symbol)?;
        self.base_symbol = encode_fixed::<MAX_ASSET_SYMBOL_LEN>(base_symbol)?;
        self.quote_symbol = encode_fixed::<MAX_ASSET_SYMBOL_LEN>(quote_symbol)?;
        Ok(())
    }
}

#[derive(
    AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, InitSpace,
)]
pub enum FeedStatus {
    Active,
    CarryForward,
    Stale,
    Diverged,
    Paused,
    Halted,
}

#[derive(
    AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, InitSpace,
)]
pub enum HaltBehavior {
    Reject,
    StoreHalted,
}

pub fn encode_fixed<const N: usize>(value: &str) -> std::result::Result<[u8; N], OracleError> {
    let trimmed = value.trim();
    let bytes = trimmed.as_bytes();
    if bytes.len() > N {
        return Err(if N == MAX_FEED_SYMBOL_LEN {
            OracleError::FeedSymbolTooLong
        } else {
            OracleError::AssetSymbolTooLong
        });
    }

    let mut out = [0u8; N];
    out[..bytes.len()].copy_from_slice(bytes);
    Ok(out)
}
