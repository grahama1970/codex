 Title: cxplus Review Context (CodeRabbit + Mailbox)

  Overview

  - cxplus is a terminal‑first, policy‑hardened, deterministic CLI.
  - Key edges: Knowledge‑First pre‑hook (versioned context.summary
    events), strict local‑only posture, seeded determinism parity across
    Chat/Responses, async PR review via mailbox, snapshot‑tested TUI.
  - This document explains how to run CodeRabbit from the CLI, capture
    results, render unified diffs, and route everything through the
    mailbox so I can watch and summarize.

  What’s Installed

  - CodeRabbit CLI is already installed.
  - Your token is set in /home/graham/.codex/config.toml; the adapter
    accepts CODERABBIT_TOKEN from the environment.

  Review Backends

  - Default: GitHub Copilot (web request + watcher).
  - Optional: CodeRabbit CLI for deep PR diffs (adapter + diff renderer
    included).
  - We do not rebuild a full review engine; we orchestrate backends and
    keep runs deterministic and auditable.

  Local‑Only Policy

  - Enforced: CODEX_LOCAL_ONLY=1 disables proxies and denies non‑local
    providers with a WARN + fatal on both Responses and Chat paths.
  - Context crate mirrors .no_proxy() so retrieval cannot leak via
    proxies.
  - To run CodeRabbit, ensure network egress is allowed:
      - unset CODEX_LOCAL_ONLY

  Determinism Contract

  - With deterministic_seed set, Responses and Chat clamp temperature=0.0,
    top_p=1.0 and neutralize penalties/top_k/typical/logit_bias.
  - We emit a single, pre‑stream context.summary v2 line with:
    provider, max_context_tokens, budget., retrieval_ms, evidence_items,
    search_k, neighbors_depth, retry_count, cache_hit, fallback_reason,
    reflowed_from., section_tokens., truncated., total_tokens.

  Knowledge‑First Context (pre‑hook)

  - Retrieval via memory‑agent/Arango runs before model streaming.
  - The shaped metrics and budgets are recorded in context.summary v2
    (NDJSON).
  - Use fixtures for deterministic runs where possible.

  CLI Review Loop (Copilot Web)

  - Build/paste (manual): make copilot-prompt-send OUT=local/
    copilot_prompt.txt SEND=0 BROWSER=safari URL=https://github.com/
    copilot
  - Auto‑send: make copilot-prompt-send OUT=local/copilot_prompt.txt
    SEND=1 BROWSER=safari URL=https://github.com/copilot
  - Wait/stabilize: make copilot-web-wait OUT=local/copilot_review.txt
    BROWSER=safari URL=https://github.com/copilot INTERVAL=1 STABLE=3
    MAX=90
  - Process: make copilot-process-review IN=local/copilot_review.txt
  - Mailbox: make mailbox-append BODY="$(cat local/copilot_review.txt)"
    CHANNEL=reviews PRIO=5 TTL=3600
  - Watcher: make mailbox-watch MAILBOX=.codex/mailbox.jsonl

  CodeRabbit Flow (Primary for diffs)

  - Token (if needed): export CODERABBIT_TOKEN="$(rg -No
    'coderabbit_token\s*=\s*"(.)"' /home/graham/.codex/config.toml | sed
    -E 's/.="(.*)"/\1/')"
  - Submit review:
      - make coderabbit-review REPO=owner/repo PR=123 OUT=local/
        coderabbit_review.jsonl
      - Honors local‑only: denied if CODEX_LOCAL_ONLY=1
      - Produces normalized mailbox JSONL (run_id/status/tags/body)
  - Render unified diffs:
      - bash scripts/cx_review_diff.sh --in local/coderabbit_review.jsonl
  - Route into shared mailbox:
      - cat local/coderabbit_review.jsonl >> .codex/mailbox.jsonl
      - make mailbox-watch MAILBOX=.codex/mailbox.jsonl
        ONLY_STATUS=processed

  Environment Variables

  - CODEX_LOCAL_ONLY=1: no_proxy + fail‑closed on non‑local providers
    (Chat + Responses).
  - RUN_ID: set to group mailbox items (optional). Example:
    RUN_ID=ci-$(date +%s)
  - TAGS: comma‑separated tags stored in mailbox JSONL for routing.
    Example: TAGS="coderabbit,pr-123"

  Key Files

  - Policy + Clients:
      - codex-rs/core/src/default_client.rs
      - codex-rs/core/src/client.rs
  - Chat Determinism (test helper + unit test):
      - codex-rs/core/src/chat_completions.rs
  - Knowledge‑First Retrieval:
      - codex-rs/context/src/lib.rs
      - codex-rs/context/src/retrieval.rs
  - Exec Event Emission:
      - codex-rs/exec/src/lib.rs
  - Mailbox + Tools:
      - scripts/mailbox_append.sh
      - scripts/mailbox_watch.sh
      - scripts/review_backend_coderabbit.sh
      - scripts/cx_review_diff.sh
  - Docs:
      - QUICKSTART.md (CLI Review Loop)
      - FEATURES.md (HTTP client policy)
      - docs/generated/events/context-summary-v2.json (schema)

  What I’ll Do On Restart

  - Tail .codex/mailbox.jsonl (ONLY_STATUS=processed when set), summarize
    findings into an action‑ready TODO list with file:line anchors, and
    apply safe diffs.
  - For CodeRabbit runs, parse local/coderabbit_review.jsonl and render
    unified diffs via scripts/cx_review_diff.sh before proposing a patch.

  Safety and House Rules

  - Do not modify CODEX_SANDBOX_* checks; they are intentional for test
    early‑exit/sandbox semantics.
  - Keep diffs surgical; prefer inlined format! args, collapsed ifs, and
    method references per clippy rules.
  - TUI styling: use ratatui Stylize helpers (e.g., .dim(), .red()); avoid
    hardcoded white.

  Quick Start (CodeRabbit)

  - unset CODEX_LOCAL_ONLY
  - export CODERABBIT_TOKEN="…"
  - make coderabbit-review REPO=owner/repo PR=123 OUT=local/
    coderabbit_review.jsonl
  - bash scripts/cx_review_diff.sh --in local/coderabbit_review.jsonl
  - cat local/coderabbit_review.jsonl >> .codex/mailbox.jsonl
  - make mailbox-watch MAILBOX=.codex/mailbox.jsonl ONLY_STATUS=processed