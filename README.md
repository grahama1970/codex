<p align="center"><code>npm i -g @openai/codex</code><br />or <code>brew install codex</code></p>

<p align="center"><strong>Codex CLI</strong> is a coding agent from OpenAI that runs locally on your computer.
</br>
</br>If you want Codex in your code editor (VS Code, Cursor, Windsurf), <a href="https://developers.openai.com/codex/ide">install in your IDE</a>
</br>If you are looking for the <em>cloud-based agent</em> from OpenAI, <strong>Codex Web</strong>, go to <a href="https://chatgpt.com/codex">chatgpt.com/codex</a></p>

<p align="center">
  <img src="./.github/codex-cli-splash.png" alt="Codex CLI splash" width="80%" />
  </p>

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
- See FEATURES.md for an overview table of major CLI/TUI features, discovery, scenarios, and safety controls.
- See docs/SCILLM_LOCAL.md for using a local scillm (litellm) checkout from downstream Python projects.
