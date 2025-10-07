<p align="center"><code>npm i -g @openai/codex</code><br />or <code>brew install codex</code></p>

<p align="center"><strong>Codex CLI</strong> is a coding agent from OpenAI that runs locally on your computer.
</br>
</br>If you want Codex in your code editor (VS Code, Cursor, Windsurf), <a href="https://developers.openai.com/codex/ide">install in your IDE</a>
</br>If you are looking for the <em>cloud-based agent</em> from OpenAI, <strong>Codex Web</strong>, go to <a href="https://chatgpt.com/codex">chatgpt.com/codex</a></p>

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="./codex-rs/logo-dark.svg" />
    <source media="(prefers-color-scheme: light)" srcset="./codex-rs/logo-light.svg" />
    <img src="./codex-rs/logo-light.svg" alt="cxplus logo" width="640" style="display:block;margin:0 auto; transform: translateX(36px);" />
  </picture>
  
</p>
<p align="center">
  <img src="./.github/codex-cli-splash.png" alt="Codex CLI splash" width="80%" />
</p>

> Experimental fork disclaimer
>
> This repository is an experimental, personal fork ("cxplus playground"). It is not intended to be merged upstream into OpenAI’s Codex, and it has no official support. See FORK_POLICY.md for details.

---

## TL;DR

- Who it’s for: CLI-first devs who want reproducible runs and artifact trails.
- Why it’s different: Chutes (model auto-discovery), deterministic headless parity, and pre/post hooks.
- Try it now: `make package` → `make test` → `dist/bin/codex exec "hello"` and open `.codex/runs/*-events.ndjson`.

## What's New in This Fork (Skimmable)

| Feature | What it adds | Why it matters |
| --- | --- | --- |
| One-command packaging | `make package` → `dist/bin/codex` (+ `cxplus` symlink); stamped releases; instant switch/rollback | Ship compiled artifacts; switch versions instantly—no rebuilds |
| Execution reliability | Always-on artifacts (NDJSON events + summary JSON); `--run-timeout-secs` with graceful shutdown | Reproduce/diff any run; deterministic CI exits |
| **Chutes** (model auto-discovery) | `codex chutes recommend/exec` with capability/cost filters; safe price caps; image models | Picks capable, budget-aligned models; explains skips |
| Knowledge-first context | Externalized cache (ArangoDB + memory-agent); single `context.summary` v2 metrics line | Prevents context rot; smaller, traceable prompts |
| Tests & scenarios (post-compile) | `make test` deterministic; `make scenarios` live; `RUN_LIVE=1 make verify` | Validates the exact binary you ship |
| Policy hooks (pre/post) | Pre-exec MCP/script hooks; post-run notifiers | Enforce org policies; augment prompts; notify on completion |
| Agent↔Agent comms | Low-latency local/LAN messaging between agents | Simple multi-agent orchestration |
| Observability | OpenTelemetry export (HTTP/GRPC) + local artifacts | Plug into monitoring; inspect locally when you can’t |
| UX & theming | Animated, theme-aware branding; TUI slash helpers | Better ergonomics with minimal ceremony |

> **Chutes** = cost-/capability-aware model auto-discovery for `codex` that can recommend/execute under price caps (includes image models).

**Jump to:** [Quickstart](./QUICKSTART.md) • [Scenarios](./scenarios/) • [Features](./FEATURES.md) • [Config](./docs/config.md)

---

## Why cxplus (Beyond a Typical CLI)

cxplus bundles capabilities that make a CLI practical for CI/CD and automation:

- Post‑compile verification: tests and live scenarios run against the compiled binary (no dev/runtime drift).
- Headless parity + artifacts: every `codex exec` produces portable NDJSON + summary JSON; time‑budgeted runs with graceful shutdown.
- Model auto-discovery (Chutes): cost-aware, capability-aware selection with transparent skip reasons and safe price-cap behavior.
- Knowledge-First context (experimental): deterministic evidence shaping; context is cached in ArangoDB so state does not "rot" with long chats; a single `context.summary` v2 line records retrieval metrics for each run.
- One-command packaging & rollback: stamped releases, switching, and rollback without rebuilding.
- Warmup & capacity helpers: optional warmup/heuristics folded into CLI ergonomics.
- Safety rails: sandbox + approvals defaults tuned for CI automation.
- Agent↔Agent comms: near-instant messaging between agents for orchestration and delegation.

