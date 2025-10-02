#!/usr/bin/env bash
set -euo pipefail

# Measure end-to-end MCP latency once, ensuring we stay under budget.
# Requires MCP server binary installed at /usr/local/bin/memory-mcp (or set MCP_SERVER_BIN)

MCP_SERVER_BIN="${MCP_SERVER_BIN:-/usr/local/bin/memory-mcp}"
CLIENT="${MCP_CLIENT_BIN:-codex-mcp-client}"
CONNECT_MS="${PREHOOK_CONNECT_TIMEOUT_MS:-650}"
CALL_MS="${PREHOOK_CALL_TIMEOUT_MS:-700}"

payload='{"tool":"codex.prehook.review","args":{"arg":"how to build","scope":"project","k":3}}'

ts=$(date +%s%3N)
out=$("$CLIENT" --server "$MCP_SERVER_BIN" --connect-timeout-ms "$CONNECT_MS" --call-timeout-ms "$CALL_MS" --params "$payload" 2>/dev/null || true)
te=$(date +%s%3N)
dt=$((te - ts))
echo "latency_ms=$dt"
echo "$out" | sed -e 's/.\{200\}/&\n/g' | head -n 10

if [[ $dt -gt 800 ]]; then
  echo "WARNING: prehook MCP call exceeded 800 ms" >&2
  exit 1
fi

