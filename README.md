<!-- Hero: centered SVG with theme-aware sources (no hacks) -->
<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="codex-rs/logo-dark-centered.svg" type="image/svg+xml">
    <source media="(prefers-color-scheme: light)" srcset="codex-rs/logo-light-centered.svg" type="image/svg+xml">
    <img src="codex-rs/logo-light-centered.svg" alt="cxplus — deterministic, cost-aware Codex fork for CI & automation" width="420">
  </picture>
</p>

<p align="center">
  <strong>cxplus</strong> — deterministic, cost-aware Codex fork for CI & automation.
  <br />
  <sub>Artifact trails • Policy hooks • Cost-aware multi-model discovery • Knowledge-first context</sub>
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache--2.0-blue.svg" alt="Apache-2.0"></a>
</p>

---

# cxplus

**cxplus** is our production-focused fork of OpenAI’s **Codex CLI**. It keeps upstream CLI/TUI and MCP, then adds **reliability**, **cost controls**, and **post-compile validation** for CI/CD and automation.

> If you want Codex in your editor (VS Code, Cursor, Windsurf), see <a href="https://developers.openai.com/codex/ide">install in your IDE</a>.  
> If you’re looking for OpenAI’s cloud agent, **Codex Web**, go to <a href="https://chatgpt.com/codex">chatgpt.com/codex</a>.

---

## Why this fork

We operate cxplus as a **reliable, cost-aware engineering tool** for automation and CI. **cxplus calls databases and tools first, then the model**, which keeps prompts compact and decisions auditable. It selects **capable, budget-aligned models** (not just the largest SOTA), can run **deterministic** when desired, and **validates behavior after compile** via a one-command pipeline. These choices let us ship agents with **predictable cost**, **fewer regressions**, and **faster feedback**.

- **Tool-first, Knowledge-first** — call DBs and tools before the model to cut token spend and reduce drift.  
- **Determinism on demand** — `--seed` locks sampling; every run emits NDJSON events + summary JSON for reproducibility and audits.  
- **Post-compile confidence** — `make package` → `make test` (offline) → `make scenarios` (live) validates the **exact binary** you ship.  
- **Cost-effective discovery** — Chutes picks capability **and** price, with transparent filter reasons.

> **Try it in 60 seconds**
>
> ```bash
> make package && RUSTUP_TOOLCHAIN=1.90.0 make test
> ./dist/bin/codex exec "hello"    # see ./.codex/runs/*-events.ndjson
> ```

> **Fork scope & support**  
> This is an experimental, personal fork. It’s not affiliated with OpenAI’s Codex and has no official support. See [FORK_POLICY.md](./FORK_POLICY.md).

---

## Chutes in 30 seconds

```bash
export CHUTES_API_KEY=…
# Optional: deterministic discovery without network
export CHUTES_CATALOG_FIXTURE=/abs/path/to/catalog.json

# Discover a capable, budget-aligned coding/multimodal model (≥70B bias)
dist/bin/cxplus chutes recommend --show-base

# Warm-up (dry-run works without keys)
dist/bin/cxplus chutes warmup --secs 4 --dry-run

# Execute (JSON mode)
dist/bin/cxplus chutes exec --json "Say hello"
````

**Why it exists:** Choose **capability + price**, warm caches before real work, and keep context small so spend goes to **code**, not transcripts.

---

## Using cxplus with scillm (litellm)

We use cxplus as the operator-facing CLI around **scillm (litellm router)** and **Chutes**.

```bash
# Point cxplus at your litellm (OpenAI-compatible) proxy (example)
export OPENAI_API_BASE=http://localhost:4000/v1
export OPENAI_API_KEY=sk-proxy-dev

# Discover → warm-up → exec
export CHUTES_API_KEY=…
dist/bin/cxplus chutes recommend --show-base
dist/bin/cxplus chutes warmup --secs 4 --dry-run
dist/bin/cxplus chutes exec --json "List three refactor steps"
```

* We keep context tiny so routers spend tokens on **code**, not chat history.
* Deterministic runs (`--seed`) make nightly pipelines debuggable.
* See `docs/SCILLM_LOCAL.md` and the litellm README (internal path):
  `/home/graham/workspace/experiments/litellm/README.md`

---

## Reliability & artifacts (scriptable semantics)

Headless runs (`codex exec`) mirror interactive reliability.

* **Always-on artifacts** (to `./.codex/runs/` unless you pass `--summary-dir`)

  * **Events NDJSON** — one event per line (`seq`, `run_id`), with a synthetic `run_timeout` marker on budget expiry.
  * **Summary JSON** — fields include: `schema_version`, `status`, `exit_code`, `duration_ms`, `event_count`, **model/provider**, `events_path`, **seed** (when set), and last error.
* **Time budget with graceful stop** — `--run-timeout-secs <n>` sends Interrupt, waits a short grace (`--shutdown-grace-ms`, default **800ms**), then requests Shutdown (exit code **5**).
* **Helpful stderr hints** on failure, with pointers to artifacts.

### Deterministic runs

* `--seed <u64>` persists in summary; where supported we enforce **temperature=0** and **top_p=1** so repeated runs match.

---

## Failure modes (and quick fixes)

* **“No suitable model”** → Relax filters: lower `--min-params`, loosen `--max-output-ppm`, remove `--require-capabilities` / `--require-modalities`.
* **Fixture mode produces nothing** → Verify `CHUTES_CATALOG_FIXTURE` path; JSON must have top-level `items`.
* **Warm-up network failures** → Use `--dry-run` first; then set `CHUTES_API_KEY` and optionally `CHUTES_API_BASE`.
* **Timeouts** → Increase `--run-timeout-secs`; adjust `--shutdown-grace-ms` (default 800ms).
* **Artifacts missing** → Confirm working directory and any `--summary-dir` overrides; check write permissions.

---

## Install & run (upstream parity)

Install the upstream Codex binary globally, then run as usual (cxplus is the forked experience in this repo).

```bash
# npm
npm install -g @openai/codex

