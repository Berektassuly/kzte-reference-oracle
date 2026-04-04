use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, FixedOffset, NaiveDate, TimeZone, Utc};
use kzte_common::{decimal_to_scaled_i64, RateConvention, SourceQuote, PRICE_SCALE};
use regex::Regex;
use reqwest::Client;
use rust_decimal::Decimal;
use scraper::Html;
use sha2::{Digest, Sha256};
use std::str::FromStr;
use std::time::Duration;

use super::SourceAdapter;

pub struct NbkOfficialPageAdapter {
    client: Client,
    url: String,
}

impl NbkOfficialPageAdapter {
    pub fn new(url: impl Into<String>, timeout_ms: u64) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_millis(timeout_ms))
            .user_agent("kzte-reference-oracle/0.1.0")
            .build()
            .context("failed to build HTTP client for NBK official page adapter")?;

        Ok(Self {
            client,
            url: url.into(),
        })
    }

    pub fn parse(html: &str, observed_at: DateTime<Utc>) -> Result<SourceQuote> {
        let visible_text = visible_text(html);
        let date_re = Regex::new(r"Official \(market\) Exchange Rates on:\s*([0-9]{4}-[0-9]{2}-[0-9]{2})")?;
        let usd_re = Regex::new(r"(\d+)\s+US DOLLAR\s+USD\s*/\s*KZT\s+([0-9]+(?:\.[0-9]+)?)")?;

        let date = date_re
            .captures(&visible_text)
            .and_then(|captures| captures.get(1))
            .map(|value| value.as_str())
            .ok_or_else(|| anyhow!("failed to locate official rate date on NBK page"))?;

        let usd_caps = usd_re
            .captures(&visible_text)
            .ok_or_else(|| anyhow!("failed to locate USD/KZT rate on NBK page"))?;
        let nominal = usd_caps
            .get(1)
            .ok_or_else(|| anyhow!("missing nominal in USD/KZT line"))?
            .as_str()
            .parse::<i64>()
            .context("invalid nominal in USD/KZT line")?;
        let raw_price = Decimal::from_str(
            usd_caps
                .get(2)
                .ok_or_else(|| anyhow!("missing price in USD/KZT line"))?
                .as_str(),
        )
        .context("invalid decimal price in USD/KZT line")?;
        let adjusted_price = raw_price / Decimal::from(nominal);
        let scaled_price = decimal_to_scaled_i64(adjusted_price, PRICE_SCALE)?;

        let publish_time = astana_midnight_timestamp(date)?;
        let raw_payload_hash = Sha256::digest(html.as_bytes()).into();

        Ok(SourceQuote {
            pair: "USD/KZT".to_string(),
            price: scaled_price,
            publish_time,
            observed_at: observed_at.timestamp(),
            source_name: "nbk_official_page".to_string(),
            raw_payload_hash,
            confidence_hint: None,
            convention: RateConvention::KztPerUsd,
        })
    }
}

#[async_trait]
impl SourceAdapter for NbkOfficialPageAdapter {
    async fn fetch(&self) -> Result<SourceQuote> {
        let response = self
            .client
            .get(&self.url)
            .send()
            .await
            .with_context(|| format!("failed to fetch NBK official page at {}", self.url))?
            .error_for_status()
            .with_context(|| format!("NBK official page returned non-success status for {}", self.url))?;
        let body = response.text().await.context("failed to read NBK page body")?;
        Self::parse(&body, Utc::now())
    }

    fn source_name(&self) -> &'static str {
        "nbk_official_page"
    }
}

fn visible_text(html: &str) -> String {
    Html::parse_document(html)
        .root_element()
        .text()
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn astana_midnight_timestamp(date: &str) -> Result<i64> {
    let parsed = NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .with_context(|| format!("invalid NBK publish date {}", date))?;
    let astana = FixedOffset::east_opt(5 * 3600).ok_or_else(|| anyhow!("invalid fixed offset"))?;
    let datetime = astana
        .from_local_datetime(
            &parsed
                .and_hms_opt(0, 0, 0)
                .ok_or_else(|| anyhow!("invalid NBK publish time"))?,
        )
        .single()
        .ok_or_else(|| anyhow!("NBK publish date is ambiguous in Astana timezone"))?;
    Ok(datetime.timestamp())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_fixture() {
        let html = include_str!("../../../../tests/fixtures/nbk_official_page_2026_04_05.html");
        let observed_at = DateTime::parse_from_rfc3339("2026-04-05T10:15:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let quote = NbkOfficialPageAdapter::parse(html, observed_at).unwrap();

        assert_eq!(quote.pair, "USD/KZT");
        assert_eq!(quote.price, 47_046_000_000);
        assert_eq!(quote.convention, RateConvention::KztPerUsd);
    }
}
