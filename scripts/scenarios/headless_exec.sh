#!/usr/bin/env bash
set -euo pipefail

# Simulate headless exec with prehook + knowledge-first MCP.
# Requires cx-plus on PATH and scripts/mcp/knowledge_first.sh configured.

export PATH="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)/../tools/bin:$PATH"

cx-plus exec --prehook-enabled --prehook-backend mcp \
  --prehook-mcp-server "stdio:$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)/mcp/knowledge_first.sh --stdio" \
  --prehook-mcp-tool codex.prehook.review \
  --prehook-mcp-connect-timeout-ms 800 \
  --prehook-mcp-call-timeout-ms 800 \
  -m gpt-oss:20b "Summarize the latest work on agent-bus and prehook in 3 bullets."

