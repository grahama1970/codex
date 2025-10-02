# Knowledge First (Memory prehook)

Fetch Memory context before any LLM call via MCP (fallback to CLI), with a tight latency budget (≤2s typical).

## Usage

```bash
python local/feature_recipes/knowledge_first.py "your prompt"
```

Returns JSON:

```json
{ "source": "mcp|cli", "scope": "tabbed", "k": 5, "items": [ ... ] }
```

## Integration (Codex prehook)

In your prehook, call `knowledge_first_context(prompt)` and merge top N titles/snippets into the system/tool context prior to completion.

## Env

- `MEMORY_MCP_NODE` (default: `node`)
- `MEMORY_AGENT_SERVER` (default: `~/workspace/experiments/memory/mcp/memory-agent-node/server.js`)
- `MEMORY_SCOPE` (default: `tabbed`)

## Failure policy

- MCP path times out quickly (default 2s). On failure, falls back to CLI (`lessons-recall`) with its own small timeout.
- If both fail, proceed without memory and log.

