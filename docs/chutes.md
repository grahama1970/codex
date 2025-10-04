## Chutes Integration (OpenAI‑compatible)

Codex can auto‑discover and run against Chutes models using a built‑in `chutes` subcommand. This is useful when you want the cheapest multi‑modal model above a parameter threshold without hard‑coding a model id.

### Setup

Set the following in your `.env` at repo root (Makefile and tests source it automatically):

- `CHUTES_API_KEY` — required. Passed as `Authorization: Bearer …` to the catalog and used by the provider.
- `CHUTES_API_BASE` — optional. OpenAI‑compatible inference base (defaults to `https://llm.chutes.ai/v1`).
- `CHUTES_CATALOG_BASE` — optional. Catalog endpoint base (defaults to `https://api.chutes.ai/chutes/`).

The build creates `dist/config/config.toml` with a provider entry:

```toml
[model_providers.chutes]
name = "Chutes (OpenAI-compatible)"
base_url = "${CHUTES_API_BASE:-https://llm.chutes.ai/v1}"
env_key = "CHUTES_API_KEY"
wire_api = "chat"  # Responses also supported
```

### CLI usage

- Discover the cheapest multi‑modal (includes text + ≥1 other) model with effective params ≥ 70B:

  `codex chutes recommend`

  - `--json` prints the full catalog item.
  - `--min-params 100000000000` to change the threshold.
  - `--require-modalities "text,image"` to require a specific set.
  - `--require-capabilities "coding,code"` to require coding capabilities.

- Run exec using the discovered model (forces provider to `chutes`):

  `codex chutes exec --json "Say hello"`

  Options:
  - `--wire-api chat|responses` (default `chat`)
  - `--images /path/a.png,/path/b.png`
  - `--min-params`, `--require-modalities`, `--require-capabilities` as above

When `CHUTES_API_BASE` is not set, Codex attempts to derive a per‑chute base URL from the catalog item, falling back to the provider default if no domain/owner/slug is present.

### Live tests

With `CHUTES_API_KEY` set in `.env`, the live scenarios include:

- Auto‑discovery + exec via subcommand.
- Profile generator target (`make chutes-profiles`) writes two profiles into config.toml:
  - `coding`: text‑only, non‑SOTA, coding‑capable (requires `coding,code` capabilities).
  - `multimodal`: multi‑modal (text,image), coding‑capable.
- A robust fallback that skips if the catalog has no eligible models.

Run: `make scenarios`
