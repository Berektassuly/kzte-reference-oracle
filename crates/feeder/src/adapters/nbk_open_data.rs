use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use kzte_common::{decimal_to_scaled_i64, OpenDataSourceConfig, SourceQuote, PRICE_SCALE};
use reqwest::Client;
use rust_decimal::Decimal;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::str::FromStr;
use std::time::Duration;

use super::SourceAdapter;

pub struct NbkOpenDataAdapter {
    client: Client,
    config: OpenDataSourceConfig,
}

impl NbkOpenDataAdapter {
    pub fn new(config: OpenDataSourceConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_millis(config.timeout_ms))
            .user_agent("kzte-reference-oracle/0.1.0")
            .build()
            .context("failed to build HTTP client for NBK open data adapter")?;

        Ok(Self { client, config })
    }

    fn parse(&self, body: &str, observed_at: DateTime<Utc>) -> Result<SourceQuote> {
        let json: Value = serde_json::from_str(body).context("failed to parse NBK open data payload")?;
        let price_value = json
            .pointer(&self.config.rate_json_pointer)
            .ok_or_else(|| anyhow!("configured rate_json_pointer did not match the NBK open data payload"))?;
        let publish_time_value = json
            .pointer(&self.config.publish_time_json_pointer)
            .ok_or_else(|| anyhow!("configured publish_time_json_pointer did not match the NBK open data payload"))?;

        let price = parse_decimal_value(price_value)?;
        let publish_time = parse_time_value(publish_time_value)?;
        let scaled_price = decimal_to_scaled_i64(price, PRICE_SCALE)?;
        let raw_payload_hash = Sha256::digest(body.as_bytes()).into();

        Ok(SourceQuote {
            pair: "USD/KZT".to_string(),
            price: scaled_price,
            publish_time,
            observed_at: observed_at.timestamp(),
            source_name: self.source_name().to_string(),
            raw_payload_hash,
            confidence_hint: None,
            convention: self.config.rate_convention,
        })
    }
}

#[async_trait]
impl SourceAdapter for NbkOpenDataAdapter {
    async fn fetch(&self) -> Result<SourceQuote> {
        let response = self
            .client
            .get(&self.config.url)
            .send()
            .await
            .with_context(|| format!("failed to fetch NBK open data endpoint {}", self.config.url))?
            .error_for_status()
            .with_context(|| format!("NBK open data endpoint returned non-success status for {}", self.config.url))?;
        let body = response.text().await.context("failed to read NBK open data body")?;
        self.parse(&body, Utc::now())
    }

    fn source_name(&self) -> &'static str {
        "nbk_open_data"
    }
}

fn parse_decimal_value(value: &Value) -> Result<Decimal> {
    match value {
        Value::String(inner) => Decimal::from_str(inner.trim()).context("failed to parse string decimal"),
        Value::Number(inner) => Decimal::from_str(&inner.to_string()).context("failed to parse numeric decimal"),
        _ => Err(anyhow!("open data decimal field must be a string or number")),
    }
}

fn parse_time_value(value: &Value) -> Result<i64> {
    match value {
        Value::String(inner) => {
            if let Ok(parsed) = DateTime::parse_from_rfc3339(inner) {
                return Ok(parsed.timestamp());
            }
            let date = NaiveDate::parse_from_str(inner, "%Y-%m-%d")
                .with_context(|| format!("unsupported open data date format {}", inner))?;
            Ok(date
                .and_hms_opt(0, 0, 0)
                .ok_or_else(|| anyhow!("invalid open data midnight"))?
                .and_utc()
                .timestamp())
        }
        Value::Number(inner) => inner
            .as_i64()
            .ok_or_else(|| anyhow!("open data publish time number must be an i64 epoch timestamp")),
        _ => Err(anyhow!("open data publish time field must be a string or number")),
    }
}