### Monitoring & Observability

- Built‑in OpenTelemetry exporters (HTTP/GRPC) for request/response, tool calls, and run lifecycle.
- Local artifacts for every run: NDJSON event stream + summary JSON for quick inspection.
- Configure under `[otel]` in `~/.codex/config.toml`. See docs: [Monitoring via OTEL](./docs/config.md#otel).

### Customization & Theming

- Animated, theme‑aware branding assets; consistent hero placement.
- TUI styling conventions and helpers; see `codex-rs/tui/styles.md` and [THEMING_AND_ANIMATIONS.md](./docs/THEMING_AND_ANIMATIONS.md).

---

## Feature Overview (Skimmable Table)

| Area | What you get | Why it matters |
| --- | --- | --- |
| Build & Release | `make package`, stamped builds, `make switch/rollback`, `cxplus` alias | Ship compiled artifacts, switch versions instantly without re‑building |
| Headless Reliability | NDJSON + summary JSON for every run; time budget + graceful shutdown | Reproduce, diff, and audit any run; deterministic CI |
| Knowledge‑First Context | Retrieval + shaping via memory‑agent, cached in ArangoDB; no giant chat logs | Eliminates context rot; smaller prompts; traceable evidence |
| Model Auto‑Discovery | Chutes recommend/exec with cost + capability filters and safe price caps | Pick cheap, capable models automatically, reproducibly |
| Hooks (Pre/Post) | Pre‑execution MCP/script hooks; post‑run notifiers | Enforce policy, augment prompts (agent‑memory), notify on completion |
| Agent↔Agent Comms | Near‑instant local/LAN messaging between agents | Orchestrate multi‑agent workflows simply |
| Observability | OpenTelemetry export (HTTP/GRPC) + local artifacts | Integrate with your infra; inspect locally when you don’t |
| Safety | Sensible sandbox/approval defaults for CI | Secure automation by default |
| UX | Animated theme‑aware branding; TUI slash helpers | Better ergonomics without ceremony |

---

See [FEATURES.md](FEATURES.md) for details and examples.

---

## Quickstart

### Installing and running Codex CLI

Install globally with your preferred package manager. If you use npm:

```shell
npm install -g @openai/codex
```

Alternatively, if you use Homebrew:

```shell
brew install codex
```

Then simply run `codex` to get started:

```shell
codex
```

<details>
<summary>You can also go to the <a href="https://github.com/openai/codex/releases/latest">latest GitHub Release</a> and download the appropriate binary for your platform.</summary>

Each GitHub Release contains many executables, but in practice, you likely want one of these:

- macOS
  - Apple Silicon/arm64: `codex-aarch64-apple-darwin.tar.gz`
  - x86_64 (older Mac hardware): `codex-x86_64-apple-darwin.tar.gz`
- Linux
  - x86_64: `codex-x86_64-unknown-linux-musl.tar.gz`
  - arm64: `codex-aarch64-unknown-linux-musl.tar.gz`

Each archive contains a single entry with the platform baked into the name (e.g., `codex-x86_64-unknown-linux-musl`), so you likely want to rename it to `codex` after extracting it.

</details>

### Using Codex with your ChatGPT plan

<p align="center">
  <img src="./.github/codex-cli-login.png" alt="Codex CLI login" width="80%" />
  </p>

Run `codex` and select **Sign in with ChatGPT**. We recommend signing into your ChatGPT account to use Codex as part of your Plus, Pro, Team, Edu, or Enterprise plan. [Learn more about what's included in your ChatGPT plan](https://help.openai.com/en/articles/11369540-codex-in-chatgpt).

You can also use Codex with an API key, but this requires [additional setup](./docs/authentication.md#usage-based-billing-alternative-use-an-openai-api-key). If you previously used an API key for usage-based billing, see the [migration steps](./docs/authentication.md#migrating-from-usage-based-billing-api-key). If you're having trouble with login, please comment on [this issue](https://github.com/openai/codex/issues/1243).

### Model Context Protocol (MCP)

Codex CLI supports [MCP servers](./docs/advanced.md#model-context-protocol-mcp). Enable by adding an `mcp_servers` section to your `~/.codex/config.toml`.

### Configuration

Codex CLI supports a rich set of configuration options, with preferences stored in `~/.codex/config.toml`. For full configuration options, see [Configuration](./docs/config.md).

---

### Docs & FAQ

- [**Getting started**](./docs/getting-started.md)
  - [CLI usage](./docs/getting-started.md#cli-usage)
  - [Running with a prompt as input](./docs/getting-started.md#running-with-a-prompt-as-input)
  - [Example prompts](./docs/getting-started.md#example-prompts)
  - [Memory with AGENTS.md](./docs/getting-started.md#memory-with-agentsmd)
  - [Configuration](./docs/config.md)
- [**Sandbox & approvals**](./docs/sandbox.md)
- [**Authentication**](./docs/authentication.md)
  - [Auth methods](./docs/authentication.md#forcing-a-specific-auth-method-advanced)
  - [Login on a "Headless" machine](./docs/authentication.md#connecting-on-a-headless-machine)
- [**Non-interactive mode**](./docs/exec.md)

### Auto-generated Reference (Pre-Beta)

Generated reference pages (CLI flags, config keys, event schemas) live under `docs/generated/`.
Run:

```bash
make docs-gen
```

CI enforces drift with `make docs-drift`. These pages are pre-beta and may change.

#### Headless parity & reliability (under the hood)

`codex exec` now mirrors the reliability of interactive runs by default. You do not need extra flags.

- Always‑on artifacts (written to `./.codex/runs/` unless overridden with `--summary-dir`):
  - `exec-<unix_ms>-events.ndjson`: one JSON event per line (includes `seq`, `run_id`).
  - `exec-<unix_ms>-summary.json`: compact summary (`schema_version`, `status`, `exit_code`, `duration_ms`, `event_count`, model/provider, `events_path`).
- Time budget & graceful stop: `--run-timeout-secs <n>` sends Interrupt, waits a short grace, then requests Shutdown (exit code `5`).
- Helpful stderr hints on failure plus pointers to the artifacts for fast debugging.
- Advanced knobs (optional; defaults are already reliable):
  - `--force-cli-source` (attribute run as CLI for upstream parity telemetry)
  - `--keep-approval-policy` (do not force `AskForApproval::Never`)
  - `--shutdown-grace-ms` (tune grace after timeout; default 800ms)
  - `--seed <u64>` (persisted; foundation for deterministic sampling)
- [**Advanced**](./docs/advanced.md)
  - [Non-interactive / CI mode](./docs/advanced.md#non-interactive--ci-mode)
  - [Tracing / verbose logging](./docs/advanced.md#tracing--verbose-logging)
  - [Model Context Protocol (MCP)](./docs/advanced.md#model-context-protocol-mcp)
- [**Zero data retention (ZDR)**](./docs/zdr.md)
- [**Contributing**](./docs/contributing.md)
- [**Install & build**](./docs/install.md)
  - [System Requirements](./docs/install.md#system-requirements)
  - [DotSlash](./docs/install.md#dotslash)
  - [Build from source](./docs/install.md#build-from-source)
- [**FAQ**](./docs/faq.md)
- [**Open source fund**](./docs/open-source-fund.md)

---

## License

This repository is licensed under the [Apache-2.0 License](LICENSE).

---

## Build and Test (repo root)

This fork adds a simple end‑to‑end workflow at the repo root to quickly produce binaries with a working `config.toml` and verify post‑compile behavior:

- `make build` — compile the Rust CLI (`codex-rs/cli`) in release and copy the binary to `dist/bin/codex`.
- `make config` — create a minimal `dist/config/config.toml` for local testing; tests use this via `CODEX_HOME` so your user config isn’t touched.
- `make package` — runs both of the above.
- `make test` — runs deterministic, offline tests in `tests/` against the compiled binary with `CODEX_HOME=dist/config`.
- `make scenarios` — runs live, post‑compile scenarios in `scenarios/`. The included login round‑trip doesn’t require network, but additional scenarios can be added here.
- `make verify` — runs deterministic tests, and runs live scenarios when `RUN_LIVE=1`.

Environment variables used by the Makefile/tests:

- `CODEX_BIN` — path to the compiled binary (defaults to `dist/bin/codex`).
- `CODEX_HOME` — config and auth directory (tests default to `dist/config`).
- `RUN_LIVE=1` — opt‑in to execute live scenarios via `make verify`.

For Rust workspace development, continue to use `codex-rs/justfile` for formatting and clippy (`just fmt`, `just fix -p <crate>`), following the conventions in `AGENTS.md`.

### Binary name and alias

- Canonical name inside the repo: `codex` (binary at `dist/bin/codex`).
- Public alias: `cxplus` — a symlink that always points to the currently selected `codex`.
- Install a user‑level link without touching system binaries: `make install-local` (creates `~/.local/bin/cxplus`).
- Your existing shell alias (e.g., `alias cx=cxplus`) continues to work and won’t be overridden by the repo.

### Rapid deployment & versioning

- `make release` → produces a stamped binary `dist/releases/codex-<YYYYMMDDHHMM>-<branch>-<sha>` and updates `dist/bin/codex` + `dist/bin/cxplus`.
- `make list-releases` → lists available stamped binaries.
- `make switch VERSION=<stamp>` → switches `dist/bin/codex` (and thus `cxplus`) to an older/newer stamped build.
- `make rollback` → automatically points to the previously used stamped build.

#### Windows packaging

- `make package-windows` creates `dist/cxplus-windows.zip` containing:
  - `cxplus.cmd` and `cxplus.ps1` (wrappers that invoke `codex.exe` if present, otherwise `codex`)
  - `codex`/`codex.exe` (if available on the build host)
- On Windows, place `cxplus.cmd` (or `cxplus.ps1`) on your `PATH` to use the CLI via the `cxplus` name. The canonical executable name remains `codex.exe`.

### Chutes integration (auto‑discovery)

- Place credentials in `.env`:
  - `CHUTES_API_KEY` (required)
  - `CHUTES_API_BASE` (optional; defaults to `https://llm.chutes.ai/v1`)
  - `CHUTES_CATALOG_BASE` (optional; defaults to `https://api.chutes.ai/chutes/`)
- The build emits `dist/config/config.toml` with a `chutes` provider (wire_api defaults to `chat`).
- CLI:
  - `codex chutes recommend` → prints `openai/<catalog_id>` for the cheapest multi‑modal model ≥70B.
  - `codex chutes exec --json "Say hello"` → runs exec via Chutes; supports `--wire-api chat|responses` and `--images ...`.
- Live tests exercise the compiled CLI subcommand when `CHUTES_API_KEY` is present.

Details: docs/chutes.md

---

### Quick start and features index

- See QUICKSTART.md for a 60‑second build, test, and live‑scenario walkthrough.
- See FEATURES.md for an overview table of major CLI/TUI features, exec parity & reliability, discovery, scenarios, and safety controls.
- See docs/SCILLM_LOCAL.md for using a local scillm (litellm) checkout from downstream Python projects.

### Brand assets (SVG/PNG)

- Animated wordmark: `codex-rs/logo.svg` (static base letters, strong “+” pop‑in/out, 5s idle pause)
- Theme variants (animated backgrounds): `codex-rs/logo-dark.svg`, `codex-rs/logo-light.svg`
- Static snapshots: `codex-rs/logo-dark-static.svg`, `codex-rs/logo-light-static.svg`
- PNG exports (720×160): `codex-rs/logo-dark.png`, `codex-rs/logo-light.png`

Embed examples:

```html
<img src="./codex-rs/logo.svg" alt="cxplus" />
<img src="./codex-rs/logo-dark-static.svg" alt="cxplus" />
```

The accent color is themeable; when inlining the SVG, set `style="--accent:#FF4DDE"` on the `<svg>` element to override the cyan accent. Animations honor `prefers-reduced-motion` and `data-static="true"`.

### Knowledge‑First context (RFC, experimental)

cxplus is moving to a Knowledge‑First architecture that builds prompts from compact, cited evidence stored in ArangoDB via the memory‑agent MCP, rather than maintaining large in‑memory chat transcripts. This is designed to cut prompt tokens while improving determinism and traceability.

- Status: RFC/draft (experimental; gated behind a provider switch)
- Design: docs/feature_recipes/knowledge-first-context.md
- Impact: tiny recent chat window, structured evidence with citations, deterministic prompt sections

This does not change the quickstart flow or Makefile targets. When the feature ships, it will be enabled profile‑by‑profile with safe fallbacks.

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

Artifacts: when enabled, a single `context.summary` (version=2) line is written once per run to `*-events.ndjson` after context assembly and before streaming. It includes provider, quotas, max token budget, and retrieval metrics (durations, item counts, per‑section token usage, truncation flags). No raw evidence is logged.
