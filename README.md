# KZTE Reference Oracle

Production-oriented custom reference oracle for **KZTE (Evo)**, a **KZT-backed stablecoin on Solana**.

This repository treats **KZTE as a reference stablecoin pegged to KZT**, not as a freely-priced asset.  
The oracle therefore uses the **official National Bank of Kazakhstan (NBK) exchange rate as the primary source of truth** and uses market data only as an **optional peg-divergence sanity check**.

Verified official NBK sources used for this implementation:

- Official daily rates page: <https://nationalbank.kz/en/exchangerates/ezhednevnye-oficialnye-rynochnye-kursy-valyut>
- NBK Open Data repository landing page: <https://data.nationalbank.kz/>

Important assumptions:

- The repository ships with a **working official-page adapter** against the verified NBK HTML page above.
- The **open data adapter is intentionally config-driven** because the exact dataset endpoint and payload schema were not hardcoded without explicit verification.
- The repository ships with a **dev program id** (`3jFfb765du9EE86BRxcscnasbgKZV1s5DdxtAuSJccXo`) for local work. Replace it with your deployment keypair and run `anchor keys sync` before production deploy.
- `KZTE_MINT` is **not hardcoded** anywhere in core logic. It is only relevant for optional market/TWAP integrations.

## Architecture

The system is split into three layers:

1. **On-chain Anchor program**
   - Stores oracle config, publisher allowlist, and feed accounts.
   - Enforces admin-only governance, replay protection, monotonic sequencing, and bounded account layouts.
   - Computes feed status deterministically from timestamps, divergence inputs, and config thresholds.

2. **Off-chain feeder/aggregator**
   - Fetches NBK official exchange data.
   - Normalizes all prices to fixed-point `1e8`.
   - Derives `KZTE/KZT = 1.00000000` and `KZTE/USD = 1 / (USD/KZT)` from official data using nearest-integer reciprocal rounding at `1e8` precision.
   - Optionally fetches market TWAP and computes `peg_deviation_bps`.
   - Applies carry-forward logic when a new official business-day rate is unavailable.
   - Submits `submit_update` transactions with structured logs and a `/metrics` endpoint.

3. **Rust CLI**
   - Initializes config.
   - Creates feed PDAs.
   - Reads config and feed accounts.
   - Runs admin instructions.
   - Can trigger a one-shot feeder update path.

## Repository Tree

The oracle-specific tree added in this repo is:

```text
.
|-- Anchor.toml
|-- Cargo.toml
|-- Dockerfile
|-- README.md
|-- .env.example
|-- docs
|   |-- github-ci-tests-prompt.md
|   `-- price-conversions.md
|-- config
|   |-- cli.example.toml
|   `-- feeder.example.toml
|-- crates
|   |-- cli
|   |   |-- Cargo.toml
|   |   `-- src/main.rs
|   |-- common
|   |   |-- Cargo.toml
|   |   `-- src
|   |       |-- business_day.rs
|   |       |-- config.rs
|   |       |-- lib.rs
|   |       |-- math.rs
|   |       `-- types.rs
|   `-- feeder
|       |-- Cargo.toml
|       `-- src
|           |-- adapters
|           |   |-- mod.rs
|           |   |-- nbk_official_page.rs
|           |   |-- nbk_open_data.rs
|           |   `-- optional_market_twap.rs
|           |-- aggregator.rs
|           |-- config.rs
|           |-- lib.rs
|           |-- main.rs
|           |-- metrics.rs
|           |-- service.rs
|           `-- submitter.rs
|-- programs
|   `-- kzte_oracle
|       |-- Cargo.toml
|       `-- src
|           |-- constants.rs
|           |-- error.rs
|           |-- lib.rs
|           `-- state.rs
|-- scripts
|   |-- anchor-test.ps1
|   `-- run-feeder-once.ps1
`-- tests
    |-- Cargo.toml
    |-- fixtures
    |   `-- nbk_official_page_2026_04_05.html
    `-- src
        |-- integration.rs
        `-- lib.rs
