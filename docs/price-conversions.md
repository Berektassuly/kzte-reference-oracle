# Price Conversions

This document explains how the oracle represents exchange rates, converts between
rate conventions, and derives `KZTE/USD` from the official NBK `USD/KZT` quote.

## Canonical Scale

All prices are stored as signed integers at a fixed `1e8` scale:

- `PRICE_SCALE = 100_000_000`
- `1.00000000` is stored as `100_000_000`
- `470.46000000` is stored as `47_046_000_000`

This keeps the on-chain program and feeder aligned on a single lossless format
for values that fit inside `i64`.

## Supported Conventions

Official source quotes carry both a display pair and a numeric convention:

- Pair label `USD/KZT`
- Convention `KztPerUsd`
- Meaning: "how many KZT equal 1 USD"

The code also supports `UsdPerKzt`, which means "how many USD equal 1 KZT".

The pair label is for human-readable identification. Conversion logic must use
the numeric convention field rather than infer direction from the string alone.

## Reciprocal Conversion Rule

When the feeder needs `USD per KZT` from an official `KZT per USD` quote, it
computes:

```text
usd_per_kzt = PRICE_SCALE * PRICE_SCALE / kzt_per_usd
```

That result is rounded to the nearest integer at `1e8` precision.

Rounding behavior:

- values below the half step round toward zero;
- values at or above the half step round away from zero;
- this keeps reciprocal conversions symmetric and avoids a truncation bias.

The helper responsible for this is `checked_mul_div_i64` in
`crates/common/src/math.rs`.

## KZTE/USD Derivation

`KZTE` is treated as a KZT-backed reference stablecoin, so:

- `KZTE/KZT = 1.00000000`
- `KZTE/USD = USD per KZT`

That means `derive_kzte_usd_from_kzt_per_usd` intentionally reuses the same
reciprocal path as `derive_usd_per_kzt_from_kzt_per_usd`.

## Worked Example

Official NBK rate:

- `USD/KZT = 470.46`
- scaled `KztPerUsd = 47_046_000_000`

Derived reciprocal:

```text
100_000_000 * 100_000_000 / 47_046_000_000
= 212557.922033754198...
```

After nearest-integer rounding at `1e8` precision:

- `USD per KZT = 212_558`
- rendered decimal value: `0.00212558`

Because `KZTE` tracks KZT, the same scaled result is also used for `KZTE/USD`.

## Test Expectations

Regression coverage should keep these expectations stable:

- reciprocal derivation for `470.46` produces `212_558`;
- reciprocal math rounds to nearest integer instead of truncating;
- negative half-step behavior stays symmetric in the shared helper even though
  production prices are expected to be positive.
