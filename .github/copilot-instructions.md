# Copilot Code Review Instructions (Repository‑Scoped)

Fork: grahama1970/codex

Branch: feat/chutes-profiles-scenarios

Path (SSH): git@github.com:grahama1970/codex.git#feat/chutes-profiles-scenarios

URL (HTTPS): https://github.com/grahama1970/codex/tree/feat/chutes-profiles-scenarios

## Overall Context

This fork ("cxplus") extends OpenAI’s Codex CLI with:
- Chutes.ai model auto‑discovery and execution (recommend/exec/warmup)
- Deterministic post‑compile verification: Makefile pipeline, deterministic tests, live scenarios
- An iterative Knowledge‑First context system (context from evidence/tools rather than long transcripts)

Please focus on reliability, safety, determinism, and developer experience. Prefer minimal, surgical changes and concrete unified diffs.

## Live Scenarios (compiled binary, validate “real” behavior)

Relative paths under `scenarios/`:
- test_chutes_cli_subcommand.py
- test_chutes_exec.py
- test_chutes_price_relaxation.py
- test_chutes_price_partial_nan.py
- test_chutes_profiles.py
- test_chutes_warmup_dryrun.py
- test_login_roundtrip.py
- test_context_budget.py
- scillm_router_parallel_smoke.py

References: README.md, FEATURES.md, CONTEXT.md (context.summary v2 expectations); docs/chutes.md.

## Key Code Paths (relative)

- Core/exec/events & prompt plumbing:
  - codex-rs/exec/src/lib.rs
  - codex-rs/protocol/src/protocol.rs
  - codex-rs/core (prompt assembly, config)
- Chutes integration (CLI + exec + TUI):
  - codex-rs/cli/src/chutes_cmd.rs
  - codex-rs/exec/src/lib.rs
  - codex-rs/tui/src/chatwidget.rs
  - codex-rs/tui/src/slash_command.rs
  - codex-rs/tui/src/history_cell.rs
  - docs/chutes.md
- Knowledge‑First context crate:
  - codex-rs/context/src/lib.rs
  - codex-rs/context/src/retrieval.rs
  - codex-rs/docs/src/main.rs

## Constraints & Conventions

- Keep CLI UX and Makefile interface stable.
- Rust style:
  - Collapse nested `if` statements.
  - Inline `format!` args when possible.
  - Prefer method references over redundant closures.
  - Tests: prefer whole‑object equality; use `pretty_assertions::assert_eq`.
- TUI (ratatui): prefer `Stylize` helpers (e.g., `"text".dim().red()`). Avoid hardcoded white.

## What to Deliver

1) Answers to clarifying questions (below).
2) Prioritized findings (Blocking → Medium → Low) with rationale.
3) Concrete unified diffs grouped by concern (exec/events; chutes; context wiring; docs/tests). Keep changes minimal and focused; include relative file paths.

## Clarifying Questions

1) Where should prompt assembly hook(s) live today to integrate `codex-context` cleanly (function names and files)?
2) Propose a minimal, forward‑compatible `context.summary` event shape (tokens per section, retrieval_ms, evidence_items).
3) For Chutes discovery: confirm behavior and the user‑visible notice when price caps relax due to NaN output prices; where to surface (stderr line + summary field)?
4) Any conflicts with deterministic builds or event schemas to address before rollout?

## Repro Commands

- `make package` — compile + dist
- `make test` — deterministic tests (offline)
- `RUN_LIVE=1 make verify` — live scenarios

## Output Format

- Provide unified diffs only, grouped by concern. Keep patches minimal and readable. Include any new tests and docs updates needed for acceptance.