```

Note: existing unrelated `frontend/` and `backend/` directories were left untouched.

## Docs

- [`docs/price-conversions.md`](docs/price-conversions.md) explains fixed-point scale, supported rate conventions, and the reciprocal rounding rule used for `KZTE/USD`.
- [`docs/github-ci-tests-prompt.md`](docs/github-ci-tests-prompt.md) contains the repository-specific prompt for generating GitHub Actions CI coverage.

## Feeds

Implemented feed targets:

- `KZTE/KZT`
  - fixed reference price at `1e8`
- `KZTE/USD`
  - derived from the NBK official `USD/KZT` reciprocal and rounded to the nearest integer at `1e8` precision
- `KZTE/USDC`
  - optional alias feed, disabled by default
- `peg_deviation_bps`
  - optional market sanity-check field, only populated when a market TWAP adapter is configured

## Fixed-Point Rules

- Canonical scale: `PRICE_SCALE = 100_000_000`
- Program-side math uses integers only
- No floating-point math is used on-chain
- Intermediate multiplication/division uses checked arithmetic
- Reciprocal conversions round to the nearest integer at `1e8` precision instead of truncating toward zero
- Half-step reciprocal cases round away from zero to keep signed helper behavior symmetric
- `expo = -8` is enforced for submitted prices

For the full conversion notes and worked examples, see [`docs/price-conversions.md`](docs/price-conversions.md).

## Conversion Notes

- Official NBK `USD/KZT` quotes are treated numerically as `KztPerUsd`.
- `KZTE/KZT` is always `1.00000000`, so `KZTE/USD` is the same reciprocal as `USD per KZT`.
- Example: `470.46 KZT/USD` is stored as `47_046_000_000`, and its reciprocal rounds from `212557.922...` to `212_558` at `1e8` precision.

## Status Model

Status precedence is:

1. `Halted`
2. `Stale`
3. `Diverged`
4. `CarryForward`
5. `Active`

Rules:

- `CarryForward` when the official `publish_time` has not advanced but remains inside the hard stale window.
- `Stale` when `observed_at - publish_time > hard_stale_seconds`.
- `Diverged` when optional market TWAP deviation is above `warn_deviation_bps`.
- `Halted` when optional market TWAP deviation is above `halt_deviation_bps`.
- `Paused` is set by admin action or on fresh feed initialization before the first update when the config is paused.

## Weekends / Holidays / Business-Day Logic

This oracle is intentionally **not** a high-frequency trading oracle.

- If NBK publishes a new business-day official rate, feeder derives fresh reference values and submits them.
- If the official rate has not changed because of a weekend or public holiday, the feeder reuses the last good official value and resubmits it with the same `publish_time`.
- That produces `CarryForward` on-chain while widening confidence off-chain.
- If the carried-forward rate ages past `soft_stale_seconds`, confidence widens again.
- If it ages past `hard_stale_seconds`, the on-chain status becomes `Stale`.

## Confidence Model

Confidence is never published as zero.

The feeder computes `conf` from `base_confidence_bps` and widens it when:

- only a carry-forward value is available,
- age exceeds `soft_stale_seconds`,
- age exceeds `hard_stale_seconds`,
- market deviation produces `Diverged` / `Halted`.

## Source Adapters

### 1. `NbkOfficialPageAdapter`

- Working default adapter.
- Fetches the verified NBK official daily-rates HTML page.
- Reads the `#exchange-table` row for `USD / KZT`, with a text fallback for simplified fixtures.
- Treats the page date as the next business-day effective date and maps `publish_time` to the inferred previous business-day USD fixing time (`15:30` Astana).
- Normalizes to `1e8`.

### 2. `NbkOpenDataAdapter`

- Optional.
- Requires explicit config:
  - `url`
  - `rate_json_pointer`
  - `publish_time_json_pointer`
  - `rate_convention`
- This is intentionally generic to avoid hardcoding an unverified NBK dataset endpoint or schema.

### 3. `OptionalMarketTwapAdapter`

- Feature-gated: build feeder with `--features market-twap`
- Accepts a configured JSON endpoint and JSON pointers
- Intended for a future liquid KZTE market
- Not primary truth, only sanity-check input

## On-Chain Instructions

- `initialize_oracle_config`
- `transfer_admin`
- `accept_admin`
- `set_publishers`
- `set_thresholds`
- `pause_oracle`
- `resume_oracle`
- `create_feed`
- `submit_update`
- `force_set_status`

## Build Commands

```bash
cargo build
anchor build
cargo test
anchor test
```

If you want optional market TWAP support in the feeder:

```bash
cargo build -p kzte-feeder --features market-twap
```

## Example Runs

Copy `config/cli.example.toml` to `config/cli.toml` and `config/feeder.example.toml` to `config/feeder.toml` before running the commands below. The checked-in examples are prefilled for the current devnet deployment and can be edited for a different environment.

One-shot feeder run:

```bash
cargo run -p kzte-feeder -- --config config/feeder.toml --once
```

Long-running feeder service:

```bash
cargo run -p kzte-feeder -- --config config/feeder.toml
```

Initialize oracle config:

```bash
cargo run -p kzte-cli -- --config config/cli.toml init \
  --config-keypair ./target/oracle-config.json \
  --soft-stale-seconds 86400 \
  --hard-stale-seconds 259200 \
  --warn-deviation-bps 100 \
  --halt-deviation-bps 500 \
  --publishers <PUBLISHER_PUBKEY>
```

Create the canonical feeds:

```bash
cargo run -p kzte-cli -- --config config/cli.toml create-feed \
  --symbol KZTE/KZT \
  --base-symbol KZTE \
  --quote-symbol KZT \
  --is-reference-feed

cargo run -p kzte-cli -- --config config/cli.toml create-feed \
  --symbol KZTE/USD \
  --base-symbol KZTE \
  --quote-symbol USD \
  --is-reference-feed
```

