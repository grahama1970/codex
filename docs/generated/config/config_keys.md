# Configuration & Environment Reference

_Auto-generated; keys subject to change pre-beta._

## Environment Variables

| Name | Default | Description |
|------|---------|-------------|
| `CODEX_API_KEY` | `(unset)` | API key override for exec mode |
| `CHUTES_API_KEY` | `(unset)` | Chutes OpenAI-compatible key |
| `CHUTES_API_BASE` | `https://llm.chutes.ai/v1` | Override base URL for Chutes inference |
| `CHUTES_CATALOG_BASE` | `https://api.chutes.ai/chutes/` | Catalog endpoint |
| `CHUTES_WARMUP` | `0` | Enable warm-up call before exec |
| `CHUTES_WARMUP_SECS` | `8` | Warm-up budget seconds |
| `CHUTES_CATALOG_FIXTURE` | `(unset)` | Path to a static catalog JSON (testing) |
| `CONTEXT_FORCE_MINIMAL` | `0` | Force minimal context provider |
| `CONTEXT_DEBUG` | `0` | Emit verbose context summary debugging |

## `[context]` Table Keys

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `provider` | string | `minimal` | `minimal` or `arango` (experimental) |
| `max_context_tokens` | integer | `8192` | Global token budget for Knowledge-First bundle |
| `[context.budget].recent_pct` | integer% | `15` | Recent turns token share |
| `[context.budget].plan_pct` | integer% | `10` | Plan section token share |
| `[context.budget].evidence_pct` | integer% | `60` | Evidence token share |
| `[context.budget].tools_pct` | integer% | `15` | Tool deltas token share |
| `[context.arango].endpoint` | string | `http://localhost:8529` | Arango endpoint (Phase-1) |
| `[context.arango].database` | string | `codex` | Arango DB name |
| `[context.arango].mcp_tool` | string | `memory-agent` | MCP tool id |