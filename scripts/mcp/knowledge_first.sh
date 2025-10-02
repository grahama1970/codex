#!/usr/bin/env bash
set -euo pipefail

# Thin wrapper to launch your Knowledge‑First MCP server in stdio mode.
# EDIT the path below to your agent's server entrypoint.

KF_DIR="/home/graham/workspace/experiments/memory"
KF_BIN="node"
KF_ENTRY="dist/mcp_server.js"

cd "$KF_DIR"
exec "$KF_BIN" "$KF_ENTRY" --stdio "$@"

