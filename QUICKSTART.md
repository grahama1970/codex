# Quickstart

This fork adds a fast path to build a release binary, generate a minimal config, and run both deterministic tests and live, post‑compile scenarios. You can install a user‑level alias (`cxplus`) without touching any system binaries.

## 0) Prereqs
- Rust toolchain (we test with `RUSTUP_TOOLCHAIN=1.90.0`)
- Optional helpers: `just`, `rg`, `cargo-insta` (`make rust-prepare` will suggest/install)

## 1) Build and package
```
make package
```
Outputs:
- `dist/bin/codex` (canonical binary)
- `dist/config/config.toml` (minimal config for tests)

## 2) Run deterministic tests (offline)
```
RUSTUP_TOOLCHAIN=1.90.0 make test
```

## 3) Run live scenarios (post‑compile)
```
RUSTUP_TOOLCHAIN=1.90.0 make scenarios
```
Notes:
- Live scenarios use `.env` if present. For Chutes discovery, set `CHUTES_API_KEY` and optionally `CHUTES_API_BASE`.
- A deterministic fixture mode exists for discovery: set `CHUTES_CATALOG_FIXTURE=/absolute/path.json`.

## 4) Rapid deploy & versioning
Create a stamped release and update the active binary + alias:
```
make release
```
Artifacts:
- `dist/releases/codex-<YYYYMMDDHHMM>-<branch>-<sha>`
- `dist/bin/codex` → symlink to the stamped binary
- `dist/bin/cxplus` → symlink to `codex`

Switch / rollback:
```
make list-releases
make switch VERSION=<stamp>
make rollback
```

## 5) Install a user‑level alias (safe)
```
make install-local   # creates ~/.local/bin/cxplus -> dist/bin/cxplus
```
Then add to your shell:
```
alias cx=cxplus
```

## 6) Chutes quick check
```
export CHUTES_API_KEY=...            # or set via .env
dist/bin/cxplus chutes recommend     # prints openai/<model-id>
dist/bin/cxplus chutes exec --json "Say hello"
```

See also: docs/chutes.md, docs/slash-commands.md.