Read a feed:

```bash
cargo run -p kzte-cli -- --config config/cli.toml read-feed --symbol KZTE/USD
```

Run the update path from CLI:

```bash
cargo run -p kzte-cli -- --config config/cli.toml update \
  --feeder-config config/feeder.toml
```

## Deployment Steps

1. Install Rust, Solana CLI, and Anchor CLI.
2. Generate a real deploy keypair for the program.
3. Replace the dev program id and run:

```bash
anchor keys sync
```

4. Update:
   - `Anchor.toml`
   - `.env`
   - `config/feeder.toml`
   - `config/cli.toml`
   - deployment automation / secret manager
5. Build and deploy the program:

```bash
anchor build
anchor deploy
```

6. Initialize oracle config with admin + publisher allowlist.
7. Create `KZTE/KZT` and `KZTE/USD` feeds.
8. Start the feeder service.
9. Optionally enable market TWAP sanity checks once a liquid KZTE market exists.

## Environment Variables

See `.env.example`. The important ones are:

- `SOLANA_RPC_URL`
- `SOLANA_WS_URL`
- `SOLANA_KEYPAIR_PATH`
- `KZTE_ORACLE_PROGRAM_ID`
- `KZTE_ORACLE_CONFIG_PUBKEY`
- `KZTE_ORACLE_PUBLISHER_SET_PUBKEY`
- `KZTE_FEED_KZTE_KZT_PUBKEY`
- `KZTE_FEED_KZTE_USD_PUBKEY`
- `KZTE_FEED_KZTE_USDC_PUBKEY`
- `KZTE_MINT`
- `NBK_OFFICIAL_PAGE_URL`
- `NBK_OPEN_DATA_*`
- `MARKET_TWAP_*`

## Update Flow

1. Feeder loads runtime config and current on-chain oracle state.
2. Feeder fetches the NBK official page.
3. Optional open-data source is fetched and included in the median if configured.
4. Official quotes are normalized to `USD/KZT @ 1e8`.
5. `KZTE/KZT` and `KZTE/USD` are derived deterministically, with the reciprocal path rounded to the nearest integer at `1e8` precision.
6. Optional market TWAP is fetched and converted into `peg_deviation_bps`.
7. Confidence is widened according to carry-forward / stale / divergence policy.
8. Feeder submits `submit_update` transactions.
9. On-chain program re-validates publisher, sequence, timestamps, config thresholds, and status.

## Threat Model

Covered risks:

- Unauthorized publisher spoofing
  - mitigated by on-chain signer allowlist
- Replay / duplicate updates
  - mitigated by strictly monotonic global and per-feed sequence checks
- Timestamp rollback
  - mitigated by monotonic publish-time validation
- Oracle drift from market noise
  - mitigated by keeping official NBK as primary truth and using market only for sanity status
- Silent stale data
  - mitigated by carry-forward + soft/hard stale policy
- Misconfigured admin actions
  - isolated into dedicated instructions
- Panic/unwrap in critical paths
  - avoided in on-chain business logic

Not fully covered:

- Compromised NBK primary source itself
- Admin private key compromise
- Malicious but allowed publisher signing valid-looking stale submissions faster than admin rotation
- A real DEX TWAP adapter for a future liquid market still needs venue-specific hardening before production activation

## Limitations

- The repository uses a verified **HTML page adapter** as the primary default because that source was explicitly confirmed.
- The NBK open-data path is intentionally generic and requires user-supplied JSON pointers.
- The market TWAP adapter is generic and feature-gated because no verified liquid KZTE venue was hardcoded here.
- The Dockerfile builds the off-chain binaries; it is not a full Anchor build image.
- The repository currently keeps a dev program id for local work; replace it before production deployment.

## Tests

Included tests cover:

- fixed-point math
- `USD/KZT -> KZTE/USD` derivation, including reciprocal rounding regression coverage
- NBK parser fixture
- carry-forward weekend logic
- soft/hard stale transitions
- deviation threshold classification
- unauthorized signer rejection
- replay rejection
- `solana-program-test` integration flow used by `anchor test`

## Switching Source Adapters

Default official page only:

```toml
[source.official_page]
enabled = true

[source.open_data]
enabled = false
```

Add open data:

```toml
[source.open_data]
enabled = true
url = "https://<verified-endpoint>"
rate_json_pointer = "/data/0/rate"
publish_time_json_pointer = "/data/0/date"
rate_convention = "kzt_per_usd"
```

Enable market sanity checks:

```bash
cargo run -p kzte-feeder --features market-twap -- --config config/feeder.toml
```

Then set:

```toml
[source.market_twap]
enabled = true
url = "https://<your-market-adapter-endpoint>"
price_json_pointer = "/twap_price"
publish_time_json_pointer = "/publish_time"
```
