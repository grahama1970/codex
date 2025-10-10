# STATE_OF_PROJECT — cxplus competitive analysis

Repo/branch: grahama1970/codex@main

## Executive summary

- Verdict: ready for internal pilot with guardrails; not yet “broad internal default.”
- Exit codes: keep 5 for timeout/interrupted for now; revisit later behind a `--posix-exit-codes` flag.
  - Evidence: README.md:220–227; QUICKSTART.md:176–183.
- Determinism: enforce full neutralization with seed across Chat and Responses; artifacts + pre‑stream provenance are strong.
  - Evidence: Chat clamp and test: codex-rs/core/src/chat_completions.rs:291–299, 943–983; Responses clamp call: codex-rs/core/src/client.rs:279–284; artifacts/exit codes: README.md:112–122, 213–227.
- Local‑only posture: default true for CI/dev; remote providers (Runpod) only via profile or `-c` overrides with secrets.
  - Evidence: central no‑proxy/headers: codex-rs/core/src/default_client.rs:143–156; local‑only checks Chat: codex-rs/core/src/client.rs:136–153; Responses: 321–337, 351–364; README.md:64–71.
- Pre‑stream provenance: context.summary v2 emitted before streaming; CI asserts first‑line ordering.
  - Evidence: codex-rs/exec/src/lib.rs:447–538; .github/workflows/cxplus-verify.yml:24–27.
- Provider posture: Chutes documented; Runpod OpenAI‑compatible documented and optionally checked in CI.
  - Evidence: README.md:141–159, 184–207; .github/workflows/cxplus-verify.yml:40–60.
- TUI slash: useful helpers, read‑only defaults; unknown flags are strict parse errors.
  - Evidence: codex-rs/common/src/slash.rs:140–201, tests at 141–154.
- Risks: policy drift if any egress bypasses `create_client()`; reliance on external discovery (Chutes); determinism regression if new knobs appear.
- Mitigations: enforce single egress path (lint), add clamp tests for both wire APIs, keep Runpod healthcheck optional.

## Parity matrix (CLI‑only scope)

| Capability/Tool | cxplus | llm (Datasette) | Aider | LiteLLM router (CLI/proxy) | OpenHands | OpenDevin | Runpod (endpoint) |
|---|---|---|---|---|---|---|---|
| Determinism/artifacts | Strong: artifacts+seed+exit codes (docs+CI) | Default SQLite logging; `-o seed` for OpenAI | Chat/LLM history; no seed option documented | Logs/spend UI; not determinism‑oriented by default | Trajectory JSON export; no seed contract | Agent logs vary; no seed contract by default | Infra provider; determinism N/A |
| Local‑only policy | Centralized, enforced | Supports local providers; no global egress block | Can target local endpoints; no global egress block | Can run locally; policy up to operator | CLI/headless possible with local models; no explicit no‑egress contract | Local deployments possible; no explicit no‑egress | Remote by design |
| Pre‑stream provenance | Yes (context.summary first) | No pre‑stream provenance concept | Post‑hoc history/logs | Proxy/request logs; not a pre‑stream record | Trajectories capture actions; not guaranteed first record | Varies; no standard pre‑stream record | N/A |
| Provider routing/readiness | Chutes + Runpod docs/CI | Multi‑provider via plugins; user selects model | Multi‑provider by config; not a router | Router with load‑balancing, fallbacks, budgets, timeouts | Multiple backends; not a cost/cap router | Integrates backends; not a cost/cap router | Endpoint only; routing external |
| TUI/UX helpers | TUI + slash; CI ergonomics | Solid CLI; less operator TUI focus | Rich CLI with in‑chat commands | Admin UI + CLI; operator‑oriented proxy | GUI/CLI/headless modes | CLI/headless | Infra console; not agent UX |

Notes: cxplus cells are backed by Evidence below; competitors limited to CLI/proxy tools and endpoint infra.

## Evidence (file:line anchors)

