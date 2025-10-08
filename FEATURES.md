# Features Overview

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="./codex-rs/logo-dark-centered.svg" />
    <source media="(prefers-color-scheme: light)" srcset="./codex-rs/logo-light-centered.svg" />
    <img src="./codex-rs/logo-light-centered.svg" alt="cxplus logo" width="480" style="display:block;margin:0 auto;" />
  </picture>
</p>

> Experimental fork disclaimer
>
> This repository is an experimental, personal fork ("cxplus playground"). It is not intended to be merged upstream into OpenAI’s Codex, and it has no official support. See FORK_POLICY.md for details.

This fork extends Codex CLI with discovery, testing, and deployment ergonomics. Below is a high‑level, scannable overview.

---

# cxplus — Features Overview

A production‑focused fork of Codex CLI for CI/CD automation and operator workflows. This page maps features to the problems they solve.

---

## Feature Overview (Skimmable)

| Feature | What it adds | Why it matters |
| --- | --- | --- |
| One-command packaging | `make package` → `dist/bin/codex` (+ `cxplus` symlink); stamped releases; instant switch/rollback | Ship compiled artifacts; switch instantly—no rebuilds |
| Execution reliability | Always-on artifacts (NDJSON events + summary JSON); `--run-timeout-secs` with graceful shutdown | Reproduce/diff any run; deterministic CI exits |
| Chutes (model auto-discovery) | `codex chutes recommend/exec` with capability/cost filters; safe price caps; image models | Picks capable, budget-aligned models; explains skips |
| Knowledge-first context | Externalized cache (ArangoDB + memory-agent); single `context.summary` v2 metrics line | Prevents context rot; smaller, traceable prompts |
| Tests & scenarios | `make test`; `make scenarios`; `RUN_LIVE=1 make verify` | Validates the exact binary you ship |
| Policy hooks (pre/post) | Pre-exec MCP/script hooks; post-run notifiers | Enforce org policies; augment prompts; notify on completion |
| Agent↔agent comms | Low-latency local/LAN messaging between agents | Simple multi-agent orchestration |
| Observability | OpenTelemetry export (HTTP/GRPC) + local artifacts | Plug into monitoring; inspect locally when you can’t |
| UX & theming | Animated, theme-aware branding; TUI slash helpers | Better ergonomics with minimal ceremony |

> Chutes = cost-/capability-aware model auto-discovery for `codex` that can recommend/execute under price caps (includes image models).

## Why this fork (one paragraph)

Build agents that are reliable, cost‑aware, and auditable. cxplus calls databases/tools first, then the model; selects capable, budget‑aligned models; supports determinism; and validates behavior after compile via a one‑command pipeline. This lets teams ship with predictable cost, fewer regressions, and faster feedback.

---

## Exec parity & reliability (scriptable semantics)

- Headless parity: `codex exec` mirrors interactive where it matters for CI.
- Artifacts (always on):
  - Events NDJSON — one event/line (`seq`, `run_id`), with a synthetic `run_timeout` marker on budget expiry
  - Summary JSON — fields: `schema_version`, `status`, `exit_code`, `duration_ms`, `event_count`, `events_path`, model/provider, seed (when set), and last error
- Timeout budget + grace: `--run-timeout-secs <n>`; `--shutdown-grace-ms` (default 800ms)
- Helpful stderr hints for rate‑limit, DNS, timeouts
- Advanced parity flags (optional): `--force-cli-source`, `--keep-approval-policy`, `--seed <u64>`

Why it exists: You can script around exit codes and files instead of scraping logs.

---

## Determinism & audits

- `--seed <u64>` persisted to summary; enforces temperature=0 and top_p=1 where supported.
- Use artifacts to reproduce, diff, and triage noisy runs.

Why it exists: “If it ran last night, it runs tomorrow.”

---

## Chutes discovery & exec

- Auto‑discovery: `codex chutes recommend`
  - Filters: `--min-params`, `--max-params`, `--max-output-ppm`, `--require-modalities`, `--require-capabilities`
  - Tie‑breaks: output price (asc) → effective params (desc) → context (desc) → input price (asc)
  - NaN price relaxation: single pass with debug notice (so catalogs with unknown prices don’t dead‑end)
  - Fixture mode for deterministic tests: `CHUTES_CATALOG_FIXTURE=/path.json`
  - Optional: `--show-base` prints the derived base URL
- Exec: `codex chutes exec ...`
  - `--wire-api chat|responses`, `--images ...`, warm‑up via `--warmup-secs` or `CHUTES_WARMUP=1`

Why it exists: Choose capability + price, not just largest SOTA, and warm caches before the real run.

---

## Testing & scenarios

- Deterministic tests (offline): `make test`
- Live, post‑compile scenarios: `make scenarios`
- Verify combo: `RUN_LIVE=1 make verify`

Why it exists: Confidence on the compiled binary you intend to ship.

---

## TUI & slash quality‑of‑life

- `/discover` (read‑only), `/status`, `/model`, `/provider`, `/profile`
- Exec helpers: `/grep` (with truncation), `/open` (size guard), `/fmt`, `/build`, `/test`
- Write gating: set `ENABLE_SLASH_WRITE=1` (one‑time notice)

---

## Safety & ergonomics

