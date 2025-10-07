# Features Overview

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="./codex-rs/logo-dark.svg" />
    <source media="(prefers-color-scheme: light)" srcset="./codex-rs/logo-light.svg" />
    <img src="./codex-rs/logo-light.svg" alt="cxplus logo" width="480" />
  </picture>
</p>

> Experimental fork disclaimer
>
> This repository is an experimental, personal fork ("cxplus playground"). It is not intended to be merged upstream into OpenAI’s Codex, and it has no official support. See FORK_POLICY.md for details.

This fork extends Codex CLI with discovery, testing, and deployment ergonomics. Below is a high‑level, scannable overview.

## Unique Capabilities (at a glance)

- Post‑compile verification: deterministic tests and live scenarios execute the compiled binary.
- Headless artifacts by default: NDJSON event stream + summary JSON for every run.
- Cost‑aware model auto‑discovery (Chutes): parameters, capabilities, and price caps with debug skip reasons.
- Knowledge‑First context (experimental): pre‑LLM retrieval + shaping; emits a single metrics summary line.
- One‑command package/switch/rollback of stamped builds.
- Warmup + capacity helpers; CI‑friendly sandbox/approvals defaults.
 - Agent↔Agent near‑instant communications for multi‑agent coordination.

## Feature Matrix (cxplus vs a typical LLM CLI)

| Capability | Typical CLI | cxplus |
| --- | --- | --- |
| Post‑compile tests/scenarios | ✖ | ✔ |
| NDJSON + summary artifacts for every run | ✖ | ✔ |
| Time‑budgeted runs with graceful shutdown | △ | ✔ |
| Model auto‑discovery with cost + capability filters | ✖ | ✔ |
| Price‑cap safe handling (NaN/absent prices) | ✖ | ✔ |
| Knowledge‑First metrics line (context.summary v2) | ✖ | ✔ |
| Stamped build switch/rollback | ✖ | ✔ |
| Warmup + capacity hints | △ | ✔ |
| Sandbox + approvals defaults for CI | △ | ✔ |
| Agent↔Agent near‑instant communications | ✖ | ✔ |

## Exec Parity & Reliability

- Headless runs (`codex exec`) are reliable by default and mirror interactive semantics where it matters for CI:
  - Always‑on artifacts under `./.codex/runs/` (unless `--summary-dir` is used)
    - Events NDJSON: one event per line with `seq` and `run_id`; synthetic `run_timeout` marker on budget expiry
    - Summary JSON: `schema_version`, `status`, `exit_code`, `duration_ms`, `event_count`, model/provider, `events_path`, and last error if any
  - Time budget with graceful stop: `--run-timeout-secs <n>` (exit code `5` on timeout); `--shutdown-grace-ms` tunes grace (default 800ms)
  - Helpful stderr hints (rate‑limit, DNS/resolve, timeout) and pointers to artifact paths
  - Advanced parity flags (optional; defaults already stable): `--force-cli-source`, `--keep-approval-policy`, `--seed <u64>`

## CLI & Provider Integration

- Chutes auto‑discovery: `codex chutes recommend`
  - Filters: `--min-params`, `--max-params`, `--max-output-ppm`, `--require-modalities`, `--require-capabilities`
  - Ranking tie‑breakers: output price (asc) → effective params (desc) → context (desc) → input price (asc)
  - Price‑cap NaN relaxation (one‑pass) with debug notice
  - Fixture hook for deterministic tests: `CHUTES_CATALOG_FIXTURE=/path.json`
  - Optional base URL print: `--show-base`
- Chutes exec: `codex chutes exec ...`
  - `--wire-api chat|responses`, `--images`, optional warm‑up via `--warmup-secs` or `CHUTES_WARMUP=1`

## TUI

- Slash commands (MVP parity)
  - `/discover` (read‑only) — appends an info cell with model id
  - Optional session auto‑apply: `APPLY_DISCOVER_AUTO=1`
  - `/status`, `/model`, `/provider`, `/profile` (exec has more slash helpers: `/grep`, `/open`, `/fmt`, `/build`, `/test`)

## Safety & Ergonomics

- Write gating for slash (`ENABLE_SLASH_WRITE=1`) with one‑time notice
- `/open` size guard (default 512KB) → override `OPEN_MAX_KB`
- `/grep` truncation (default 200 lines) → override `GREP_MAX_LINES` and truncation marker

## Testing & Scenarios

- Deterministic tests (offline): `make test`
- Live, post‑compile scenarios: `make scenarios`
- Fixture‑based scenario validates price‑cap relaxation notice

## Knowledge‑First Context (RFC, experimental)

- Goal: source compact, cited evidence from ArangoDB via memory‑agent MCP before any LLM call; keep only a tiny recent chat window.
- Benefits: 60–85% expected token reduction on real tasks; better traceability and determinism.
- Status: design document added; wiring behind a provider flag/profile is planned (experimental).
- Docs: `docs/feature_recipes/knowledge-first-context.md`

Emits: when enabled, a single `context.summary` NDJSON record (version=2) is written once after context assembly (before streaming). It includes provider, quotas, max tokens, retrieval latency and counts, per‑section token usage, and truncation flags (no raw evidence content).

Experimental config keys:

```
[context]
provider = "arango"            # default is "minimal"
max_context_tokens = 8192

[context.budget]
recent_pct = 15
plan_pct = 10
evidence_pct = 60
tools_pct = 15

[context.arango]
endpoint = "http://localhost:8529"
database = "codex"
mcp_tool = "memory-agent"
search_k = 12
neighbors_depth = 1
timeout_ms = 800
max_evidence_items = 12
```

## Deploy & Versioning

- `make release` → stamped binary in `dist/releases` and updates active symlink in `dist/bin`
- `make switch VERSION=<stamp>` / `make rollback`
- Public alias: `dist/bin/cxplus` (symlink to `codex`) — safe to install per‑user: `make install-local`

## Key Environment Variables

| Variable | Purpose |
| --- | --- |
| `CHUTES_API_KEY` | Chutes API key (Authorization: Bearer …) |
| `CHUTES_API_BASE` | Inference base URL (OpenAI‑compatible) |
| `CHUTES_CATALOG_BASE` | Discovery catalog base URL |
| `CHUTES_CATALOG_FIXTURE` | Deterministic offline discovery JSON |
| `CHUTES_DISCOVERY_DEBUG` | Print candidate filter reasons + relax notice |
| `CHUTES_EXTRA_CAPS` | Append capability keys (e.g., `programming,tools`) |
| `CHUTES_FORCE_PROVIDER_BASE` | Force provider base instead of derived per‑chute |
| `CHUTES_WARMUP`, `CHUTES_WARMUP_SECS` | Optional warm‑up call |
| `APPLY_DISCOVER_AUTO` | TUI: auto‑apply discovered model for session |
| `OPEN_MAX_KB` | `/open` size guard override |
| `GREP_MAX_LINES` | `/grep` line cap override |
| `ENABLE_SLASH_WRITE` | Allow write‑capable slash targets (one‑time notice) |

## Docs (Auto‑generated)

- Reference lives under `docs/generated/` (see `docs/generated/README.md`).
- Regenerate locally: `make docs-gen` (or `make docs-fix` to generate + stage).
- CI gate: `make docs-drift` runs on PRs and pushes to `main` (fails on drift).
- Optional site: mdBook scaffold under `docs/book`; build with `make docs-book-build`.
