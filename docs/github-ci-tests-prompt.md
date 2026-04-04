# GitHub CI Test Generation Prompt

Use the prompt below to ask another model to generate CI tests and GitHub Actions workflows for this repository.

```text
Role
You are a senior DevOps and test automation engineer. You specialize in GitHub Actions, Rust workspaces, Solana/Anchor projects, and Next.js monorepos. You write production-grade CI that is deterministic, cache-aware, and easy to maintain.

Task
Create a CI solution for this repository using GitHub Actions. Your output should include the workflow file(s), any small supporting script changes that are truly necessary, and a short explanation of why each job exists.

The CI should:
- run on pull requests and pushes to the main development branch;
- validate the Rust workspace;
- run the existing Rust tests;
- cover the Solana/Anchor integration-test path in a practical way;
- validate the frontend build;
- avoid depending on production secrets or external NBK endpoints for required checks;
- use caching for Rust and Node dependencies;
- keep the default PR pipeline reasonably fast.

Prefer the smallest reliable solution, but do not skip important validation. If the Anchor toolchain setup is too heavy for the default workflow, separate it into a dedicated job and explain the tradeoff instead of silently omitting it.

Deliverables:
- a primary workflow such as `.github/workflows/ci.yml`;
- any additional workflow only if justified;
- minimal repo changes required to make CI pass consistently;
- a concise summary of assumptions and setup choices.

Context
Repository name: `kzte-reference-oracle`

Current repository structure:
- Rust workspace at the repo root with members:
  - `crates/common`
  - `crates/feeder`
  - `crates/cli`
  - `programs/kzte_oracle`
  - `tests`
- Frontend app in `frontend/` using Next.js.

Observed tech stack and commands:
- Root `Cargo.toml` defines a Rust workspace using edition 2021.
- Rust dependencies include Anchor `0.31.1`, Solana `2.1.11`, Tokio, Reqwest, Axum, and `solana-program-test`.
- `Anchor.toml` sets `anchor_version = "0.31.1"` and defines:
  - provider cluster: `Localnet`
  - test script: `cargo test -p integration-tests -- --nocapture`
- `tests/src/integration.rs` uses `solana-program-test`, which suggests integration tests can run locally without a real external RPC.
- `scripts/anchor-test.ps1` currently runs `anchor test`.
- README documents these common commands:
  - `cargo build`
  - `anchor build`
  - `cargo test`
  - `anchor test`
- README also says the current tests cover:
  - fixed-point math
  - `USD/KZT -> KZTE/USD` derivation, including reciprocal rounding regression coverage
  - NBK parser fixture
  - carry-forward weekend logic
  - soft/hard stale transitions
  - deviation threshold classification
  - unauthorized signer rejection
  - replay rejection
- The repo currently has no `.github/workflows/` directory.

Frontend facts:
- `frontend/package.json` has these scripts:
  - `dev`
  - `build`
  - `start`
- The frontend uses:
  - `next@16.2.2`
  - `react@19.2.4`
  - `react-dom@19.2.4`
  - `typescript@5.9.3`
- A local `npm run build` in `frontend/` succeeds.

Constraints and preferences:
- Use GitHub Actions.
- Assume Linux runners unless a different OS is strictly necessary.
- Do not require production secrets for mandatory jobs.
- Do not invent nonexistent test commands.
- If the frontend lacks dedicated test or lint scripts, use the existing build as the required validation, and only add extra scripts if there is a clear benefit.
- Keep the workflow explicit and readable rather than overly clever.
- Prefer path-aware or job-level efficiency if it improves runtime without making maintenance confusing.
- Use dependency caching for Cargo and npm.
- Call out any part that may be flaky because of Anchor or Solana installation, and make a sensible recommendation.

What I want from you
1. Propose the final CI design.
2. Generate the workflow YAML.
3. Include any minimal supporting file edits needed for CI stability.
4. Explain how the pipeline maps to the current repository layout and test surface.
5. Return ready-to-commit file contents, not placeholders.
```
