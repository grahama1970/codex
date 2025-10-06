# Features Overview

This fork extends Codex CLI with discovery, testing, and deployment ergonomics. Below is a high‑level, scannable overview.

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

