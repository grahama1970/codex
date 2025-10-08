# Knowledge‑First Context (Phase‑0 → Phase‑1)

This document defines the design, acceptance criteria, and rollout plan for Knowledge‑First context in cxplus.

Why: Reduce prompt size and drift by retrieving evidence from a memory system (via the memory‑agent MCP + ArangoDB) and strictly budgeting tokens across sections (recent, plan, evidence, tools).

## Goals

- Deterministic prompt assembly with section budgets
- Pluggable providers: `minimal` (existing behavior) and `arango` (MCP-backed)
- Small, stable `context.summary` event for observability

## Provider Surface

```toml
[context]
provider = "minimal" # or "arango"
max_context_tokens = 4096
[context.budget]
recent_pct = 15
plan_pct = 10
evidence_pct = 60
tools_pct = 15
[context.arango]
endpoint = "http://localhost:8529"
database = "codex"
mcp_tool = "memory-agent"
search_k = 16
neighbors_depth = 2
timeout_ms = 2000
max_evidence_items = 24
```

Env toggles:
- `CONTEXT_FORCE_MINIMAL=1`
- `CONTEXT_DEBUG=1`
- `CONTEXT_MCP_FIXTURE=/abs/path/catalog.json`

## Event: `context.summary`

Stable shape (v2) emitted before streaming begins (one line):

```json
{
  "kind": "context.summary",
  "version": 2,
  "provider": "Minimal|Arango",
  "max_context_tokens": 4096,
  "budget": {"recent_pct":15,"plan_pct":10,"evidence_pct":60,"tools_pct":15},
  "retrieval_ms": 0,
  "evidence_items": 0,
  "search_k": 16,
  "neighbors_depth": 2,
  "reflowed_from": {"plan":0,"recent":0,"tools":0},
  "total_tokens": 512,
  "section_tokens": {"evidence":256,"plan":32,"recent":128,"tools":96},
  "truncated": {"evidence":false,"plan":false,"recent":true,"tools":false}
}
```

## Acceptance Criteria (Phase‑0)

- Crate `codex-context` with `ContextProvider` trait and `MinimalContextProvider`, `ArangoContextProvider` (stubbed retrieval, error handling)
- Token budgeter enforces quotas and trims; unit tests
- Config surface parsed (default provider: minimal)
- Prompt assembly calls provider behind config gate
- Emit `context.summary` with token counts and retrieval metrics
- Scenario: compares token counts vs baseline; skipped unless `CONTEXT_FEATURE=1`

## Rollout Plan

1. Default `minimal`; ship behind config
2. Add fixtures and MCP stubs; validate budgeter
3. Enable `arango` per environment; expand scenarios
4. Collect metrics from `context.summary` lines; tighten budgets

