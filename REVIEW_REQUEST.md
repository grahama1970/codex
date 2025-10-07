# Comprehensive Review Request (Expanded)

- Fork: grahama1970/codex
- Branch: feat/chutes-profiles-scenarios
- Path: git@github.com:grahama1970/codex.git#feat/chutes-profiles-scenarios

Please perform a deep, end‑to‑end review oriented around reliability, safety, and developer experience. Provide prioritized findings, rationale, and concrete unified diffs. When trade‑offs exist, outline options and recommend one.

**What’s in this fork (“cxplus”)**
- Exec/interactive parity + reliability: consistent behavior, artifacts, time budgets, deterministic seed plumbing.
- Chutes.ai integration: catalog discovery → model selection → exec; warm‑up subcommand; capacity planning helper.
- Post‑compile validation pipeline (Makefile) + deterministic tests + live scenarios.
- “Knowledge‑First” proposal to make context come from ArangoDB (memory‑agent MCP) instead of a large rolling transcript (see docs/feature_recipes/knowledge-first-context.md).

**Reviewer Deliverables**
- High‑priority issues (blocking/major) with clear rationale.
- Actionable unified diffs (patches) for fixes and tests.
- Medium/low‑priority polish items (brief but concrete).
- Doc diffs where claims are inaccurate or need hardening.

**Environment & Repro**
- Toolchain: Rust 1.90.0 recommended for builds.
- One‑liners:
  - `make package` → compile + dist
  - `make test` → deterministic tests (offline) against compiled binary
  - `RUN_LIVE=1 make verify` → runs both deterministic and live scenarios
  - `make chutes-profiles` → auto‑discover coding + multimodal models (requires CHUTES_API_KEY in `.env`)

--------------------------------------------------------------------------------

## A. Exec Parity & Reliability (Headless vs Interactive)

Scope
- Ensure `codex exec` runs are indistinguishable downstream from interactive sessions where it matters: attribution/telemetry, approvals, artifacts, and time‑budget handling.

Relevant Files
- `codex-rs/exec/src/lib.rs` (session source, summary writing, timeouts, slash wiring)
- `codex-rs/core/src/config.rs` (approval policy derivation, deterministic_seed)
- `docs/exec.md`, `README.md` (claims of parity/reliability)

What To Validate
- Artifacts: events NDJSON and summary JSON always written; exit codes and timeout behavior are consistent.
- Approvals/sandbox: defaults make sense and respect config overrides; no silent surprises for CI users.
- Determinism switch: seed flows into model payloads (temperature=0, top_p=1 where supported).

Requested Output
- If gaps exist, provide diffs to: (1) add missing summary fields (`retry_attempts`, `last_http_status`), (2) add unit tests asserting summary schema and seed propagation, (3) correct any approval/sandbox inconsistencies.

--------------------------------------------------------------------------------

## B. Chutes Discovery, Exec, Warm‑Up, Planning

Scope
- End‑to‑end correctness of discovery filters, tie‑breaks, network robustness, and warm‑up behavior with/without API keys.

Relevant Files
- `codex-rs/cli/src/chutes_cmd.rs` (Recommend/Exec/Warmup/Plan impl)
- `docs/chutes.md`, `QUICKSTART.md` (usage + troubleshooting)
- Scenarios and tests:
  - `scenarios/test_chutes_cli_subcommand.py`
  - `scenarios/test_chutes_autodiscover_exec.py`
  - `scenarios/test_chutes_warmup_dryrun.py`
  - `scenarios/test_chutes_price_relaxation.py`
  - `scenarios/test_chutes_price_partial_nan.py`
  - `scenarios/test_chutes_profiles.py`
  - `tests/test_codex_cli.py` (CLI basics)

Review Focus
- Filtering & ranking: min/max params, capabilities, modalities; tie‑breakers (output price → params → context → input price); NaN price handling (cap vs unpriced).
- Base URL derivation: sanitization of owner/slug/domain; precedence of `CHUTES_API_BASE`.
- Warm‑up UX: dry‑run without keys; errors and exit codes; bounded retries and timeouts.
- Planning: inputs/outputs and cost CPM math sanity.

