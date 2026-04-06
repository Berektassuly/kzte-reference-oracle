use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use kzte_common::{
    calculate_deviation_bps, carry_forward_decision, checked_mul_div_i64, classify_staleness,
    confidence_from_bps, derive_kzte_usd_from_kzt_per_usd, FeedStatus, HaltBehavior, PolicyConfig,
    RateConvention, ReferenceSubmission, SourceQuote, StalenessTier, PRICE_SCALE,
};

#[derive(Debug, Clone)]
pub struct OracleThresholds {
    pub soft_stale_seconds: i64,
    pub hard_stale_seconds: i64,
    pub warn_deviation_bps: u32,
    pub halt_deviation_bps: u32,
    pub halt_behavior: HaltBehavior,
    pub last_sequence: u64,
}

#[derive(Debug, Clone)]
pub struct FeedSnapshot {
    pub price: i64,
    pub last_good_price: i64,
    pub publish_time: i64,
    pub source_count: u8,
    pub metadata_hash: [u8; 32],
    pub metadata_version: u32,
}

#[derive(Debug, Clone)]
pub struct FeedSet {
    pub kzte_kzt: FeedSnapshot,
    pub kzte_usd: FeedSnapshot,
    pub kzte_usdc: Option<FeedSnapshot>,
}

#[derive(Debug, Clone)]
pub struct AggregatedOfficialQuote {
    pub kzt_per_usd: i64,
    pub publish_time: i64,
    pub observed_at: i64,
    pub raw_payload_hash: [u8; 32],
    pub source_count: u8,
}

pub fn aggregate_official_quotes(mut quotes: Vec<SourceQuote>) -> Result<AggregatedOfficialQuote> {
    if quotes.is_empty() {
        return Err(anyhow!("at least one official source quote is required"));
    }

    let mut normalized_prices = quotes
        .iter()
        .map(normalize_to_kzt_per_usd)
        .collect::<Result<Vec<_>>>()?;
    normalized_prices.sort_unstable();
    quotes.sort_by_key(|quote| quote.publish_time);

    let median_price = normalized_prices[normalized_prices.len() / 2];
    let publish_time = quotes
        .iter()
        .map(|quote| quote.publish_time)
        .max()
        .context("missing publish time in official quotes")?;
    let observed_at = quotes
        .iter()
        .map(|quote| quote.observed_at)
        .max()
        .unwrap_or_else(|| Utc::now().timestamp());
    let raw_payload_hash = combine_hashes(quotes.iter().map(|quote| quote.raw_payload_hash));
    let source_count = u8::try_from(quotes.len()).context("too many official sources")?;

    Ok(AggregatedOfficialQuote {
        kzt_per_usd: median_price,
        publish_time,
        observed_at,
        raw_payload_hash,
        source_count,
    })
}

