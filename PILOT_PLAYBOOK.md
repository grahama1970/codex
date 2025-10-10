# Pilot Playbook — cxplus internal rollout

This playbook captures guardrails, success criteria, and the exact commands to validate cxplus in dev/staging. It assumes the stamped binary lives under `dist/bin/codex` (alias `cxplus`).

## Guardrails (defaults)

- Local‑only by default in CI/dev: `local_only = true` (no egress; proxy bypass; logins/search/remote MCP/OTEL/notifications blocked).
- Determinism on demand: pass `--seed <u64>`; cxplus clamps temperature/top_p and neutralizes penalties where supported; seed persists in artifacts.
- Artifacts always on: NDJSON events + summary JSON per run; first NDJSON line is `context.summary v2`.
- Exit codes: `0` ok, `1` error, `5` timeout or interrupted (graceful).
- Providers: Start with local‑only; opt‑in to Runpod via profiles/`-c` overrides with secrets.

## Success criteria (pilot)

1) 100% of runs produce both artifacts; first NDJSON line is `context.summary`; summary includes `exit_code` and `seed` when provided.
2) 0 unexpected egress in local‑only mode; remote endpoints are denied with a clear policy error.
3) Timeout and SIGINT produce markers (`run_timeout` / `run_interrupted`) and exit code `5`.
4) If Runpod lane is enabled: `/models` and a single `/chat/completions` smoke succeed.

## Validation steps (copy/paste)

### 1) Seeded run (determinism + artifacts)

```bash
dist/bin/codex exec "hello from cxplus" --seed 4242

# Inspect artifacts
ls -1 .codex/runs/*-events.ndjson | tail -n1 | xargs -I{} head -n1 {}
jq '{status,exit_code,model,model_provider,seed}' .codex/runs/*-summary.json | tail -n1
```

Expect: first NDJSON line `kind=context.summary`, summary includes `seed`, `exit_code` in {0,1,5}.

### 2) Timeout path

```bash
dist/bin/codex exec "force timeout demo" --run-timeout-secs 1 || true
grep -m1 '"kind":"run_timeout"' .codex/runs/*-events.ndjson | tail -n1
jq '{status,exit_code,timed_out,interrupted}' .codex/runs/*-summary.json | tail -n1
```

Expect: `run_timeout` marker present; summary `status="timeout"`, `exit_code=5`.

### 3) Local‑only policy denial (remote provider)

```bash
dist/bin/codex -c local_only=true -c model_provider=openai -c model="gpt-5" exec --json "hello" || true
```

Expect: JSON stream shows `unsupported operation: external model provider blocked by policy`.

### 4) Router fixture lane (optional)

```bash
# Use a local catalog fixture to keep CI deterministic
export CHUTES_CATALOG_FIXTURE=/abs/path/catalog.json
dist/bin/codex chutes recommend --show-base
dist/bin/codex chutes warmup --secs 4 --dry-run
```

Expect: a model id is printed; warmup emits `[chutes-warmup]` lines. For live discovery, supply `CHUTES_API_KEY`.

### 5) Runpod lane (optional)

```bash
export RUNPOD_API_BASE=https://<your-endpoint>/v1
export RUNPOD_API_KEY=rpv2-...
make config   # adds [model_providers.runpod]
dist/bin/codex -c model_provider=runpod -c model="gpt-<your-model>" exec "hello" --seed 42
```

Expect: a normal run (remote egress), artifacts present; use only in environments where remote egress is permitted.

## Known caveats & triage

- Some endpoints reject clamp fields (e.g., `temperature`) on specific wire paths. If you see a `400` like “Unsupported parameter: temperature”, the seed clamp was active but the endpoint does not accept that field. Mitigations:
  - Prefer the wire API (chat vs responses) that supports determinism for your provider.
  - Keep local‑only profiles off remote endpoints.
- (Roadmap) Add capability probes to suppress unsupported clamp fields per provider.

## Knowledge‑First (Arango/MCP) — Opt‑in track during pilot

Knowledge‑First is the core reason cxplus exists. Enable a small, opt‑in lane so the main pilot remains predictable while retrieval infra stabilizes.

### Two ways to enable

1) Deterministic fixture (no infra):

```bash
export CONTEXT_MCP_FIXTURE=/abs/path/fixture.json   # static retrieval data

# Force provider via CLI (no config changes needed)
dist/bin/codex -c context.provider=arango exec "hello" --seed 42
head -n1 .codex/runs/*-events.ndjson   # expect kind=context.summary v2 with evidence/metrics fields
```

2) Live Arango + memory‑agent MCP:

```bash
// Optional debugging and safety knobs
export CONTEXT_DEBUG=1
export CONTEXT_EVIDENCE_ALLOW_CODE=0   # set 1 only if you allow code in evidence

# Start/point to your memory‑agent MCP and ArangoDB; cxplus defaults:
# endpoint=http://localhost:8529 database=codex tool=memory-agent

dist/bin/codex -c context.provider=arango \
  -c context.arango.endpoint=http://localhost:8080/jsonrpc \
  -c context.arango.mcp_tool=memory-agent \
  exec "summarize the repo" --seed 4242
head -n1 .codex/runs/*-events.ndjson
```

### Success criteria (Knowledge‑First lane)

- `context.summary` remains the first NDJSON line.
- `retrieval_ms > 0` and `evidence_items > 0` on at least one run.
- With fixture set, runs are deterministic and include `seed` in summary.
- No unintended egress in local‑only environments.

## Owner & rollback

- Owners: designate a maintainer for stamped releases (see `make release`, `make switch`, `make rollback`).
- Store `dist/releases/codex-<stamp>` as the artifact of record; use symlink switch to promote/rollback.

## Docker compose (reference)

- A minimal ArangoDB + memory‑agent template is in `docs/knowledge-first/docker-compose.yml`. Replace the `memory-agent` image with your org’s build and set the `endpoint` via `-c context.arango.endpoint` as shown above.