Requested Output
- Diffs to improve: additional tests (offline fixture for selection ordering), error messages, or boundary conditions (e.g., price‑only NaN fallback). Call out any unnecessary unwraps.

--------------------------------------------------------------------------------

## C. Build → Package → Test → Scenarios → Release (DX)

Scope
- Single‑command confidence before shipping binaries; portability and quoting/robustness.

Relevant Files
- `Makefile`, `README.md`, `QUICKSTART.md`, `FEATURES.md`

Review Focus
- Default toolchain: recommend adding `RUSTUP_TOOLCHAIN ?= 1.90.0` at top to avoid E0658 for non‑MSRV hosts?
- Shell safety: `set -euo pipefail` and quoting in model discovery writes and release flow; Windows zip path handling.
- “Install‑local” link safety; rollback/switch logic.

Requested Output
- Diffs to harden quoting, set default toolchain, and add a `make doctor` section that exercises minimal live commands.

--------------------------------------------------------------------------------

## D. Deterministic Seed Plumbing

Relevant Files
- `codex-rs/core/src/config.rs` (field + overrides)
- `codex-rs/core/src/client.rs`, `codex-rs/core/src/chat_completions.rs` (payload)

Review Focus
- When seed is set: payload contains `seed`, `temperature=0.0`, `top_p=1.0` for responses/chat APIs (where applicable); no unintended side effects.

Requested Output
- Diffs for unit test that captures serialized payload content with seed present/absent; update docs if behavior should be constrained to specific providers.

--------------------------------------------------------------------------------

## E. Scenarios & Tests: Coverage and Gaps

What Exists
- Deterministic tests: `tests/test_codex_cli.py` (help/version/completions/login‑status/exec guards)
- Live scenarios (opt‑in): warm‑up dry‑run; price relaxation/no‑relax; CLI subcommand flow; profiles write; proxy/MCP smoke.

Gaps To Evaluate
- Add offline fixture tests for discovery ordering and NaN price behaviors.
- Add an exec summary schema test.
- (Later) Add Knowledge‑First scenario (see below).

Requested Output
- Diffs to add missing tests + any fixes revealed.

--------------------------------------------------------------------------------

## F. Knowledge‑First Context (RFC for Review)

Reference
- `docs/feature_recipes/knowledge-first-context.md`

Ask
- Validate the architecture (ContextProvider, TokenBudgeter, MCP queries) and the config surface. Suggest minimal invasive places to wire it into `codex-core` prompt assembly.
- Recommend metrics and tests to prove token savings and correctness.

Requested Output
- Diffs for an initial crate scaffold (`codex-rs/context`), config parsing, and a stub provider wired behind a feature flag/profile; one deterministic test and one scenario skeleton.

--------------------------------------------------------------------------------

## G. Assets: SVG/Brand Stability

Scope
- Ensure shipped `logo5.svg` is safe/stable across major engines or advise pinning to static asset by default.

Relevant Files
- `codex-rs/logo5.svg`, notes in `codex-rs/README.md`

Requested Output
- Recommendation and (if needed) diffs to use the static version by default in docs/banners while keeping animated for demo pages.

--------------------------------------------------------------------------------

## Clarifying Questions

1) Should we set `RUSTUP_TOOLCHAIN ?= 1.90.0` in the Makefile to reduce contributor friction?
2) Should exec summary include `retry_attempts` and `last_http_status` for CI flakiness triage?
3) Knowledge‑First default rollout: `exec` first, TUI opt‑in later — agree?
4) Can we emit `context.summary` events (counts/tokens/ids) for observability?
5) Any provider constraints on `seed` we should document (e.g., ignored by some backends)?

--------------------------------------------------------------------------------

## How To Run Locally

```
make package
make test
RUN_LIVE=1 make verify

# With Chutes key in .env
make chutes-profiles
```

If you want a minimal “doctor” flow, recommend adding a target; feel free to include diffs.