pub fn build_reference_submissions(
    official_quote: Option<&AggregatedOfficialQuote>,
    market_twap: Option<&SourceQuote>,
    oracle: &OracleThresholds,
    feeds: &FeedSet,
    policy: &PolicyConfig,
) -> Result<Vec<ReferenceSubmission>> {
    let observed_at = official_quote
        .map(|quote| quote.observed_at)
        .unwrap_or_else(|| Utc::now().timestamp());

    let (
        kzte_kzt_price,
        kzte_usd_price,
        publish_time,
        source_count,
        raw_payload_hash,
        metadata_version,
    ) = if let Some(quote) = official_quote {
        (
            PRICE_SCALE,
            derive_kzte_usd_from_kzt_per_usd(quote.kzt_per_usd)?,
            quote.publish_time,
            quote.source_count,
            quote.raw_payload_hash,
            feeds.kzte_usd.metadata_version.saturating_add(1),
        )
    } else {
        let usd_price = if feeds.kzte_usd.last_good_price > 0 {
            feeds.kzte_usd.last_good_price
        } else {
            feeds.kzte_usd.price
        };
        if usd_price <= 0 {
            return Err(anyhow!(
                "cannot carry forward without an existing last_good_price for KZTE/USD"
            ));
        }
        (
            if feeds.kzte_kzt.last_good_price > 0 {
                feeds.kzte_kzt.last_good_price
            } else {
                PRICE_SCALE
            },
            usd_price,
            feeds.kzte_usd.publish_time,
            feeds.kzte_usd.source_count.max(1),
            feeds.kzte_usd.metadata_hash,
            feeds.kzte_usd.metadata_version,
        )
    };

    let deviation_bps = market_twap
        .map(|quote| calculate_deviation_bps(kzte_usd_price, quote.price))
        .transpose()?
        .unwrap_or(0);

    let mut next_sequence = oracle.last_sequence + 1;
    let mut submissions = Vec::with_capacity(if policy.publish_kzte_usdc { 3 } else { 2 });

    let kzt_status = expected_status(
        feeds.kzte_kzt.publish_time,
        publish_time,
        observed_at,
        None,
        oracle,
    )?;
    let kzt_conf = confidence_for_submission(
        kzte_kzt_price,
        publish_time,
        observed_at,
        source_count,
        matches!(kzt_status, FeedStatus::CarryForward),
        matches!(kzt_status, FeedStatus::Diverged),
        oracle,
        policy,
    )?;
    submissions.push(ReferenceSubmission {
        feed_symbol: "KZTE/KZT".to_string(),
        base_symbol: "KZTE".to_string(),
        quote_symbol: "KZT".to_string(),
        price: kzte_kzt_price,
        conf: kzt_conf,
        expo: -8,
        publish_time,
        observed_at,
        source_count,
        sequence: next_sequence,
        peg_deviation_bps: 0,
        twap_price: None,
        raw_payload_hash,
        metadata_version,
        expected_status: kzt_status,
        is_reference_feed: true,
    });
    next_sequence += 1;

    let usd_status = expected_status(
        feeds.kzte_usd.publish_time,
        publish_time,
        observed_at,
        market_twap.map(|quote| (quote.price, deviation_bps)),
        oracle,
    )?;
    let usd_conf = confidence_for_submission(
        kzte_usd_price,
        publish_time,
        observed_at,
        source_count,
        matches!(usd_status, FeedStatus::CarryForward),
        matches!(usd_status, FeedStatus::Diverged | FeedStatus::Halted),
        oracle,
        policy,
    )?;
    submissions.push(ReferenceSubmission {
        feed_symbol: "KZTE/USD".to_string(),
        base_symbol: "KZTE".to_string(),
        quote_symbol: "USD".to_string(),
        price: kzte_usd_price,
        conf: usd_conf,
        expo: -8,
        publish_time,
        observed_at,
        source_count,
        sequence: next_sequence,
        peg_deviation_bps: deviation_bps,
        twap_price: market_twap.map(|quote| quote.price),
        raw_payload_hash,
        metadata_version,
        expected_status: usd_status,
        is_reference_feed: true,
    });
    next_sequence += 1;

    if policy.publish_kzte_usdc {
        if let Some(feed) = &feeds.kzte_usdc {
            let usdc_status =
                expected_status(feed.publish_time, publish_time, observed_at, None, oracle)?;
            let usdc_conf = confidence_for_submission(
                kzte_usd_price,
                publish_time,
                observed_at,
                source_count,
                matches!(usdc_status, FeedStatus::CarryForward),
                matches!(usdc_status, FeedStatus::Diverged),
                oracle,
                policy,
            )?;
            submissions.push(ReferenceSubmission {
                feed_symbol: "KZTE/USDC".to_string(),
                base_symbol: "KZTE".to_string(),
                quote_symbol: "USDC".to_string(),
                price: kzte_usd_price,
                conf: usdc_conf,
                expo: -8,
                publish_time,
                observed_at,
                source_count,
                sequence: next_sequence,
                peg_deviation_bps: 0,
                twap_price: None,
                raw_payload_hash,
                metadata_version,
                expected_status: usdc_status,
                is_reference_feed: false,
            });
        }
    }

    Ok(submissions)
}

pub fn expected_status(
    previous_publish_time: i64,
    publish_time: i64,
    observed_at: i64,
    market_context: Option<(i64, u32)>,
    oracle: &OracleThresholds,
) -> Result<FeedStatus> {
    let staleness = classify_staleness(
        publish_time,
        observed_at,
        oracle.soft_stale_seconds,
        oracle.hard_stale_seconds,
    )?;

    if let Some((_market_price, deviation_bps)) = market_context {
        if deviation_bps > oracle.halt_deviation_bps {
            return Ok(FeedStatus::Halted);
        }
        if deviation_bps > oracle.warn_deviation_bps && staleness != StalenessTier::HardStale {
            return Ok(FeedStatus::Diverged);
        }
    }

    if staleness == StalenessTier::HardStale {
        return Ok(FeedStatus::Stale);
    }

    let carry_forward_state = carry_forward_decision(
        previous_publish_time,
        publish_time,
        observed_at,
        oracle.hard_stale_seconds,
    )?;
    if matches!(
        carry_forward_state,
        kzte_common::CarryForwardDecision::CarryForward
    ) {
        return Ok(FeedStatus::CarryForward);
    }

    Ok(FeedStatus::Active)
}

pub fn should_skip_submission(
    status: FeedStatus,
    oracle: &OracleThresholds,
    policy: &PolicyConfig,
) -> bool {
    matches!(status, FeedStatus::Halted)
        && (matches!(oracle.halt_behavior, HaltBehavior::Reject)
            || !policy.allow_submit_when_halted)
}

