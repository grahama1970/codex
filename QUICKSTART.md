# Quickstart

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="./codex-rs/logo-dark-centered.svg" />
    <source media="(prefers-color-scheme: light)" srcset="./codex-rs/logo-light-centered.svg" />
    <img src="./codex-rs/logo-light-centered.svg" alt="cxplus logo" width="480" />
  </picture>
</p>

> Experimental fork disclaimer
>
> This repository is an experimental, personal fork ("cxplus playground"). It is not intended to be merged upstream into OpenAI’s Codex, and it has no official support. See FORK_POLICY.md for details.

This fork ("cxplus") provides a compiled, versioned CLI with Chutes auto‑discovery (fixture mode), warm‑up, slash command QOL, capacity planning, and a frictionless deploy/switch/rollback flow. This guide is the fastest way for any project agent (human or automated) to build, test, deploy, and use cxplus.

## 0) Prereqs
- Rust toolchain (we test with `RUSTUP_TOOLCHAIN=1.90.0`)
- Optional helpers: `just`, `rg`, `cargo-insta` (`make rust-prepare` will suggest/install)

## 1) Build and package
```
make package
```
Outputs:
- `dist/bin/codex` (canonical binary)
- `dist/config/config.toml` (minimal config for tests)
 - `dist/bin/cxplus` (alias pointing to `codex`)

## 2) Run deterministic tests (offline)
```
RUSTUP_TOOLCHAIN=1.90.0 make test
```

## 3) Run live scenarios (post‑compile)
```
RUSTUP_TOOLCHAIN=1.90.0 make scenarios
```
Notes:
- Live scenarios use `.env` if present. For Chutes discovery, set `CHUTES_API_KEY` and optionally `CHUTES_API_BASE`.
- A deterministic fixture mode exists for discovery: set `CHUTES_CATALOG_FIXTURE=/absolute/path.json`.

## 3.1) Scenario tips
- Price relaxation: fixture with all NaN prices prints `[chutes-relax] …` once.
- Partial NaN: at least one valid priced candidate prevents relaxation.
- Warm‑up dry run: works without keys (`--dry-run` or `CHUTES_WARMUP_DRYRUN=1`).

Note: A Knowledge‑First context mode (RFC) is being prepared to reduce prompt size by retrieving evidence from ArangoDB via the memory‑agent MCP. This does not affect the quickstart flow. See `docs/feature_recipes/knowledge-first-context.md` for the design.

## 4) Rapid deploy & versioning
Create a stamped release and update the active binary + alias:
```
make release
```
Artifacts:
- `dist/releases/codex-<YYYYMMDDHHMM>-<branch>-<sha>`
- `dist/bin/codex` → symlink to the stamped binary
- `dist/bin/cxplus` → symlink to `codex`
 - `dist/release.json` → `{ "stamp": "…", "binary": "…" }` (TUI `/status` shows the stamp)

Switch / rollback:
```
make list-releases
make switch VERSION=<stamp>
make rollback
```

## 5) Install a user‑level alias (safe)
```
make install-local   # creates ~/.local/bin/cxplus -> dist/bin/cxplus
```
Then add to your shell:
```
alias cx=cxplus
```

## 6) Core commands (cheat sheet)
- Discovery (CLI):
  - `dist/bin/cxplus chutes recommend`  # prints `openai/<id>`
  - Optional: `--show-base` to print derived base_url
  - Fixture mode: `CHUTES_CATALOG_FIXTURE=/abs/path.json`
- Discovery (TUI): `/discover`
  - Read‑only info cell; set `APPLY_DISCOVER_AUTO=1` to apply for the session
- Warm‑up:
  - CLI: `dist/bin/cxplus chutes warmup --secs 4 [--dry-run]`  (lines prefixed `[chutes-warmup]`)
  - TUI/Exec slash: `/warmup [secs]`
- Capacity planning (like chutes_planner.py):
  - `dist/bin/cxplus chutes plan --requests 10000 --avg-input-tokens 300 --avg-output-tokens 200 --deadline-hours 2 --gpu-type A5000 --hourly-rate-usd 1.30 --save`
  - Env overrides: `CHUTES_PERF_JSON`, `CHUTES_RATES_JSON`, `CHUTES_GPU_TYPE`, `CHUTES_HOURLY_RATE`

## 7) Chutes quick check
```
export CHUTES_API_KEY=...            # or set via .env
dist/bin/cxplus chutes recommend     # prints openai/<model-id>
dist/bin/cxplus chutes exec --json "Say hello"
```

## 8) Windows packaging
```
make package-windows   # writes dist/cxplus-windows.zip (cxplus.cmd/.ps1 + codex/codex.exe if present)
```
Place cxplus.cmd or cxplus.ps1 on PATH and invoke `cxplus …`.

## 9) Slash command QOL (exec)
- `/grep` truncation marker (env `GREP_MAX_LINES`)
- `/open` size guard (env `OPEN_MAX_KB`)
- Write‑enabled notice when `ENABLE_SLASH_WRITE=1` for `/fmt`, `/build`, `/test`

## 10) Troubleshooting
- “No suitable model” → try lowering `--min-params`, relaxing `--max-output-ppm`, removing `--require-capabilities`, or adjusting `--require-modalities`.
- Fixture mode set but no output → confirm file path and JSON shape (top‑level `items`).
- Warm‑up network failures → use `--dry-run` first, then set `CHUTES_API_KEY` and `CHUTES_API_BASE`.

## 11) Helpful envs (quick list)
- Discovery: `CHUTES_API_KEY`, `CHUTES_API_BASE`, `CHUTES_CATALOG_FIXTURE`, `CHUTES_DISCOVERY_DEBUG`, `CHUTES_FORCE_PROVIDER_BASE`, `CHUTES_EXTRA_CAPS`
- Warm‑up: `CHUTES_WARMUP_DRYRUN`, `CHUTES_WARMUP_SECS`
- Slash QOL: `GREP_MAX_LINES`, `OPEN_MAX_KB`, `ENABLE_SLASH_WRITE`
- Slack notifications: set `SLACK_WEBHOOK_URL` and in `~/.codex/config.toml` add `notify = ["codex-notify-slack"]`

See also: docs/chutes.md (discovery + troubleshooting) and docs/slash-commands.md (slash behaviors).

## 12) Optional TUI theming
You can customize the cxplus banner/accent colors via `~/.codex/config.toml`.

```
[tui.brand]
title_color = "magenta"     # also supports: red, green, yellow, blue, cyan, gray, white, or hex like "#A855F7"
accent_color = "magenta"    # used for small labels (e.g., "model changed:")
```

If unset, cxplus defaults to magenta.

## 13) Slack notifications (built‑in)
To receive a Slack message when a turn completes, set a webhook and enable the notifier:

```
# ~/.codex/config.toml
notify = ["codex-notify-slack"]

# Shell env
export SLACK_WEBHOOK_URL="https://hooks.slack.com/services/…"
```

Optionally, pass an explicit URL: `notify = ["codex-notify-slack", "--webhook", "https://hooks…"]`.

## 14) Docs (auto‑generated)

- Reference is generated under `docs/generated/`.
- Regenerate locally: `make docs-gen` (or `make docs-fix` to generate + stage changes).
- CI checks for drift on PRs and `main` via `make docs-drift`.
