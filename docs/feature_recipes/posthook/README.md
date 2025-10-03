# Post‑Hook Feature Recipe (Design)

Goal
- Run a fast, non‑blocking post‑execution step after each model turn (and later after apply) to capture signal, enrich memory safely, and produce actionable telemetry without impacting user latency.

Constraints
- Budget ≤ 800 ms total; no retries (caller never waits beyond budget).
- Read‑only by default; writes only when explicitly enabled and scope allow‑listed.
- No payload logging; one structured line per decision/latency.
- Heavy work (research, clustering, reporting) is offloaded asynchronously via Agent Bus.

Core Use‑Cases (pick one or combine)
- Outcome logging (episodes)
  - Log a compact episode with thread_id/repo/outcome/why for later analysis.
  - Enabled only when MEMORY_AGENT_ALLOW_WRITE=1 and scope allow‑listed.
- Knowledge indexing (light)
  - Upsert a short summary/keywords for future recall (opt‑in; scope allow‑list).
  - Keep writes < 500 ms; any failure → no‑op.
- Quality/reporting
  - Emit a signed /audit/event via Agent Bus (HMAC + X‑Idempotency‑Key) with outcome + small metrics for dashboards.
  - No sensitive payload; receiver performs dedup by idempotency key.
- Async research follow‑up
  - If prehook confidence was low or the turn failed, return ask/plan and trigger repository_dispatch or queue with a background “research plan” (arXiv/YouTube). Results populate Memory and notify /notify.

Wiring (MCP + Agent Bus)
- MCP posthook tool: `codex.posthook.record` (read‑only default)
  - input: { thread_id?, repo?, scope?, cwd?, last_user_msg?, summary?, outcome?, token_usage?, latency_ms? }
  - timeout: 700 ms; returns `{ ok: true, meta:{version,latency_ms,wrote?:bool} }`.
- cx‑plus posthook (env‑gated in MVP)
  - `CODEX_POSTHOOK_ENABLED=1`
  - `CODEX_POSTHOOK_MCP_SERVER='stdio:/usr/local/bin/memory-mcp'`
  - `CODEX_POSTHOOK_MCP_TOOL='codex.posthook.record'`
  - `CODEX_POSTHOOK_TIMEOUT_MS=800`
- Agent Bus
  - Optional: send `/audit/event` with {repo,thread_id,outcome,latency_ms} signed (HMAC) and with idempotency key.

Latency Discipline
- Always set connect/call budgets (e.g., 650/700) and never retry in server.
- If the server is down or non‑responsive → exit silently; user experience never blocked.

Security & Privacy
- Read‑only by default; writes require MEMORY_AGENT_ALLOW_WRITE=1 and scope allow‑list (MEMORY_AGENT_WRITE_SCOPES).
- Redact obvious secret patterns; strip URL queries in any text fields.
- No payload logging; structured logs only: {decision,latency_ms,version,trace_id}.

Scenarios (live)
- `scripts/scenarios/headless_exec.sh` — ensures prehook augment appears safely.
- `scripts/scenarios/queue_example.sh` — demonstrates async task routing for research plans.
- Add (optional) `scripts/scenarios/posthook_audit.sh` — emit a signed /audit/event via Agent Bus in response to a completed turn.

Deterministic Tests (no network)
- Posthook disabled: ensure no effects.
- Posthook enabled with stub MCP: call returns <800 ms; errors do not block.
- Episode logging only when write flags/scopes allow; otherwise no‑op.

Rollout Plan
- Phase 1: enable posthook read‑only; only emit /audit/event; no writes.
- Phase 2: enable episode logging for a narrow scope; watch latency + error budget.
- Phase 3: introduce async research plan mode for low‑confidence turns via Agent Bus.