fn confidence_for_submission(
    price: i64,
    publish_time: i64,
    observed_at: i64,
    source_count: u8,
    carry_forward: bool,
    diverged: bool,
    oracle: &OracleThresholds,
    policy: &PolicyConfig,
) -> Result<u64> {
    let mut bps = policy.base_confidence_bps.max(1);
    if source_count > 1 {
        bps = ((u64::from(bps) * 75) / 100).max(1) as u32;
    }

    let staleness = classify_staleness(
        publish_time,
        observed_at,
        oracle.soft_stale_seconds,
        oracle.hard_stale_seconds,
    )?;
    if carry_forward {
        bps = scale_bps(bps, policy.carry_forward_multiplier_bps);
    }
    if matches!(staleness, StalenessTier::SoftStale) {
        bps = scale_bps(bps, policy.soft_stale_multiplier_bps);
    }
    if matches!(staleness, StalenessTier::HardStale) {
        bps = scale_bps(bps, policy.stale_multiplier_bps);
    }
    if diverged {
        bps = scale_bps(bps, policy.diverged_multiplier_bps);
    }

    confidence_from_bps(price, bps.max(1), policy.minimum_confidence).map_err(Into::into)
}

fn normalize_to_kzt_per_usd(quote: &SourceQuote) -> Result<i64> {
    match quote.convention {
        RateConvention::KztPerUsd => Ok(quote.price),
        RateConvention::UsdPerKzt => {
            checked_mul_div_i64(PRICE_SCALE, PRICE_SCALE, quote.price).map_err(Into::into)
        }
    }
}

fn combine_hashes(hashes: impl Iterator<Item = [u8; 32]>) -> [u8; 32] {
    let mut merged = [0u8; 32];
    for hash in hashes {
        for (slot, value) in merged.iter_mut().zip(hash) {
            *slot ^= value;
        }
    }
    merged
}

fn scale_bps(base_bps: u32, multiplier_bps: u32) -> u32 {
    ((u64::from(base_bps) * u64::from(multiplier_bps)) / 10_000)
        .max(1)
        .min(u64::from(u32::MAX)) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use kzte_common::{HaltBehavior, PolicyConfig};

    fn policy() -> PolicyConfig {
        PolicyConfig {
            interval_seconds: 300,
            retry_count: 3,
            retry_backoff_ms: 500,
            base_confidence_bps: 25,
            carry_forward_multiplier_bps: 20_000,
            soft_stale_multiplier_bps: 20_000,
            stale_multiplier_bps: 50_000,
            diverged_multiplier_bps: 30_000,
            minimum_confidence: 1,
            publish_kzte_usdc: false,
            allow_submit_when_halted: true,
        }
    }

    fn oracle() -> OracleThresholds {
        OracleThresholds {
            soft_stale_seconds: 86_400,
            hard_stale_seconds: 3 * 86_400,
            warn_deviation_bps: 100,
            halt_deviation_bps: 500,
            halt_behavior: HaltBehavior::StoreHalted,
            last_sequence: 41,
        }
    }

    #[test]
    fn derives_kzte_usd_submission_from_usd_kzt() {
        let aggregated = AggregatedOfficialQuote {
            kzt_per_usd: 47_046_000_000,
            publish_time: 1_743_801_600,
            observed_at: 1_743_810_600,
            raw_payload_hash: [7u8; 32],
            source_count: 1,
        };
        let feeds = FeedSet {
            kzte_kzt: FeedSnapshot {
                price: PRICE_SCALE,
                last_good_price: PRICE_SCALE,
                publish_time: 1_743_715_200,
                source_count: 1,
                metadata_hash: [1u8; 32],
                metadata_version: 1,
            },
            kzte_usd: FeedSnapshot {
                price: 212_000,
                last_good_price: 212_000,
                publish_time: 1_743_715_200,
                source_count: 1,
                metadata_hash: [2u8; 32],
                metadata_version: 1,
            },
            kzte_usdc: None,
        };

        let submissions =
            build_reference_submissions(Some(&aggregated), None, &oracle(), &feeds, &policy())
                .unwrap();
        let usd = submissions
            .iter()
            .find(|submission| submission.feed_symbol == "KZTE/USD")
            .unwrap();

        assert_eq!(usd.price, 212_558);
        assert_eq!(usd.sequence, 43);
    }

    #[test]
    fn soft_and_hard_stale_statuses_transition() {
        let oracle = oracle();

        let active = expected_status(100, 200, 201, None, &oracle).unwrap();
        let stale = expected_status(100, 200, 200 + (4 * 86_400), None, &oracle).unwrap();

        assert_eq!(active, FeedStatus::Active);
        assert_eq!(stale, FeedStatus::Stale);
    }

    #[test]
    fn deviation_thresholds_are_applied() {
        let oracle = oracle();

        let diverged = expected_status(100, 200, 201, Some((220_000, 150)), &oracle).unwrap();
        let halted = expected_status(100, 200, 201, Some((220_000, 700)), &oracle).unwrap();

        assert_eq!(diverged, FeedStatus::Diverged);
        assert_eq!(halted, FeedStatus::Halted);
    }
}
