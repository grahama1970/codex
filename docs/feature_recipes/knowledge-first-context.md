# Feature Recipe: Knowledge‑First Context Orchestration

Status: Draft (RFC‑0)

Owner: cxplus

Goal: Make cxplus “Knowledge‑First” by default so we fetch compact, cited evidence from ArangoDB (via memory‑agent MCP) before calling an LLM, reducing volatile in‑memory chat context and token usage.

## Problem

- Rolling transcripts bloat prompt tokens and require constant lossy summarization.
- We already persist durable knowledge (facts, procedures, episodic artifacts) in ArangoDB but under‑use it during prompt assembly.

## Outcomes (Acceptance Criteria)

- ≥60% reduction in average prompt tokens for real tasks vs chat‑first baseline.
- Deterministic, schema‑first prompt sections with citations to Arango nodes.
- Transcripts kept to a tiny sliding window (≤3 recent user/assistant turns).
- Graceful degradation: if Arango/MCP is unavailable, run with minimal context + explicit warning.

## Non‑Goals

- Replacing ArangoDB or the memory‑agent MCP.
- Building an RAG vector index inside cxplus (we rely on memory‑agent MCP for retrieval).

## Architecture

New crate: `codex-rs/context`.

- `trait ContextProvider { fn build_bundle(&self, input: TurnInput) -> EvidenceBundle }`
- `ArangoContextProvider` (default) talks to memory‑agent MCP.
- `TokenBudgeter` allocates/trims tokens per section.

Prompt sections (fixed order):

1) Recent turns (≤3 exchanges)
2) Task state (plan, cwd, flags)
3) Evidence (facts, procedures, episodic artifacts) with citations
4) Tool deltas (diffs/outputs since last turn)

Token quotas (configurable): recent 15%, plan 10%, evidence 60%, tools 15%; unused quota reflows.

## Retrieval Strategy (via memory‑agent MCP)

1) Cheap intent parsing → tags and query terms (on‑device or current model with small budget)
2) `memory.search { q, k, within_days, types }` → candidate nodes
3) `memory.neighbors { id, depth, types }` → enrich with local graph
4) Dedupe/score by novelty, authority, freshness; then trim to budget

## Config (new)

`~/.codex/config.toml`

```
[context]
provider = "arango"            # or "minimal"
max_context_tokens = 8192

[context.budget]
recent_pct = 15
plan_pct = 10
evidence_pct = 60
tools_pct = 15

[context.arango]
endpoint = "http://localhost:8529"
database = "codex"
collection_facts = "facts"
collection_procedures = "procedures"
collection_episodes = "episodes"
# mcp tool id providing memory.* endpoints
mcp_tool = "memory-agent"
```

Env toggles:
- `CONTEXT_FORCE_MINIMAL=1` → bypass Arango, minimal context only
- `CONTEXT_DEBUG=1` → print inclusion/trim decisions to stderr

## Wire Points

- `codex-core`: prompt builder calls `ContextProvider` before LLM request.
- `exec`/`tui`/`app-server`: pass `TurnInput` (intent, cwd, changed files, policy) + budget.

## Testing

Unit:
- `TokenBudgeter` trims by importance/novelty; respects quotas and reflow.
- Evidence shaping keeps citations, never exceeds budget.

Integration (fixtures):
- Offline Arango fixtures exercised via MCP stubs; stable golden prompts.
- Delta context: only new evidence appears after a file change.

Scenarios (optional live):
- `scenarios/test_context_budget.py` ensures token savings vs baseline for a known task.

## Telemetry

- Emit `context.summary` event per turn: selected_nodes, tokens_per_section, trimmed_counts, elapsed_ms.

## Rollout

Phase 0 (behind flag/profile):
- Implement crate + config + tests; default provider remains “minimal”.

Phase 1 (default on for exec):
- Switch default provider to `arango` for `exec`; TUI opt‑in.

Phase 2:
- Remove transcript compaction; keep ≤3 turns; migrate old logs into durable summaries.

## Risks & Mitigations

- Arango/MCP latency → cache first‑hop results; set 8s timeout + partial results.
- Over‑trimming relevant facts → feedback loop using tool outcomes to boost missed nodes next turn.

## Milestones

1) Crate + minimal provider + tests (2–3 days)
2) MCP adapter + fixtures + scenario (2–3 days)
3) Wire into exec default + docs (1–2 days)

