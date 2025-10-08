#!/usr/bin/env bash
set -euo pipefail

echo "[pre-commit] Regenerating docs..."
make docs-gen >/dev/null
git add docs/generated || true
echo "[pre-commit] Done."

