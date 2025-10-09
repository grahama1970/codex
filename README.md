<!-- Hero: centered SVG with theme-aware sources -->
<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="codex-rs/logo-dark-centered.svg" type="image/svg+xml">
    <source media="(prefers-color-scheme: light)" srcset="codex-rs/logo-light-centered.svg" type="image/svg+xml">
    <img
      src="codex-rs/logo-light-centered.svg"
      alt="cxplus logo"
      width="600"
      style="max-width:92vw;height:auto;"
    >
  </picture>
</p>

<!-- Headline -->
<p align="center">
  <strong>cxplus</strong> — knowledge-first, deterministic Codex fork for agent-to-agent automation.
</p>

<!-- Subline: pillars -->
<p align="center">
  <sub>Agent-to-agent • ArangoDB pre-hooks • Deterministic runs • Auditable context</sub>
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache--2.0-blue.svg" alt="Apache-2.0"></a>
</p>

---

# cxplus

**cxplus** is a knowledge-first fork of OpenAI’s **Codex CLI**.  
It introduces **agent-to-agent communication**, **ArangoDB pre-hooks** for cited context retrieval, and **deterministic, cost-aware execution** for CI/CD pipelines — keeping the familiar Codex interface while extending it for multi-agent automation.

> If you want Codex in your editor (VS Code, Cursor, Windsurf), see <a href="https://developers.openai.com/codex/ide">install in your IDE</a>.  
> For OpenAI’s cloud agent (**Codex Web**), visit <a href="https://chatgpt.com/codex">chatgpt.com/codex</a>.

---

## Why this fork

We ship automation with **predictable cost** and **fewer regressions**.

- **Tool-first, Knowledge-first** → Call databases/tools **before** the model; compact prompts and auditable decisions.  
- **Determinism on demand** → `--seed` + artifacts (NDJSON events, summary JSON) for reproducible runs and audits.  
- **Confidence before release** → `make package` → `make test` (offline) → `make scenarios` (live) validates the **exact** binary.  
- **Cost-aware discovery** → Chutes selects for capability **and** price (not just biggest SOTA), with transparent filter reasons.

### Local‑Only (No Egress)

Enable a strict no‑egress posture:

```toml
# ~/.codex/config.toml
local_only = true

[tools]
web_search = false  # implied by local_only = true
```

When `local_only = true`:
- Only localhost/127.0.0.1/[::1] model endpoints are allowed.
- HTTP clients bypass environment proxies via `CODEX_LOCAL_ONLY=1`.
- Web search, remote MCP over HTTP, OTEL exporter, and notifier hooks are disabled.
- Login flows (device code / ChatGPT / API key) are blocked at the CLI.

Verify:

```bash
printf 'sk-test' | codex login --with-api-key; echo $?   # expect 1
```

### Determinism on demand (`--seed <u64>`) 

- Where supported we enforce `temperature=0.0`, `top_p=1.0`, and set `seed=<u64>`.
- We neutralize other sampling knobs when available (e.g., frequency/presence penalties, top_k/typical_p, logit_bias).
- Artifacts (events NDJSON + summary JSON) include the seed for reproducibility.
- Caveat: external tools/live data/time‑varying prompts can introduce drift—pin inputs/fixtures in CI.

### Artifacts (where and how to read)

- Events: `./.codex/runs/*-events.ndjson` (one event per line)
- Summary: `./.codex/runs/*-summary.json`

Quick reads:

```bash
jq -r '.status, .model, .provider, .seed' ./.codex/runs/*-summary.json | paste - - - -
jq -c 'select(.msg.type=="agent_message")' ./.codex/runs/*-events.ndjson | head -n 3
```

> **Try it in 60 seconds**
>
> ```bash
> make package && RUSTUP_TOOLCHAIN=1.90.0 make test
> ./dist/bin/codex exec "hello"    # inspect ./.codex/runs/*-events.ndjson
> ```

> **Fork scope & support**  
> Experimental, personal fork; not affiliated with OpenAI’s Codex. See [FORK_POLICY.md](./FORK_POLICY.md).

---

## Chutes in 30 seconds

```bash
export CHUTES_API_KEY=…
# Optional: deterministic discovery without network
export CHUTES_CATALOG_FIXTURE=/abs/path/catalog.json

# Discover a capable, budget-aligned coding/multimodal model
dist/bin/cxplus chutes recommend --show-base

# Warm-up (dry-run works without keys)
dist/bin/cxplus chutes warmup --secs 4 --dry-run

# Execute (JSON mode)
dist/bin/cxplus chutes exec --json "Say hello"
````

**Why:** Route for capability + price, warm caches, and keep context small so spend goes to **code**, not transcripts.

---

## Using cxplus with [scillm](docs/SCILLM_LOCAL.md) (litellm fork)

*See also:* [litellm upstream](https://github.com/BerriAI/litellm)

cxplus serves as the operator-facing CLI around **scillm (litellm router)** + **Chutes**.

```bash
# Example: point cxplus at a local OpenAI-compatible proxy
export OPENAI_API_BASE=http://localhost:4000/v1
export OPENAI_API_KEY=sk-proxy-dev

