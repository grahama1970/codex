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

### Troubleshooting & advanced

- `CHUTES_DISCOVERY_DEBUG=1` — prints skip reasons (stderr) for each catalog item filtered out
  during `recommend` or fallback auto‑discovery (price, capabilities, params, modalities).
- `CHUTES_API_BASE` — when set, overrides any derived per‑chute base URL.
- `CHUTES_FORCE_PROVIDER_BASE=1` — always prefer the provider base URL even if a sanitized
  per‑chute domain can be derived. Useful when routing via a centralized gateway/LB.
- `CHUTES_EXTRA_CAPS="programming,tools"` — appends additional capability keys to the fallback
  auto‑discovery (exec without explicit model) in addition to the default `coding,code`.

#### Warm‑up behavior

New or cold Chutes models may require a short “warm‑up” period before the first token is
returned. You can ask Codex to pre‑warm the model before launching exec:

- CLI flag: `codex chutes exec --warmup-secs 8 "…"`
- Env: set `CHUTES_WARMUP=1` and optionally `CHUTES_WARMUP_SECS=8`.

This sends a tiny chat completions request (`max_tokens=1`, `temperature=0`) to the selected
model on your Chutes base URL (provider defaults or derived per‑chute). Non‑2xx responses
are retried briefly with exponential backoff (up to the provided budget).
