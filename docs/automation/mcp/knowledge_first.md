# Knowledge‑First MCP prehook integration

Goal: run your Knowledge‑First memory agent as an MCP stdio server, and have cx‑plus call it as a prehook before exec/apply.

## 1) Build/launch your MCP server

From your memory repo (example paths):

```
cd /home/graham/workspace/experiments/memory
# (follow QUICK_START_GUIDE.md to install deps and build the server)
# Example: node dist/mcp_server.js --stdio  # prefer stdio mode
```

If your agent supports stdio MCP directly, great. If not, use a thin wrapper (below) that starts your server in stdio mode or bridges to your process.

## 2) Wrapper script (stdio launcher)

We provide a small wrapper you can customize:

- `scripts/mcp/knowledge_first.sh`

Edit the absolute path and args inside to match your agent’s entrypoint.

## 3) Configure cx‑plus prehook

Initialize config once:

```
scripts/init-cx-plus-config.sh
$EDITOR ~/.cx-plus/config.toml
```

Set:

```
[prehook]
enabled = true
backend = "mcp"
mcp_server = "stdio:/ABS/PATH/TO/your/knowledge_first.sh --stdio"  # wrapper
mcp_tool = "codex.prehook.review"
connect_timeout_ms = 800
call_timeout_ms = 800
```

## 4) Contract

- Input: Context JSON (repo, planned action, diff preview for apply, risk flags)
- Output: MCP structured_content → a JSON object with at least:
  - `decision`: "allow" | "augment" | "ask" | "patch" | "deny" | "rate_limit"
  - When `augment`: `context_items` ≤ 128 with `{ title, content, why, scope?, score? }`

Keep payload ≤ 64 KiB and respond in ≤ 800 ms. On timeout or missing fields, our prehook treats it as retryable and continues per on_error policy.

## 5) Verify

```
# Start your MCP server (stdio)
/path/to/knowledge_first.sh --stdio &

# Run exec with prehook on
cx-plus exec --prehook-enabled --prehook-backend mcp \
  --prehook-mcp-server 'stdio:/path/to/knowledge_first.sh --stdio' \
  --prehook-mcp-tool codex.prehook.review \
  --prehook-mcp-connect-timeout-ms 800 \
  --prehook-mcp-call-timeout-ms 800 \
  -m gpt-oss:20b "What is the plan for X?"
```

You should see either `augment` items or `deny/ask/patch/rate_limit` outcomes shaping execution.

