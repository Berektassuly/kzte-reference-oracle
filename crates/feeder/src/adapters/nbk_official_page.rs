use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Datelike, FixedOffset, NaiveDate, TimeZone, Utc, Weekday};
use kzte_common::{decimal_to_scaled_i64, RateConvention, SourceQuote, PRICE_SCALE};
use regex::Regex;
use reqwest::Client;
use rust_decimal::Decimal;
use scraper::{ElementRef, Html, Selector};
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
        let document = Html::parse_document(html);
        let visible_text = visible_text(&document);
        let effective_date = extract_effective_date(&document, &visible_text)?;
        let (nominal, raw_price) = extract_usd_quote(&document, &visible_text)?;
        let adjusted_price = raw_price / Decimal::from(nominal);
        let scaled_price = decimal_to_scaled_i64(adjusted_price, PRICE_SCALE)?;
        let publish_time = inferred_usd_publish_time(&effective_date)?;
        let observed_at = observed_at.timestamp();
        if observed_at < publish_time {
            return Err(anyhow!(
                "observed_at {} is earlier than inferred NBK publish_time {} for effective date {}",
                observed_at,
                publish_time,
                effective_date
            ));
        }
        let raw_payload_hash = Sha256::digest(html.as_bytes()).into();

        Ok(SourceQuote {
            pair: "USD/KZT".to_string(),
            price: scaled_price,
            publish_time,
            observed_at,
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
            .with_context(|| {
                format!(
                    "NBK official page returned non-success status for {}",
                    self.url
                )
            })?;
        let body = response
            .text()
            .await
            .context("failed to read NBK page body")?;
        Self::parse(&body, Utc::now())
    }

    fn source_name(&self) -> &'static str {
        "nbk_official_page"
    }
}

