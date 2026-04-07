# UI Design Prompt

Use the prompt below in a UI/design generation model. The model should invent the visual style itself, but must keep the product logic, information architecture, and interface intent exactly as described.

## Prompt

Design a modern web interface for a product called **KZTE Reference Oracle**.

This is **not** a trading terminal, not a DEX, not a generic crypto dashboard, and not a wallet-first experience.

The product is a **reference oracle for KZTE**, a **KZT-backed stablecoin on Solana**. The oracle treats **KZTE as a stable reference asset pegged to Kazakhstani tenge (KZT)**. The primary source of truth is the **official National Bank of Kazakhstan (NBK) exchange rate**, while market data is only an optional secondary sanity check for peg divergence.

The interface should communicate **trust, clarity, auditability, operational health, and deterministic logic**. It should feel like infrastructure for operators, integrators, and protocol teams, not like a speculative trading product.

### Product Truths

- `KZTE/KZT` is a reference feed and should read as `1.00000000`.
- `KZTE/USD` is derived from the official NBK `USD/KZT` rate using reciprocal conversion.
- The oracle has deterministic feed states:
  - `Active`
  - `CarryForward`
  - `Stale`
  - `Diverged`
  - `Halted`
  - `Paused`
- `CarryForward` matters because official business-day rates may not change on weekends or holidays.
- Confidence, publish time, observed time, sequence number, source count, and optional peg deviation are important pieces of information.
- This product has an off-chain feeder, an on-chain Solana program, and operator/developer tooling.

### High-Level UX Goal

Create a **clean, serious, high-signal oracle control and observability interface** that helps a user quickly answer:

1. What is the current reference state of KZTE?
2. Are the feeds healthy right now?
3. Is the oracle using a fresh NBK rate or carry-forward logic?
4. Is there any peg divergence warning or halt condition?
5. What are the important config values, feed accounts, and developer integration details?

### Information Architecture

Design the product as a multi-page web app with these core views:

#### 1. Overview / Home

The landing page should immediately show:

- Product identity: `KZTE Reference Oracle`
- Short explanation that this is a **KZT-backed stablecoin reference oracle on Solana**
- Primary live cards for:
  - `KZTE/KZT`
  - `KZTE/USD`
- For each feed, show:
  - current price
  - current status
  - confidence
  - publish time
  - observed time
  - sequence
- A clearly visible oracle health summary
- A short explanation of why this oracle is different from market-priced crypto feeds

The homepage should feel like an operator's summary screen, not a marketing hero for retail users.

#### 2. Feed Details

Create a dedicated page or section for detailed feed inspection.

For each feed, expose:

- feed symbol
- current price
- confidence
- expo / precision
- status
- publish time
- previous publish time
- observed time
- source count
- last good price
- sequence
- metadata version
- raw payload hash
- optional peg deviation bps
- optional market TWAP comparison if present
- Solana feed account address

The feed detail layout should make it easy to inspect state transitions and judge data quality.

#### 3. Oracle Health / Operations

Create a monitoring-oriented page for runtime health.

Include panels for:

- feeder cycle health
- failure count
- skipped submissions
- last submission sequence
- last submitted `KZTE/USD` value
- metrics endpoint status
- source adapter status
- whether official NBK source fetch succeeded
- whether the system is in carry-forward mode
- whether market sanity check is enabled or disabled

This page should look like serious infrastructure monitoring, but still product-polished.

#### 4. Methodology / How It Works

Create an explanatory page that makes the oracle understandable to technical and semi-technical users.

Explain visually:

- NBK official source input
- normalization to fixed-point precision
- derivation of `KZTE/KZT`
- derivation of `KZTE/USD`
- optional market TWAP sanity check
- confidence widening rules
- carry-forward logic on weekends / holidays
- state transitions:
  - `Active`
  - `CarryForward`
  - `Stale`
  - `Diverged`
  - `Halted`

This page should prioritize clarity over hype. Use diagrams, process sections, or visual logic blocks rather than dense prose alone.

#### 5. Developers

Create a developer-facing page that feels credible to protocol engineers.

Include space for:

- program ID
- config account address
- publisher set address
- feed addresses
- examples of data fields returned by the oracle
- integration notes
- CLI / API / account-read examples

This page should feel practical and implementation-focused.

### Core Components To Design

Design components such as:

- feed status badges with clear semantics
- structured metric cards
- timeline or freshness indicators
- state transition explainer
- feed inspection table
- configuration summary blocks
- account/address cards
- operational alerts / warning banners
- methodology diagram section

### Content Priorities

The interface should emphasize:

- correctness
- freshness
- determinism
- provenance
- status clarity
- operational visibility

The interface should de-emphasize:

- speculative price excitement
- trading behavior
- wallet actions
- token hype
- flashy "number go up" storytelling

### Strong Anti-Requirements

Do **not** design this like:

- a DEX
- a meme coin landing page
- a wallet onboarding flow
- a candlestick trading terminal
- a generic crypto portfolio dashboard
- a "high-frequency market data" hero product
- a Pyth / SOL / BTC / ETH market feed product

Do **not** center the interface around:

- `Connect Wallet`
- swap actions
- order books
- candlesticks
- trading pairs unrelated to KZTE
- speculative charts pretending this is a volatile asset

### Data and State Behavior To Reflect In The UI

The design should clearly support these situations:

- Normal fresh update from official NBK source
- Weekend/holiday carry-forward state
- Soft aging of data
- Hard stale state
- Divergence warning from optional market sanity check
- Halt condition
- Paused oracle state

The status system must feel operationally meaningful, not decorative.

### Tone

The product tone should be:

- authoritative
- calm
- technical
- transparent
- infrastructure-grade
- audit-friendly

It should not feel:

- noisy
- retail
- speculative
- gimmicky
- trend-chasing

### Visual Freedom

You are free to invent the full visual system, including:

- typography
- color palette
- spacing
- card styles
- diagrams
- page composition
- iconography
- motion

But the design must still feel appropriate for a **reference oracle and monitoring interface**, not for a consumer trading app.

### Deliverable

Produce a complete interface concept for desktop and mobile with:

- homepage
- feed details view
- operations / health view
- methodology view
- developer view

Show realistic UI blocks, labels, states, and information hierarchy for this exact product.

If example data is needed, use realistic placeholders for:

- `KZTE/KZT = 1.00000000`
- `KZTE/USD` as a small decimal derived from reciprocal conversion
- statuses such as `CarryForward` or `Active`
- timestamps, sequence numbers, confidence values, and feed/account identifiers

The interface should make a user trust the oracle because it is legible, inspectable, and operationally honest.

## Short Version

If you need a shorter prompt, use this:

Design a polished multi-page web UI for **KZTE Reference Oracle**, a **KZT-backed stablecoin oracle on Solana** that uses the **official National Bank of Kazakhstan exchange rate** as primary truth. This is an **infrastructure and observability product**, not a trading app. The UI must focus on `KZTE/KZT`, `KZTE/USD`, feed status (`Active`, `CarryForward`, `Stale`, `Diverged`, `Halted`, `Paused`), confidence, publish/observed times, sequence, peg divergence, feeder health, config/accounts, methodology, and developer integration. Avoid DEX patterns, wallet-first UX, candlestick charts, hype, and generic crypto dashboard tropes. Make it feel trustworthy, deterministic, technical, calm, and audit-friendly.
