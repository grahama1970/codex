# Comprehensive Review Request

- Fork: grahama1970/codex
- Branch: feat/chutes-profiles-scenarios
- Path: git@github.com:grahama1970/codex.git#feat/chutes-profiles-scenarios

Please perform a thorough review focused on correctness, safety, DX, and maintainability. Provide prioritized findings and proposed unified diffs. Where opinions differ, include trade‑offs and rationale.

## Context

This fork (“cxplus”) adds:
- Exec/interactive parity and reliability (artifacts, time budgets, deterministic seed plumbing)
- Chutes.ai integration: catalog discovery + exec fallback + warm‑up + capacity planning
- Post‑compile validation pipeline via Makefile (`make package`, `make test`, `make scenarios`, `make verify`)
- Scenario tests covering live behaviors where credentials exist; deterministic tests otherwise

We are now proposing a “Knowledge‑First” initiative (see `docs/feature_recipes/knowledge-first-context.md`) to build prompts from ArangoDB knowledge via memory‑agent MCP and keep in‑memory transcripts tiny.

## What to Review (by area)

1) Exec parity & reliability
   - Files: 
     - `codex-rs/exec/src/lib.rs`
     - `codex-rs/core/src/config.rs` (deterministic_seed, approval policy derivation)
     - `docs/exec.md`, `README.md` sections claiming parity
   - Ask: verify headless vs CLI defaults are equivalent in downstream behavior; ensure artifacts and timeout flows are durable and helpful.

2) Chutes discovery + exec + warm‑up
   - Files:
     - `codex-rs/cli/src/chutes_cmd.rs`
     - `scenarios/test_chutes_*`, `tests/test_codex_cli.py`
     - `docs/chutes.md`, `QUICKSTART.md`
   - Ask: correctness of filters (min/max params, capabilities, modalities), tie‑breaks (output price → params → context → input price); NaN price handling; base URL derivation sanitization; warm‑up dry‑run behavior without keys.

3) Makefile pipeline & DX
   - Files: `Makefile`, `README.md`, `QUICKSTART.md`, `FEATURES.md`
   - Ask: is the build→package→test→scenarios→release flow robust on clean machines; should `RUSTUP_TOOLCHAIN ?= 1.90.0` be set by default; any quoting/shell safety issues.

4) Deterministic seed propagation
   - Files: `codex-rs/core/src/client.rs`, `codex-rs/core/src/chat_completions.rs`, `codex-rs/core/src/config.rs`
   - Ask: confirm that when a seed is set, payloads include `seed`, `temperature=0.0`, `top_p=1.0`; propose tests if missing.

5) Scenarios & tests coverage
   - Files: `tests/`, `scenarios/`
   - Ask: identify gaps to cover end‑to‑end features (e.g., Arango/MCP fixtures once Knowledge‑First lands). Ensure failures surface actionable messages.

6) Logo/SVG changes (stability)
   - Files: `codex-rs/logo5.svg` and related README notes
   - Ask: confirm we ship a working animated SVG across engines or advise pinning to a static asset; recommend a process to avoid churn.

## Clarifying Questions

1) Should we set `RUSTUP_TOOLCHAIN ?= 1.90.0` in the Makefile to avoid E0658 regressions?
2) Should exec summary include `retry_attempts` and `last_http_status` for CI diagnostics?
3) For Knowledge‑First: do we prefer `arango` provider default for `exec` only, and keep TUI opt‑in initially?
4) Any objections to emitting `context.summary` telemetry per turn (tokens per section, selected node ids)?

## Expected Output

- A written review with:
  - High‑priority issues first (with rationale)
  - Medium/low‑priority items for polish
  - Concrete unified diffs for fixes and tests
  - Doc changes where claims are too strong or imprecise

## Reference Paths (quick map)

- Exec parity & summaries: `codex-rs/exec/src/lib.rs`
- Deterministic seed: `codex-rs/core/src/config.rs`, `codex-rs/core/src/client.rs`, `codex-rs/core/src/chat_completions.rs`
- Chutes: `codex-rs/cli/src/chutes_cmd.rs`
- Makefile pipeline: `Makefile`
- Tests: `tests/test_codex_cli.py`, `scenarios/`
- Knowledge‑First RFC: `docs/feature_recipes/knowledge-first-context.md`

## How to Reproduce

```
make package
make test
RUN_LIVE=1 make verify
# optional, with CHUTES_API_KEY in .env
make chutes-profiles
```

