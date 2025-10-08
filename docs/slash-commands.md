# Slash Commands (MVP)

Codex supports simple “slash commands” that you can type as a one‑line input beginning with `/`.
They are handled locally (no model call) and are available in both non‑interactive `exec` mode and in the TUI.

Example:

```
/help
/status
/discover --min-params 10000000000 --require-modalities text,image --require-capabilities coding,code
```

Supported verbs (MVP)
- `/help` — list commands
- `/status` — show basic session status (provider/model/profile)
- `/model <id>` — print how to set model via `-c model="…"`
- `/provider <id>` — print how to set provider via `-c model_provider="…"`
- `/profile <name>` — print how to run with a profile via `-p <name>`
- `/discover [flags]` — call `codex chutes recommend` under the hood and print the discovered model id
  - Flags: `--min-params N`, `--max-params N`, `--max-output-ppm X`, `--require-modalities A,B`, `--require-capabilities X,Y`
  - Exec/TUI: prints `discovered model: <id>` and a ready‑to‑paste hint `-c model="<id>"`. TUI also appends an info cell with the same guidance.

Safety & behavior
- Slash commands print to stderr and exit immediately; the agent is not invoked.
- Commands that would mutate the repo (fmt/build/test) should respect sandbox/approval policies; they are not enabled yet in this MVP.

 Troubleshooting
- Discovery uses the same filters and logic as `codex chutes recommend`.
- If discovery fails due to price‑cap + NaN, the CLI prints a one‑line relaxation notice and retries once without the cap.
 - Invalid numeric flags on `/discover` return an inline `parse-error` message.
 - `/grep` truncates output to 200 lines and marks with `(truncated)` if more results remain.
 - `/open` refuses files larger than 512KB to keep UX responsive.

Environment
- `CHUTES_DISCOVERY_DEBUG=1` — prints skip reasons during discovery.
- `CHUTES_API_BASE` / `CHUTES_FORCE_PROVIDER_BASE=1` — see `docs/chutes.md`.
- `CHUTES_EXTRA_CAPS` — appended capability keys for exec fallback (not applied to explicit `/discover`).
 - `CHUTES_CATALOG_FIXTURE=/path/catalog.json` — offline deterministic discovery (no network).
 - `OPEN_MAX_KB=1024` — adjust `/open` file size guard.
 - `GREP_MAX_LINES=400` — adjust `/grep` output cap.
 - `APPLY_DISCOVER_AUTO=1` — TUI: auto-apply the discovered model to the current session.
- `ENABLE_SLASH_WRITE=1` — enable running `make` targets for `/fmt`, `/build`, `/test` (emits a one‑time warning).
