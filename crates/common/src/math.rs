use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use thiserror::Error;

pub const PRICE_SCALE: i64 = 100_000_000;

#[derive(Debug, Error)]
pub enum MathError {
    #[error("division by zero")]
    DivisionByZero,
    #[error("integer overflow")]
    Overflow,
    #[error("negative price is not allowed")]
    NegativePrice,
    #[error("decimal conversion failed")]
    DecimalConversion,
}

pub fn checked_mul_div_i64(lhs: i64, rhs: i64, divisor: i64) -> Result<i64, MathError> {
    if divisor == 0 {
        return Err(MathError::DivisionByZero);
    }

    let value = (lhs as i128)
        .checked_mul(rhs as i128)
        .ok_or(MathError::Overflow)?
        .checked_div(divisor as i128)
        .ok_or(MathError::DivisionByZero)?;

    i64::try_from(value).map_err(|_| MathError::Overflow)
}

pub fn decimal_to_scaled_i64(value: Decimal, scale: i64) -> Result<i64, MathError> {
    if value.is_sign_negative() {
        return Err(MathError::NegativePrice);
    }

    let scaled = value
        .checked_mul(Decimal::from(scale))
        .ok_or(MathError::Overflow)?
        .round();

    scaled.to_i64().ok_or(MathError::DecimalConversion)
}

pub fn derive_usd_per_kzt_from_kzt_per_usd(kzt_per_usd: i64) -> Result<i64, MathError> {
    checked_mul_div_i64(PRICE_SCALE, PRICE_SCALE, kzt_per_usd)
}

pub fn derive_kzte_usd_from_kzt_per_usd(kzt_per_usd: i64) -> Result<i64, MathError> {
    let usd_per_kzt = derive_usd_per_kzt_from_kzt_per_usd(kzt_per_usd)?;
    checked_mul_div_i64(PRICE_SCALE, usd_per_kzt, PRICE_SCALE)
}

pub fn calculate_deviation_bps(reference_price: i64, observed_price: i64) -> Result<u32, MathError> {
    if reference_price <= 0 || observed_price <= 0 {
        return Err(MathError::NegativePrice);
    }

    let diff = (observed_price as i128 - reference_price as i128).abs();
    let bps = diff
        .checked_mul(10_000)
        .ok_or(MathError::Overflow)?
        .checked_div(reference_price as i128)
        .ok_or(MathError::DivisionByZero)?;

    u32::try_from(bps).map_err(|_| MathError::Overflow)
}

pub fn confidence_from_bps(price: i64, bps: u32, minimum_confidence: u64) -> Result<u64, MathError> {
    if price <= 0 {
        return Err(MathError::NegativePrice);
    }

    let widened = (price as i128)
        .checked_mul(i128::from(bps))
        .ok_or(MathError::Overflow)?
        .checked_div(10_000)
        .ok_or(MathError::DivisionByZero)?;

    let widened = u64::try_from(widened).map_err(|_| MathError::Overflow)?;
    Ok(widened.max(minimum_confidence))
}

pub fn scaled_to_f64_lossy(value: i64) -> f64 {
    value as f64 / PRICE_SCALE as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_usd_per_kzt() {
        let kzt_per_usd = decimal_to_scaled_i64(Decimal::new(47046, 2), PRICE_SCALE).unwrap();
        let usd_per_kzt = derive_usd_per_kzt_from_kzt_per_usd(kzt_per_usd).unwrap();

        assert_eq!(usd_per_kzt, 212_558);
    }

    #[test]
    fn derives_kzte_usd_from_kzt_per_usd() {
        let kzt_per_usd = decimal_to_scaled_i64(Decimal::new(47046, 2), PRICE_SCALE).unwrap();
        let kzte_usd = derive_kzte_usd_from_kzt_per_usd(kzt_per_usd).unwrap();

        assert_eq!(kzte_usd, 212_558);
    }

    #[test]
    fn computes_deviation() {
        let bps = calculate_deviation_bps(100_000_000, 101_500_000).unwrap();
        assert_eq!(bps, 150);
    }
}