- Determinism, artifacts, exit codes
  - Chat clamp payload (seed → temp=0, top_p=1, penalties neutralized): codex-rs/core/src/chat_completions.rs:291–299
  - Chat unit test for clamp: codex-rs/core/src/chat_completions.rs:943–983
  - Responses path applies determinism: codex-rs/core/src/client.rs:279–284; codex-rs/core/src/responses_payload.rs:6–18
  - Artifacts locations and summary fields: README.md:112–122, 213–227
  - Exit codes policy: README.md:220–227; QUICKSTART.md:176–183
- Local‑only + proxy bypass
  - Global client adds `no_proxy()` under sandbox/local‑only: codex-rs/core/src/default_client.rs:143–156
  - local_only detection: codex-rs/core/src/default_client.rs:162–167
  - Deny non‑local endpoints (Chat): codex-rs/core/src/client.rs:136–153
  - Deny non‑local endpoints (Responses attempt): codex-rs/core/src/client.rs:321–337, 351–364
  - Doc note requiring `create_client()`: FEATURES.md (HTTP egress policy section)
- Pre‑stream provenance (context.summary v2)
  - Emitted before user turn; seq/run_id/budget/metrics captured: codex-rs/exec/src/lib.rs:447–538
  - CI test ensures it is first NDJSON line: .github/workflows/cxplus-verify.yml:24–27
- Provider posture and Runpod
  - Chutes quickstart: README.md:141–159
  - Runpod provider config: README.md:188–207
  - Optional CI healthcheck: .github/workflows/cxplus-verify.yml:40–60
- TUI slash helpers
  - Commands and parsing: codex-rs/common/src/slash.rs:5–40, 51–138
  - Discover flag parsing + strict unknown flags: codex-rs/common/src/slash.rs:140–201; tests: 141–154

## Gaps and prioritized fixes (2‑week plan, acceptance)

1) Single HTTP egress path + lint
- Change: enforce `create_client()` everywhere; add CI grep to forbid direct `reqwest::Client::new()` outside default_client.rs.
- Acceptance: grep clean; cargo test --all-features green.

2) Better diagnostics on client fallback
- Change: warn on fallback (implemented).
- Acceptance: unit test asserts warning emitted.

3) Determinism clamp tests for Responses
- Change: test added (implemented).
- Acceptance: fails if fields regress.

4) Strict unknown flag handling for /discover
- Change: implemented; unknown flags surface `parse-error`.
- Acceptance: unit test added.

5) Optional: Rust integration for pre‑stream ordering
- Change: exec test asserting first NDJSON record is `context.summary` (Python scenario already in CI).
- Acceptance: test passes; complements scenario.

## Ready‑for‑internal‑use checklist

- CI workflow: .github/workflows/cxplus-verify.yml
  - Build + deterministic tests: lines 20–23
  - Pre‑stream `context.summary` check: lines 24–27
  - Workspace tests: lines 28–39
  - Optional Runpod healthcheck (non‑blocking): lines 40–60
- Provider posture
  - Default local‑only for CI/dev:
    ```toml
    local_only = true
    [tools]
    web_search = false
    ```
    Behavior: localhost‑only, bypass env proxies; login blocked (README.md:64–71)
  - Runpod opt‑in (keep model parametric):
    ```toml
    [model_providers.runpod]
    name = "Runpod (OpenAI-compatible)"
    base_url = "https://<your-endpoint>/v1"
    env_key = "RUNPOD_API_KEY"
    wire_api = "chat"
    ```
    One‑off: `RUNPOD_API_KEY=... cxplus -c model_provider=runpod -c model="gpt-<your-model>" exec "hello" --seed 42`

## Appendix

- Smoke test: `make package && RUN_LIVE=0 make test && ./dist/bin/codex exec "hello" --seed 42` — expect artifacts in `./.codex/runs/`, first NDJSON line `context.summary`, summary `exit_code=0`.
- Risks: Chutes reliability (external catalogs and warm‑ups) can drift/fail; mitigate via local router or Runpod, fixture‑based discovery in CI, price caps, debug flags.
