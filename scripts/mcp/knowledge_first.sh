#!/usr/bin/env bash
set -euo pipefail

# Thin wrapper to launch your Knowledge‑First MCP server in stdio mode.
# Uses a system-installed stdio server binary by default. Override with MCP_SERVER_BIN.

MCP_SERVER_BIN="${MCP_SERVER_BIN:-/usr/local/bin/memory-mcp}"

if [[ ! -x "$MCP_SERVER_BIN" ]]; then
  echo "[knowledge_first] ERROR: MCP server not found or not executable: $MCP_SERVER_BIN" >&2
  echo "Hint: set MCP_SERVER_BIN=/abs/path/to/your/server or install to /usr/local/bin/memory-mcp" >&2
  exit 1
fi

# Dev convenience: allow default ARANGO_PASS unless explicitly disabled
export DEV_ALLOW_DEFAULT="${DEV_ALLOW_DEFAULT:-1}"
exec "$MCP_SERVER_BIN" --stdio "$@"
