#!/usr/bin/env bash
set -euo pipefail

CFG_DIR="${HOME}/.cx-plus"
CFG_FILE="${CFG_DIR}/config.toml"
SRC_EXAMPLE="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)/docs/local/cx_plus.config.example.toml"

mkdir -p "$CFG_DIR"
if [[ -f "$CFG_FILE" ]]; then
  echo "[init] config exists at $CFG_FILE"; exit 0
fi
cp "$SRC_EXAMPLE" "$CFG_FILE"
echo "[init] wrote default config to $CFG_FILE"
echo "[init] please edit owners, barriers, and base_url before enabling prehook"

