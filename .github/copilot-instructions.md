# Copilot Code Review Instructions

## Context
- This repository contains the cxplus Codex CLI fork and related crates under `codex-rs/`.
- Key areas: Chutes model auto‑discovery/exec, Knowledge‑First context provider, deterministic tests and live scenarios.

## Review Focus
- Reliability, safety, determinism, and developer experience.
- Prefer minimal, surgical diffs; preserve CLI UX and Makefile targets.

## Key Paths (relative)
- codex-rs/exec/src/lib.rs (eventing, prompt/exec plumbing)
- codex-rs/core (prompt assembly, config)
- codex-rs/cli/src/chutes_cmd.rs (Chutes CLI)
- codex-rs/tui/* (TUI UX; prefer Stylize helpers)
- codex-rs/context/* (Knowledge‑First providers and budgeting)
- scenarios/* (post‑compile live tests)
- tests/* (deterministic tests)

## Constraints & Conventions
- Rust style: collapse nested `if`s; inline `format!` args; prefer method refs over redundant closures.
- Tests: compare whole objects; use `pretty_assertions::assert_eq`.
- TUI: prefer `"text".dim().red()` over manual `Span::styled` where possible; avoid hardcoded white.

## What To Deliver
- Clarifying answers (if applicable).
- Prioritized findings (Blocking → Medium → Low) with rationale.
- Concrete unified diffs grouped by concern (exec/events; chutes; context wiring; docs/tests). Keep changes minimal and readable.

## Repro Commands
- `make package` — compile + dist
- `make test` — deterministic tests
- `RUN_LIVE=1 make verify` — live scenarios
