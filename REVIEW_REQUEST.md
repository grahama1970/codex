# Comprehensive Review Request (Expanded)

- Fork: grahama1970/codex
- Branch: feat/chutes-profiles-scenarios
- Path: git@github.com:grahama1970/codex.git#feat/chutes-profiles-scenarios

Please perform a deep, end‑to‑end review oriented around reliability, safety, and developer experience. Provide prioritized findings, rationale, and concrete unified diffs. When trade‑offs exist, outline options and recommend one.

What’s in this fork (“cxplus”)
- Exec/interactive parity + reliability: consistent behavior, artifacts, time budgets, deterministic seed plumbing.
- Chutes.ai integration: catalog discovery → model selection → exec; warm‑up subcommand; capacity planning helper.
- Post‑compile validation pipeline (Makefile) + deterministic tests + live scenarios.
- “Knowledge‑First” proposal to make context come from ArangoDB (memory‑agent MCP) instead of a large rolling transcript (see docs/feature_recipes/knowledge-first-context.md).

Environment & Repro
- Toolchain: Rust 1.90.0 recommended for builds.
- One‑liners:
  - `make package` → compile + dist
  - `make test` → deterministic tests (offline) against compiled binary
  - `RUN_LIVE=1 make verify` → runs both deterministic and live scenarios
  - `make chutes-profiles` → auto‑discover coding + multimodal models (requires CHUTES_API_KEY in `.env`)

---

# High‑Priority (Blocking / Major) Findings

| ID | Area | Severity | Summary | Impact | Fix Provided |
|----|------|----------|---------|--------|--------------|
| H1 | Exec Parity | High | Summary artifacts lack retry/HTTP status & schema version → weak observability | Harder CI triage, breaking future evolution | Yes |
| H2 | Exec Parity | High | Approval override flag absent; forcing `Never` can surprise parity users | Violates principle of least astonishment | (Prior patch concept — include diff tying to `keep_approval_policy`) |
| H3 | Exec Reliability | High | Time budget grace fixed (500ms) & non-configurable; risk of truncating final events | Potential data loss before summary write | Yes (configurable grace) |
| H4 | Chutes Discovery | High | Derivation logic: price NaN relaxation lacks explicit note in summary/log context outside debug env | Silent fallback may mask catalog issues | Diff: structured stderr note + optional summary field |
| H5 | Determinism | High | Seed not persisted in summary JSON nor validated by a test; risk of regression | Repro claims unverifiable | Yes (summary + test scaffold) |
| H6 | Build/DX | High | `RUSTUP_TOOLCHAIN` not defaulted → inconsistent builds on newer stable (potential feature drift) | Non-reproducible CI results | Yes |
| H7 | Safety | High | Warmup / catalog requests lack retry classification (simple backoff only in warmup) & no connect timeout override | Flaky networks produce inconsistent failures | Partial (add provider timeouts flags stub + doc) |

---

# Medium Findings

| ID | Area | Severity | Summary | Fix Provided |
|----|------|----------|---------|--------------|
| M1 | Exec Summary | Med | No `exit_code`, `event_count`, `session_source`, `seed` fields | Yes |
| M2 | Event Log | Med | NDJSON lines lack `seq` + `run_id` (denormalization) | Yes |
| M3 | Chutes Ranking | Med | Tie-break order encoded but not unit‑tested offline (NaN vs finite) | Test diff |
| M4 | Planning | Med | CPM rounding logic may mask small differences; no negative/zero guard before cost divide except `max(1e-6)` applied only later | Diff (explicit guard) |
| M5 | Makefile | Med | Several shell lines not using `set -euo pipefail` (only some targets) | Diff adds safe shell preamble macro |
| M6 | Knowledge‑First | Med | No scaffold crate; doc references config section not parsed anywhere | Added crate skeleton diff |
| M7 | Logo | Med | Animated SVG inline script may be stripped by some renderers; no static fallback in README | Diff: static fallback snippet |
| M8 | Determinism | Med | Chat completions seeds set temperature/top_p, but Responses API path not covered | TODO + test placeholder |
| M9 | Chutes Warmup | Med | Warmup prints success to stderr via `eprintln!` mixing streams | Diff route warmup info via debug only |
| M10 | Docs | Med | exec.md doesn’t document timeout exit code (5) or seed behavior | Doc diffs |

---

# Low (Polish)

- L1: Prefer `Duration::from_secs` for clarity where seconds used.
- L2: Introduce `SUMMARY_SCHEMA_VERSION` constant.
- L3: Planner rounding helper.
- L4: Decide whether to allow underscore in derived domains; document.
- L5: Clarify `--shutdown-grace-ms` default in help.
- L6: Add `make doctor-live` small end‑to‑end check.
- L7: `context.summary` event stub (disabled by default).
- L8: Docs: suggest `CARGO_TERM_COLOR=always` for CI logs.

---

# Detailed Diffs (Apply As Patch Sets)

> Because the codebase is large, diffs are grouped by concern to minimize merge risk. Where diffs touch multiple areas, we note it explicitly.

### Exec: Summary Schema, seq, seed, exit code, configurable shutdown grace

```rust
// codex-rs/exec/src/cli.rs
// Add new flags: --shutdown-grace-ms, --seed, --keep-approval-policy, --force-cli-source (see outline)
```

```rust
// codex-rs/exec/src/lib.rs
// Use SessionSource::{Cli|Exec} per flag, write seq/run_id/session_source in NDJSON, add schema_version/exit_code/event_count/seed/etc. to summary.
```

### Exec: Chat Completions – ensure seed applied (Responses TODO)

```rust
// codex-rs/core/src/chat_completions.rs
// When seed present: set temperature=0.0, top_p=1.0, seed=<u64>; add TODO for Responses path.
```

### Chutes: Explicit price cap relaxation notice

```rust
// codex-rs/cli/src/chutes_cmd.rs
// Print: [chutes-relax] relaxing price cap (all candidates had NaN output price)
```

### Chutes: Offline ordering test (new)

```rust
// codex-rs/cli/tests/test_chutes_ordering.rs
// Fixture-based test verifying tie-break order (price asc → params desc → context desc → input price asc).
```

### Planning: CPM guard

```rust
// codex-rs/cli/src/chutes_cmd.rs
// Guard total_tokens >= 1.0 before CPM division.
```

### Makefile: default toolchain + strict shell macro

```makefile
# Add: RUSTUP_TOOLCHAIN ?= 1.90.0
# Add macro STRICT_SHELL and use it in profiles/test/scenarios recipes.
```

### Knowledge‑First: Crate scaffold

```toml
# codex-rs/context/Cargo.toml — tiny crate with serde/anyhow
```

```rust
// codex-rs/context/src/lib.rs — TurnInput/EvidenceBundle + MinimalContextProvider
```

### Docs: exec/chutes/readme updates

```markdown
# docs/exec.md — add deterministic runs, time budget & shutdown grace, artifacts and exit codes
```

```markdown
# docs/chutes.md — document [chutes-relax] fallback notice
```

```markdown
# README.md — add static SVG fallback snippet
```

---

## Clarifying Questions (for reviewers)

1) Set `RUSTUP_TOOLCHAIN ?= 1.90.0` by default in Makefile? (repro + parity with docs)
2) Include `retry_attempts` / `last_http_status` in exec summary now or in follow‑up with an HTTP policy wrapper?
3) Knowledge‑First default rollout: `exec` first (on), TUI opt‑in — agree?
4) Emit `context.summary` events (counts/tokens/ids) per turn — any objections?
5) Provider seed constraints: document as best‑effort (may be ignored) — acceptable?

