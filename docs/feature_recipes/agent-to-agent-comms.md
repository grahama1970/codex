# Agent‑to‑Agent Communications (Feature Recipe)

This recipe shows how two agents cooperate using cxplus (our Codex fork) with a clean submit/event model, live notifications, and reproducible artifacts. It’s written so other project agents can copy the pattern.

## TL;DR
- Pattern: one agent (Requester) opens a task and streams events; another agent (Reviewer/Operator) reacts to events and posts results back (e.g., code review, acceptance, or follow‑up tasks).
- Transport: Submission Queue (SQ) and Event Queue (EQ) over the local filesystem (NDJSON) and GitHub PR comments for long‑running review tasks.
- Outcome: fully asynchronous flow with desktop/Slack notifications, deterministic artifacts, and simple hand‑off between agents.

## Actors
- Requester Agent: invokes cxplus to perform work (e.g., code changes or “request a PR review”).
- Reviewer Agent: monitors events (NDJSON or PR comments), runs checks, and posts results (suggested diffs, accept/reject, clarifying questions).

## Prerequisites
- Build: `make package` (writes `dist/bin/codex`).
- Minimal config: `make config` (writes `dist/config/config.toml`).
- Optional (for PR reviews): enable Copilot auto‑review ruleset; see `docs/copilot-auto-review.md`.
- Optional notifications: set `SLACK_WEBHOOK` if you want Slack alerts.

## One‑Command Demo (Local, no network)
Run a deterministic, file‑based exchange using the compiled binary. This simulates Agent A requesting work and Agent B consuming events.

```
# Terminal A (Requester): open a task and stream events
CONTEXT_FEATURE=1 dist/bin/codex exec "Summarize this repository and list 3 tests to add"

# Observe artifacts written under .codex/runs/<run-id>-events.ndjson
```

```
# Terminal B (Reviewer): tail the Event Queue (EQ) and react to events
run=$(ls -t .codex/runs/*-events.ndjson | head -n1)
rg -n "^" "$run" | sed -n '1,20p'   # peek first events
rg -n '"kind":"context.summary"' "$run"  # verify metrics line v2
```

What you’ll see:
- Exactly one `context.summary` line before any model output:

```json
{"kind":"context.summary","version":2,"provider":"Minimal","max_context_tokens":4096,
 "budget":{"recent_pct":15,"plan_pct":10,"evidence_pct":60,"tools_pct":15},
 "retrieval_ms":0,"evidence_items":0,
 "total_tokens":512,
 "section_tokens":{"evidence":0,"plan":32,"recent":128,"tools":96}}
```

## GitHub PR Review as Agent‑to‑Agent
Use GitHub PR comments as the long‑running EQ for review tasks. Requester opens/updates a PR; Reviewer (Copilot Code Review) posts findings and suggested diffs; a local watcher notifies you.

- Submit (Requester):
```
# From a feature branch with changes:
make review-submit-async
# Starts a background watcher; writes results to local/copilot_pr<PR>_review.txt
```

- Watch (Operator):
```
# Manually start/stop if needed
nohup bash -c 'while true; do make copilot-watch; sleep 30; done' &
make review-stop-watch PR=4
```

- Results: `local/copilot_pr<PR>_review.txt` captures Copilot’s comments so downstream agents can parse and act (e.g., apply small diffs, open follow‑ups).

## Event Flow (SQ/EQ) Cheat‑Sheet
- Submit Turn (SQ):
  - `codex exec "…"` emits `UserMessage` → `AgentMessage`/tool/MCP events → `TaskComplete`.
- Deterministic Context Line:
  - `context.summary` (v2) is emitted once before streaming; contains provider, budgets, tokens, truncation flags.
- Long‑running Review (EQ):
  - PR comments act as an out‑of‑band EQ; the watcher translates comments into local files + notifications.

## Repro Steps (Copy/Paste Ready)
1) Build and config:
```
make package
```
2) Open PR and watch asynchronously:
```
# On your working branch
make review-submit-async
# Optional: Slack webhook
export SLACK_WEBHOOK="https://hooks.slack.com/services/..."
```
3) Act on findings:
```
# Summarize Copilot TODOs
jq -r '.[] | select(.user.login|test("copilot";"i")) | [.path, (.position // .original_position // 0), (.body|gsub("\r";""))] | @tsv' \
  local/pr*_review_comments.json | column -t -s$'\t'
```
4) Stop the watcher:
```
make review-stop-watch PR=<number>
```

## Code Touchpoints (for Maintainers)
- Exec runtime & events: `codex-rs/exec/src/lib.rs` (submits, streams, emits `context.summary`).
- Context providers: `codex-rs/context/src/lib.rs`, `codex-rs/context/src/retrieval.rs` (Minimal/Arango; fixture‑friendly retrieval).
- TUI affordances: `codex-rs/tui/src/{chatwidget.rs, slash_command.rs, history_cell.rs}`.
- PR review automation: `scripts/copilot_submit_async.sh`, `scripts/copilot_watch.sh`, `scripts/notify.sh`; Make targets: `review-submit-async`, `review-stop-watch`, `copilot-watch`.

## Extending to Tool/MCP Exchanges
- Add MCP servers to your config; Agent emits tool call events and ingests results.
- Keep tools deterministic (bounded timeouts; stable outputs) to preserve reproducibility and reliable `context.summary` metrics.

## Production Tips
- Always capture artifacts (NDJSON + summarized results) into a job folder so CI can archive them.
- Prefer stable providers and fixture-driven retrieval in CI (set `CONTEXT_MCP_FIXTURE` for deterministic tests).
- Keep instructions in `.github/copilot-instructions.md` concise; use `REVIEW_REQUEST.md` for per‑PR details.

---

With this pattern, any project agent can request work from another agent, observe progress via events, and act on results (including PR reviews) without blocking or context rot.