fn visible_text(document: &Html) -> String {
    document
        .root_element()
        .text()
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn extract_effective_date(document: &Html, visible_text: &str) -> Result<String> {
    let date_re =
        Regex::new(r"Official \(market\) Exchange Rates on:\s*([0-9]{4}-[0-9]{2}-[0-9]{2})")?;
    let heading_selector = Selector::parse("h3.title-section, h3")
        .map_err(|error| anyhow!("failed to build NBK heading selector: {error}"))?;

    if let Some(date) = document.select(&heading_selector).find_map(|heading| {
        let text = element_text(heading);
        date_re
            .captures(&text)
            .and_then(|captures| captures.get(1))
            .map(|value| value.as_str().to_string())
    }) {
        return Ok(date);
    }

    date_re
        .captures(visible_text)
        .and_then(|captures| captures.get(1))
        .map(|value| value.as_str().to_string())
        .ok_or_else(|| anyhow!("failed to locate official rate date on NBK page"))
}

fn extract_usd_quote(document: &Html, visible_text: &str) -> Result<(i64, Decimal)> {
    if let Some(parsed) = extract_usd_quote_from_table(document)? {
        return Ok(parsed);
    }

    let usd_re = Regex::new(r"(\d+)\s+US DOLLAR\s+USD\s*/\s*KZT\s+([0-9]+(?:\.[0-9]+)?)")?;
    let usd_caps = usd_re
        .captures(visible_text)
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

    Ok((nominal, raw_price))
}

fn extract_usd_quote_from_table(document: &Html) -> Result<Option<(i64, Decimal)>> {
    let row_selector = Selector::parse("table#exchange-table tr")
        .map_err(|error| anyhow!("failed to build NBK row selector: {error}"))?;
    let cell_selector = Selector::parse("td")
        .map_err(|error| anyhow!("failed to build NBK cell selector: {error}"))?;

    for row in document.select(&row_selector) {
        let cells = row
            .select(&cell_selector)
            .map(element_text)
            .collect::<Vec<_>>();
        let Some(pair_index) = cells
            .iter()
            .position(|cell| cell.eq_ignore_ascii_case("USD / KZT"))
        else {
            continue;
        };
        if pair_index == 0 || pair_index + 1 >= cells.len() {
            continue;
        }

        let nominal = cells[pair_index - 1]
            .split_whitespace()
            .next()
            .ok_or_else(|| anyhow!("missing nominal cell for USD/KZT row"))?
            .parse::<i64>()
            .context("invalid nominal in USD/KZT table row")?;
        let raw_price = Decimal::from_str(cells[pair_index + 1].trim())
            .context("invalid price in USD/KZT table row")?;

        return Ok(Some((nominal, raw_price)));
    }

    Ok(None)
}

fn element_text(element: ElementRef<'_>) -> String {
    element
        .text()
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn inferred_usd_publish_time(effective_date: &str) -> Result<i64> {
    let parsed = NaiveDate::parse_from_str(effective_date, "%Y-%m-%d")
        .with_context(|| format!("invalid NBK effective date {}", effective_date))?;
    let trading_day = previous_business_day(parsed);
    let astana = FixedOffset::east_opt(5 * 3600).ok_or_else(|| anyhow!("invalid fixed offset"))?;
    let datetime = astana
        .from_local_datetime(
            &trading_day
                .and_hms_opt(15, 30, 0)
                .ok_or_else(|| anyhow!("invalid NBK USD fixing time"))?,
        )
        .single()
        .ok_or_else(|| anyhow!("NBK publish date is ambiguous in Astana timezone"))?;
    Ok(datetime.timestamp())
}

fn previous_business_day(mut date: NaiveDate) -> NaiveDate {
    loop {
        date = date
            .pred_opt()
            .expect("NBK effective date must have a predecessor");
        match date.weekday() {
            Weekday::Sat | Weekday::Sun => continue,
            _ => return date,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_real_nbk_table_fixture() {
        let html =
            include_str!("../../../../tests/ezhednevnye-oficialnye-rynochnye-kursy-valyut.html");
        let observed_at = DateTime::parse_from_rfc3339("2026-04-06T18:45:00+05:00")
            .unwrap()
            .with_timezone(&Utc);

        let quote = NbkOfficialPageAdapter::parse(html, observed_at).unwrap();

        assert_eq!(quote.pair, "USD/KZT");
        assert_eq!(quote.price, 47_046_000_000);
        assert_eq!(
            quote.publish_time,
            DateTime::parse_from_rfc3339("2026-04-03T15:30:00+05:00")
                .unwrap()
                .timestamp()
        );
        assert!(quote.observed_at >= quote.publish_time);
        assert_eq!(quote.convention, RateConvention::KztPerUsd);
    }

    #[test]
    fn parses_simplified_fixture_via_text_fallback() {
        let html = include_str!("../../../../tests/fixtures/nbk_official_page_2026_04_05.html");
        let observed_at = DateTime::parse_from_rfc3339("2026-04-05T10:15:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let quote = NbkOfficialPageAdapter::parse(html, observed_at).unwrap();

        assert_eq!(quote.pair, "USD/KZT");
        assert_eq!(quote.price, 47_046_000_000);
        assert_eq!(
            quote.publish_time,
            DateTime::parse_from_rfc3339("2026-04-03T15:30:00+05:00")
                .unwrap()
                .timestamp()
        );
        assert_eq!(quote.convention, RateConvention::KztPerUsd);
    }

    #[test]
    fn previous_business_day_skips_weekends() {
        let effective_monday = NaiveDate::from_ymd_opt(2026, 4, 6).unwrap();
        let effective_tuesday = NaiveDate::from_ymd_opt(2026, 4, 7).unwrap();

        assert_eq!(
            previous_business_day(effective_monday),
            NaiveDate::from_ymd_opt(2026, 4, 3).unwrap()
        );
        assert_eq!(
            previous_business_day(effective_tuesday),
            NaiveDate::from_ymd_opt(2026, 4, 6).unwrap()
        );
    }
}