# Discover → warm-up → exec
export CHUTES_API_KEY=…
dist/bin/cxplus chutes recommend --show-base
dist/bin/cxplus chutes warmup --secs 4 --dry-run
dist/bin/cxplus chutes exec --json "List three refactor steps"
```

* Tiny context keeps routers spending tokens on **code**.
* `--seed` makes nightly pipelines debuggable.
* See the [scillm guide](docs/SCILLM_LOCAL.md) and [litellm upstream](https://github.com/BerriAI/litellm) for router setup.

---

## Reliability & Determinism (scriptable semantics)

* **Always-on artifacts** (default `./.codex/runs/` or `--summary-dir`):

  * **Events NDJSON** — one event per line (`seq`, `run_id`), with a `run_timeout` marker on budget expiry.
  * **Summary JSON** — `schema_version`, `status`, `exit_code`, `duration_ms`, `event_count`, **model/provider**, `events_path`, **seed** (when set), last error.
* **Time budget & graceful stop** — `--run-timeout-secs <n>` sends Interrupt, waits a short grace (`--shutdown-grace-ms`, default **800ms**), then Shutdown (exit code **5**).
* **Deterministic runs** — `--seed <u64>` persists; where supported we enforce **temperature=0** and **top_p=1**.

### CI quick check (GitHub Actions)

```yaml
# .github/workflows/cxplus-check.yml
name: cxplus-check
on: [push, pull_request]
jobs:
  run-cxplus:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: sudo apt-get update && sudo apt-get install -y build-essential
      - run: make package && RUSTUP_TOOLCHAIN=1.90.0 make test
      - run: ./dist/bin/codex exec "hello"
```

---

## Failure modes (fast fixes)

* **“No suitable model”** → Relax filters: lower `--min-params`, loosen `--max-output-ppm`, remove `--require-capabilities` / `--require-modalities`.
  For discovery analysis, set `CHUTES_DISCOVERY_DEBUG=1` to print filter reasons.
* **Fixture returns nothing** → Confirm `CHUTES_CATALOG_FIXTURE` path; JSON must have top-level `items`.
* **Warm-up fails** → Use `--dry-run` first; then set `CHUTES_API_KEY` and optionally `CHUTES_API_BASE`.
* **Timeouts** → Increase `--run-timeout-secs`; tune `--shutdown-grace-ms` (default 800ms).
* **Artifacts missing** → Check CWD and any `--summary-dir` override; verify write perms.

---

## Install & run (uses upstream Codex binary)

Install the upstream Codex binary globally; cxplus builds on it.

```bash
# npm
npm install -g @openai/codex
# Homebrew
brew install codex
```

Run:

```bash
codex    # canonical binary
# or the alias provided by this repo:
cxplus
```

<details>
<summary>Download a prebuilt binary</summary>

See the <a href="https://github.com/openai/codex/releases/latest">latest GitHub Release</a>:

* macOS: `codex-aarch64-apple-darwin.tar.gz` (Apple Silicon), `codex-x86_64-apple-darwin.tar.gz`
* Linux: `codex-x86_64-unknown-linux-musl.tar.gz`, `codex-aarch64-unknown-linux-musl.tar.gz`

Extract and rename to `codex` if desired.

</details>

---

## Knowledge-first context (RFC)

Prompts are built from compact, cited **evidence** (ArangoDB via memory-agent MCP), not long transcripts.

* **Benefits:** 60–85% expected token reduction; better traceability/determinism.
* **Status:** RFC/draft; gated behind a provider/profile switch.
* **Design:** `docs/feature_recipes/knowledge-first-context.md`

---

## Make targets (essentials)

* `make package` — build + config
* `make test` — deterministic, offline tests (uses `dist/config` via `CODEX_HOME`)
* `make scenarios` — live, post-compile
* `RUN_LIVE=1 make verify` — test + live in one
* `make release` — stamped binary in `dist/releases/`; updates `dist/bin/codex` + `dist/bin/cxplus`
* `make install-local` — creates `~/.local/bin/cxplus` → `dist/bin/cxplus`

---

## Windows packaging

```bash
make package-windows   # writes dist/cxplus-windows.zip
```

Zip includes `cxplus.cmd` / `cxplus.ps1` wrappers and `codex` / `codex.exe` (when available). Put `cxplus.cmd` on your `%PATH%`.

---

## Brand assets (SVG)

* `codex-rs/logo-dark-centered.svg` and `codex-rs/logo-light-centered.svg`

  * Centered geometry: `viewBox="0 0 400 80"` with `translate(126.5,56)`
  * Light wash fix: c/x masks force white glyphs for full accent sweep
  * Themeable accent: override with `style="--accent:#FF4DDE"`
  * Honors `prefers-reduced-motion` and `data-static="true"`

**Accent override example (HTML):**

```html
<img src="codex-rs/logo-light-centered.svg" style="--accent:#FF4DDE" width="220" alt="cxplus logo">
```

Embed with the `<picture>` block at the top (as shown).

---

## Docs & FAQ

* Getting started: `docs/getting-started.md`, `docs/config.md`, `docs/authentication.md`, `docs/advanced.md`
* Sandbox & approvals: `docs/sandbox.md` • Exec: `docs/exec.md` • ZDR: `docs/zdr.md` • FAQ: `docs/faq.md`
* Chutes: `docs/chutes.md` (discovery + troubleshooting) • Slash: `docs/slash-commands.md`
* Knowledge-first RFC: `docs/feature_recipes/knowledge-first-context.md`

---

## License

Licensed under the [Apache-2.0 License](LICENSE).

