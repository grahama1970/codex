# Copilot Code Request: Implement Knowledge‑First Context (Phase‑0), Validate, and Propose Diffs

- Fork: grahama1970/codex
- Branch: feat/chutes-profiles-scenarios
- Path: git@github.com:grahama1970/codex.git#feat/chutes-profiles-scenarios

You are reviewing a fork of OpenAI’s Codex CLI ("cxplus"). We want you to produce concrete, unified diffs to land Phase‑0 of a Knowledge‑First context system and validate the approach end‑to‑end. Please read the referenced files and deliver structured diffs only for the areas requested below.

Context you should read first
- README.md (project overview and rationale additions)
- FEATURES.md (feature map + Knowledge‑First section)
- QUICKSTART.md (Makefile pipeline & scenarios)
- docs/feature_recipes/knowledge-first-context.md (design & acceptance criteria)
- Makefile (package/test/scenarios/verify) and tests/scenarios folders

Why this matters
- cxplus is used to drive automation in our projects (e.g., scillm/litellm). Our pain point is “context rot” from long chat transcripts. We want database/tool‑first retrieval (via memory‑agent MCP + ArangoDB), compact evidence bundles, and a tiny recent chat window — while preserving deterministic builds, post‑compile validation, and multi‑model routing (Chutes).

Acceptance criteria (Phase‑0)
1) Add a new crate `codex-rs/context` with:
   - `TurnInput`, `EvidenceBundle`, `ContextProvider` trait
   - `MinimalContextProvider` (existing behavior, small recent window only)
   - `ArangoContextProvider` stub that calls memory‑agent MCP (no heavy logic yet — just request/response shape + error handling)
   - `TokenBudgeter` (simple allocator that enforces section quotas and trims; unit‑tested)
2) Config surface (parsed but default‑off):
   - `~/.codex/config.toml`: `[context] provider = "minimal"|"arango"`, `max_context_tokens`, `[context.budget] recent_pct/plan_pct/evidence_pct/tools_pct`, `[context.arango] endpoint/database/mcp_tool`
   - Env toggles: `CONTEXT_FORCE_MINIMAL`, `CONTEXT_DEBUG`
3) Wire into prompt assembly (guarded):
   - In `codex-core` (prompt builder), if provider=arango → call `ContextProvider` to build an `EvidenceBundle`, then assemble the prompt sections in a deterministic order. Else fallback to minimal provider.
   - Emit a `context.summary` NDJSON event with selected counts/tokens per section.
4) Tests:
   - Unit: budgeter trims and respects quotas; evidence shaping never exceeds budget; serialization sanity
   - Integration (fixture): MCP stub responses (no network) and deterministic golden prompt
   - Scenario skeleton: comparisons of token counts vs baseline (skipped unless `CONTEXT_FEATURE=1`)
5) Docs: add a short “How to enable Knowledge‑First” subsection pointing to the config keys (do not change quickstart commands).

Constraints
- Do not change existing Makefile targets or CLI UX.
- Keep the change behind config (default provider stays `minimal` for now).
- Follow repo conventions (see AGENTS.md). After Rust edits, we run `just fmt` and targeted `just fix -p <crate>`, then `make test` and `RUN_LIVE=1 make verify` for scenarios.

Where to touch (relative paths)
- New crate: `codex-rs/context/` (Cargo.toml + src/lib.rs + tests)
- Config parsing: `codex-rs/core/src/config.rs` (new `[context]` section + env overrides)
- Prompt assembly hook: locate current prompt builder in `codex-rs/core` (search for where user/system/tool messages are assembled) and gate the call to `ContextProvider` behind config.
- Event emission: `codex-rs/exec/src/lib.rs` or the core event stream path — add a `context.summary` event type with tokens per section + elapsed_ms (keep schema small).
- Tests: new unit tests under `codex-rs/context/tests/` and minimal integration test under `codex-rs/core/tests/` (fixture‑driven)
- Scenario: add `scenarios/test_context_budget.py` (skipped unless `CONTEXT_FEATURE=1`)
- Docs: minimal additions to `README.md`/`FEATURES.md` pointing to the RFC and how to enable.

Scenarios already present (use to validate end‑to‑end)
- `make package` → compile
- `make test` → deterministic tests
- `RUN_LIVE=1 make verify` → live scenarios (will remain unchanged)

Clarifying questions (please answer inline before diffs):
1) Best insertion point for prompt assembly hook in `codex-core` — confirm the function(s) you’ll modify.
2) Preferred serialization for `context.summary` — propose a minimal shape that won’t require migrations later.
3) MCP message shapes you expect for `memory.search` / `memory.neighbors` — provide request/response structs you will use.
4) Any objections to the default quotas (recent 15%, plan 10%, evidence 60%, tools 15%)? If yes, propose initial values.

Deliverables
- Unified diffs only, grouped by concern (Context crate, Config, Core hook, Events, Tests, Scenario, Docs). Keep changes minimal and focused. Include help text and config comments as needed.

