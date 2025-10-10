# Quickstart

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="./codex-rs/logo-dark-centered.svg" />
    <source media="(prefers-color-scheme: light)" srcset="./codex-rs/logo-light-centered.svg" />
    <img src="./codex-rs/logo-light-centered.svg" alt="cxplus logo" width="480" style="display:block;margin:0 auto;" />
  </picture>
</p>

> Experimental fork disclaimer
>
> This repository is an experimental, personal fork ("cxplus playground"). It is not intended to be merged upstream into OpenAI’s Codex, and it has no official support. See `FORK_POLICY.md` for details.

This fork ("cxplus") provides a compiled, versioned CLI with Chutes auto‑discovery (fixture mode), warm‑up, slash command QOL, capacity planning, and a frictionless deploy/switch/rollback flow. This guide is the fastest way for any project agent (human or automated) to build, test, deploy, and use cxplus.

> Note: Auth, MCP, and general CLI usage follow upstream Codex docs. This fork adds packaging/switch/rollback, pre/post hooks, Chutes discovery, and knowledge‑first options.

### Security & Privacy

Telemetry is off by default. OpenTelemetry export is opt‑in; artifacts remain local in `./.codex/runs/`.

## 0) Prereqs
- Rust toolchain (we test with `RUSTUP_TOOLCHAIN=1.90.0`)
- Optional helpers: `just`, `rg`, `cargo-insta` (`make rust-prepare` will suggest/install)

## 1) Build and package
```bash
make package
```
Outputs:
- `dist/bin/codex` (canonical binary)
- `dist/config/config.toml` (minimal config for tests)
 - `dist/bin/cxplus` (alias pointing to `codex`)

## 2) Run deterministic tests (offline)
```bash
RUSTUP_TOOLCHAIN=1.90.0 make test
```

## 3) Run live scenarios (post‑compile)
```bash
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


## 3.2) Local‑Only (ITAR) quick start (no egress)

Strict no‑egress posture with a minimal config:

```toml
# ~/.codex/config.toml
local_only = true

[tools]
web_search = false  # redundant when local_only = true
```

Or one‑shot override for a single run:

```bash
codex -c local_only=true exec "hello" --seed 42
```

What this enforces:
- Blocks non‑local model providers (only localhost/127.0.0.1/[::1] allowed)
- Bypasses HTTP(S)_PROXY/ALL_PROXY (internal `CODEX_LOCAL_ONLY=1`)
- Disables web search, RMCP (HTTP), OTEL exporter, notifier hooks
- Blocks login commands (device code / ChatGPT / API key)

Verify:

```bash
printf 'sk-test' | codex login --with-api-key; echo $?   # expect 1
```

## 4) Rapid deploy & versioning
Create a stamped release and update the active binary + alias:
```bash
make release
```
Artifacts:
- `dist/releases/codex-<YYYYMMDDHHMM>-<branch>-<sha>`
- `dist/bin/codex` → symlink to the stamped binary
- `dist/bin/cxplus` → symlink to `codex`
 - `dist/release.json` → `{ "stamp": "…", "binary": "…" }` (TUI `/status` shows the stamp)

Switch / rollback:
```bash
make list-releases
make switch VERSION=<stamp>
make rollback
```

## 5) Install a user‑level alias (safe)
```bash
make install-local   # creates ~/.local/bin/cxplus -> dist/bin/cxplus
```
Then add to your shell:
```bash
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
```bash
export CHUTES_API_KEY=...            # or set via .env
dist/bin/cxplus chutes recommend     # prints openai/<model-id>
dist/bin/cxplus chutes exec --json "Say hello"
```

Price cap example (skip reasons visible with `CHUTES_DISCOVERY_DEBUG=1`):

```bash
# Cap output price to 15 USD / 1M tokens and show why candidates were skipped
export CHUTES_DISCOVERY_DEBUG=1
dist/bin/cxplus chutes recommend --max-output-ppm 15 --show-base
```

## 7.1) Runpod quick start (OpenAI‑compatible)

If you run an OpenAI‑compatible endpoint on Runpod (vLLM/SGLang template or your own image), set these envs and add the provider automatically during `make config`:

```bash
export RUNPOD_API_BASE=https://<your-endpoint>/v1
export RUNPOD_API_KEY=rpv2-...
make config   # adds [model_providers.runpod] to dist/config/config.toml

# Use the Runpod provider for a one‑off run
dist/bin/cxplus -c model_provider=runpod -c model="gpt-<your-model>" exec "hello" --seed 42
```

Notes:
- If `local_only=true`, remote endpoints (like Runpod) are denied by policy.
- For Responses API, set `wire_api = "responses"` in the provider block if your endpoint supports it; default is chat.

## 8) Windows packaging
```bash
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

### Exit codes (for CI)

- `0` — ok (no errors)
- `1` — model/tool error during run
- `5` — timed out or interrupted (graceful shutdown completed)

The same value is written to each run’s summary JSON as `exit_code`.

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

```toml
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


## CLI Review Loop (terminal‑first)

Prereqs: logged in to GitHub; macOS Accessibility granted if using Copilot Web helpers.

1. Build prompt:
   ```bash
   make copilot-prompt-send OUT=local/copilot_prompt.txt SEND=0 BROWSER=safari URL=https://github.com/copilot
   ```
2. Manually tweak in the browser, then auto‑send:
   ```bash
   make copilot-prompt-send OUT=local/copilot_prompt.txt SEND=1 BROWSER=safari URL=https://github.com/copilot
   ```
3. Wait for a stable response and save:
   ```bash
   make copilot-web-wait OUT=local/copilot_review.txt BROWSER=safari URL=https://github.com/copilot INTERVAL=1 STABLE=3 MAX=90
   ```
4. Process and surface TODOs / patches:
   ```bash
   make copilot-process-review IN=local/copilot_review.txt
   ```
5. Optional mailbox append/watch (idempotent JSONL):
   ```bash
   make mailbox-append BODY="$(cat local/copilot_review.txt)" CHANNEL=reviews PRIO=5 TTL=3600
   make mailbox-watch MAILBOX=.codex/mailbox.jsonl
   ```