# Homebrew
brew install codex
```

Run the CLI:

```bash
codex    # canonical binary name
# or the alias in this repo (see make target below):
cxplus
```

<details>
<summary>Download a prebuilt binary</summary>

Go to the <a href="https://github.com/openai/codex/releases/latest">latest GitHub Release</a> and grab the archive for your platform:

* macOS

  * Apple Silicon/arm64: `codex-aarch64-apple-darwin.tar.gz`
  * x86_64: `codex-x86_64-apple-darwin.tar.gz`
* Linux

  * x86_64: `codex-x86_64-unknown-linux-musl.tar.gz`
  * arm64: `codex-aarch64-unknown-linux-musl.tar.gz`

Extract and rename to `codex` for convenience.

</details>

### Using Codex with your ChatGPT plan (optional)

Run `codex` and select **Sign in with ChatGPT** (Plus/Pro/Team/Edu/Enterprise).
You can also use an API key—see `docs/authentication.md`.

---

## Knowledge-first context (RFC)

We’re adopting a **Knowledge-first** strategy: build prompts from compact, cited **evidence** (ArangoDB via the memory-agent MCP), not long transcripts.

* **Benefits:** 60–85% expected token reduction on real tasks; improved determinism and traceability.
* **Status:** RFC/draft; initially gated behind a provider/profile switch.
* **Design:** `docs/feature_recipes/knowledge-first-context.md`

> **Trade-off:** We keep only a tiny recent chat window and pass an evidence bundle with citations.

---

## Make targets you’ll use

* `make package` — build + config
* `make test` — deterministic, offline tests (uses `dist/config` via `CODEX_HOME`)
* `make scenarios` — live, post-compile scenarios
* `RUN_LIVE=1 make verify` — deterministic tests **plus** live scenarios
* `make release` — stamped binary in `dist/releases/`; updates `dist/bin/codex` + `dist/bin/cxplus`
* `make install-local` — creates `~/.local/bin/cxplus` → `dist/bin/cxplus`
* `make chutes-profiles` — generate cached discovery profiles (when present in your repo)

**Environment knobs**

* General: `CODEX_BIN` (defaults to `dist/bin/codex`), `CODEX_HOME` (tests default to `dist/config`), `RUN_LIVE=1` for `make verify`
* Chutes: `CHUTES_API_KEY`, `CHUTES_API_BASE`, `CHUTES_CATALOG_BASE`, `CHUTES_CATALOG_FIXTURE`, `CHUTES_DISCOVERY_DEBUG`, `CHUTES_EXTRA_CAPS`, `CHUTES_FORCE_PROVIDER_BASE`, `CHUTES_WARMUP`, `CHUTES_WARMUP_SECS`
* Slash QOL: `GREP_MAX_LINES`, `OPEN_MAX_KB`, `ENABLE_SLASH_WRITE`
* Notifications: `SLACK_WEBHOOK_URL` (enable via `notify = ["codex-notify-slack"]` in `~/.codex/config.toml`)

---

## Windows packaging

```bash
make package-windows   # writes dist/cxplus-windows.zip
```

Zip includes `cxplus.cmd` / `cxplus.ps1` wrappers and `codex` / `codex.exe` when available. Put `cxplus.cmd` on your `%PATH%`.

---

## Brand assets (SVG)

* Animated wordmark: `codex-rs/logo-dark-centered.svg` and `codex-rs/logo-light-centered.svg`

  * **Centered geometry:** `viewBox="0 0 400 80"` with `translate(126.5,56)`
  * **Light wash fix:** c/x masks force white glyphs; the accent fully washes both letters
  * **Themeable accent:** override with `style="--accent:#FF4DDE"` (CLI defaults to magenta)
  * **Motion:** idle-only halo; zero movement of base letters; `+` accent pop-in/spin/pop-out
  * **Accessibility:** honors `prefers-reduced-motion` and `data-static="true"`

Embed in README using the `<picture>` block at the top (as shown).

---

## Docs & FAQ

* Getting started: `docs/getting-started.md`, `docs/config.md`, `docs/authentication.md`, `docs/advanced.md`
* Sandbox & approvals: `docs/sandbox.md`
* Exec: `docs/exec.md`
* ZDR: `docs/zdr.md`
* FAQ: `docs/faq.md`
* Chutes: `docs/chutes.md` (discovery + troubleshooting)
* Slash commands: `docs/slash-commands.md`
* Knowledge-first RFC: `docs/feature_recipes/knowledge-first-context.md`

---

## License

This repository is licensed under the [Apache-2.0 License](LICENSE).