- `/open` size guard (default 512KB) → override `OPEN_MAX_KB`
- `/grep` truncation (default 200 lines) → `GREP_MAX_LINES`
- ZDR docs and approvals parity with upstream

---

## Knowledge‑First context (RFC)

- Goal: Retrieve compact, cited evidence (ArangoDB via memory‑agent MCP) before any LLM call; keep only a tiny recent chat window.
- Benefits: 60–85% expected token reduction on real tasks; improved determinism and traceability.
- Status: Design complete; to be gated behind a provider/profile flag.
- Docs: `docs/feature_recipes/knowledge-first-context.md`

See `docs/feature_recipes/agent-to-agent-comms.md` for a concrete agent‑to‑agent communications recipe (submit/events, PR reviews, notifications, and artifacts).


---

## Deploy & versioning

- `make release` — produce a stamped binary in `dist/releases` and update active symlinks in `dist/bin`
- `make switch VERSION=<stamp>` / `make rollback`
- Public alias: `dist/bin/cxplus` (symlink to `codex`); safe per‑user install: `make install-local`

---

## Environment variables (quick map)

| Variable | Purpose |
| --- | --- |
| `CODEX_BIN` | Path to compiled binary (defaults to `dist/bin/codex`) |
| `CODEX_HOME` | Config/auth directory for tests (defaults to `dist/config`) |
| `RUN_LIVE` | When `1`, `make verify` includes live scenarios |
| `CHUTES_API_KEY` | Chutes API key (Authorization: Bearer …) |
| `CHUTES_API_BASE` | Inference base URL (OpenAI‑compatible) |
| `CHUTES_CATALOG_BASE` | Discovery catalog base URL |
| `CHUTES_CATALOG_FIXTURE` | Deterministic offline discovery JSON |
| `CHUTES_DISCOVERY_DEBUG` | Print filter reasons + relaxation notice |
| `CHUTES_EXTRA_CAPS` | Append capability keys (e.g., `programming,tools`) |
| `CHUTES_FORCE_PROVIDER_BASE` | Force provider base instead of derived |
| `CHUTES_WARMUP`, `CHUTES_WARMUP_SECS` | Optional warm‑up call |
| `OPEN_MAX_KB` | `/open` size guard override |
| `GREP_MAX_LINES` | `/grep` line cap override |
| `ENABLE_SLASH_WRITE` | Allow write‑capable slash targets (one‑time notice) |

---

## Branding note (SVG)

Animated wordmark: `codex-rs/logo.svg`  
Idle‑only halo, robust c/x masks, and a themeable `--accent`. Respect `prefers-reduced-motion` and `data-static="true"` when embedding.

## Unique Capabilities (at a glance)

- Post‑compile verification: deterministic tests and live scenarios execute the compiled binary.
- Headless artifacts by default: NDJSON event stream + summary JSON for every run.
- OpenTelemetry API monitoring/export (HTTP/GRPC) with structured event catalog.
- Cost‑aware model auto‑discovery (Chutes): parameters, capabilities, and price caps with debug skip reasons.
- Knowledge‑First context (experimental): pre‑LLM retrieval + shaping; emits a single metrics summary line.
- One‑command package/switch/rollback of stamped builds.
- Warmup + capacity helpers; CI‑friendly sandbox/approvals defaults.
 - Agent↔agent near‑instant communications for multi‑agent coordination.



### Local‑Only (ITAR) Mode

Set in `~/.codex/config.toml`:

```toml
# Enforce no outbound calls except localhost
local_only = true

# Optional fine‑grained controls (effective when local_only = false)
allow_external_model_providers = false           # default true
external_provider_allowlist = ["models.acme.local"]
external_provider_denylist = ["openai.com", "api.anotherhost.com"]

[tools]
web_search = false                               # redundant when local_only = true
```

Behavior:
- Blocks non‑local model providers unless explicitly allowlisted.
- Disables web search tool and forces OTEL exporter off.
- Locals (localhost/127.0.0.1/[::1]) always allowed.

## Feature Matrix (cxplus vs a typical LLM CLI)

| Capability | Typical CLI | cxplus |
| --- | --- | --- |
| Post‑compile tests/scenarios | ✖ | ✔ |
| NDJSON + summary artifacts for every run | ✖ | ✔ |
| OpenTelemetry API monitoring/export | △ | ✔ |
| Time‑budgeted runs with graceful shutdown | △ | ✔ |
| Model auto‑discovery with cost + capability filters | ✖ | ✔ |
| Price‑cap safe handling (NaN/absent prices) | ✖ | ✔ |
| Knowledge‑First metrics line (context.summary v2) | ✖ | ✔ |
| Stamped build switch/rollback | ✖ | ✔ |
| Warmup + capacity hints | △ | ✔ |
| Sandbox + approvals defaults for CI | △ | ✔ |
| Agent↔agent near‑instant communications | ✖ | ✔ |

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
- Persistent context cache: evidence lives in ArangoDB (via memory‑agent), not transient chat history — eliminating “context rot” during long sessions and across runs.
- Benefits: 60–85% expected token reduction on real tasks; better traceability and determinism.
- Status: design document added; wiring behind a provider flag/profile is planned (experimental).
- Docs: `docs/feature_recipes/knowledge-first-context.md`

See `docs/feature_recipes/agent-to-agent-comms.md` for a concrete agent‑to‑agent communications recipe (submit/events, PR reviews, notifications, and artifacts).


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
