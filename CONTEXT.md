# cxplus — Work Session Context (Oct 7, 2025)

This is the runbook to resume instantly tomorrow.

- Branch: `feat/chutes-profiles-scenarios`
- Scope today: Knowledge‑First plumbing + retrieval hardening + docs/branding refresh.

## What Landed (today)

- Single, completed `context.summary` v2 emission (pre‑stream) in `codex-exec` with real metrics gathered from the Knowledge‑First provider.
- Core client now exposes a helper that builds Knowledge‑First context and returns metrics when desired.
- Context provider (Arango) now uses the current Tokio runtime handle instead of creating a nested runtime.
- Retrieval layer implements a proper 429 retry path with a short backoff under the remaining deadline.
- Scenario test updated to assert v2 semantics. Docs adjusted accordingly.

## Touched Files

- `codex-rs/core/src/client.rs:582` — return metrics from knowledge‑first build; added `prepare_prompt_with_context`.
- `codex-rs/context/src/lib.rs:201` — use `tokio::runtime::Handle::current().block_on`.
- `codex-rs/context/src/retrieval.rs` — 429 retry + small refactor around timeout handling.
- `codex-rs/exec/src/lib.rs` — emit a single `context.summary` line with metrics before streaming.
- `codex-rs/exec/Cargo.toml` and `codex-rs/Cargo.toml` — added `codex-context` dependency for `codex-exec`.
- `scenarios/test_context_budget.py` — asserts v2 record and (when fixture+Arango) non‑zero `evidence_items`.
- `README.md`, `FEATURES.md` — wording changed to reflect single (completed) line semantics.

## How to Build & Test (quick)

- Format after code changes (required by AGENTS.md):
  - `cd codex-rs && just fmt`
- Lints (scoped to changed crates):
  - `just fix -p codex-exec`
  - `just fix -p codex-context`
  - Note: `just fix -p codex-core` currently compiles but core tests include struct‑literal expectations that may need adjustments if we add new config fields in the future. We did not modify those tests today.
- Run unit/integration tests for what changed:
  - `cargo test -p codex-exec`
  - `cargo test -p codex-context`

## Scenario (events.ndjson contains context.summary v2)

- Build a binary first (repo root): `make package` (produces `dist/bin/codex`).
- Run the scenario (repo root):
  - Offline/minimal verification:
    - `CONTEXT_FEATURE=1 dist/bin/codex exec "Summarize this repository"`
  - With fixture + Arango (offline, deterministic):
    - `export CONTEXT_FEATURE=1`
    - `export CONTEXT_MCP_FIXTURE=codex-rs/context/tests/fixtures/mcp_fixture.json` (add a fixture file before running)
    - `dist/bin/codex exec "Summarize this repository"`
- Inspect `.codex/runs/*-events.ndjson` and verify a single `{"kind":"context.summary","version":2,...}` line appears early with non‑zero metrics when fixture+Arango are used.

## Configuration knobs (recap)

- `[context]` in `~/.codex/config.toml`:
  - `provider = "arango" | "minimal"`
  - `max_context_tokens = 8192` (example)
  - `[context.budget] recent_pct, plan_pct, evidence_pct, tools_pct`
  - `[context.arango] endpoint, database, mcp_tool, search_k, neighbors_depth, timeout_ms, max_evidence_items`
- Env during runs:
  - `CONTEXT_DEBUG=1` — extra stderr diagnostics in context/retrieval
  - `CONTEXT_MCP_FIXTURE=/path/to/fixture.json` — deterministic offline retrieval
  - `CONTEXT_EVIDENCE_ALLOW_CODE=1` — allow code blocks in evidence shaping (default off)

## Next Steps (execute tomorrow)

P0 – Wire hooks + add scenarios
- Prehook in `exec` (before first submit): use MCP preset (agent‑memory) to Augment prompt; handle Allow/Deny/Ask/Patch/Augment/RateLimit; emit `prehook_result` (NDJSON + OTEL).
- Posthook pipeline: generic script posthook; keep Slack notifier compatibility; emit `posthook_result`.
- Scenarios (compiled binary):
  - `scenarios/prehook_augment_smoke.py` — asserts augmentation + event present.
  - `scenarios/posthook_notifier_smoke.py` — asserts posthook executed.
  - `scenarios/agent_comms_smoke.py` — two agents exchange 3–5 messages; assert <100ms local latency + NDJSON evidence.

P1 – Reliability & docs (2–3 days)
- Generate `docs/generated/events/context-summary-v2.json` + index.
- Chutes offline unit tests with fixtures (filters, NaN price caps, tie‑breaks, base URL).
- Scenarios: Chutes exec (fixture), warmup delta capture, timeout/run_timeout marker, image input path.
- Docs: `docs/agent-comms.md`; expand `docs/advanced.md` with pre/post‑hook usage.

## Quick Resume Checklist
1) `git switch feat/chutes-profiles-scenarios`
2) `make package`
3) Run new scenarios after P0 lands; today: verify context line quickly → `CONTEXT_FEATURE=1 dist/bin/codex exec "hello"` and check the newest `.codex/runs/*-events.ndjson` for `{"kind":"context.summary","version":2,...}`.

---
This CONTEXT.md summarizes only today’s Knowledge‑First/metrics work stream. The Chutes auto‑discovery and profiles integration work remains in this branch and can be iterated separately.
