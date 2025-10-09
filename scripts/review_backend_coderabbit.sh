#!/usr/bin/env bash
set -euo pipefail
# Optional CodeRabbit CLI adapter (disabled unless CODERABBIT_TOKEN set).
# Usage: CODERABBIT_TOKEN=... scripts/review_backend_coderabbit.sh --repo owner/repo --pr 123 > local/coderabbit_review.txt
if [ -z "${CODERABBIT_TOKEN:-}" ]; then
  echo "[coderabbit] token not set; adapter disabled" >&2
  exit 1
fi
REPO=""; PR=""
while [ $# -gt 0 ]; do
  case "$1" in
    --repo) REPO="$2"; shift 2;;
    --pr) PR="$2"; shift 2;;
    *) echo "unknown arg: $1" >&2; exit 64;;
  esac
done
[ -n "$REPO" ] && [ -n "$PR" ] || { echo "usage: $0 --repo owner/repo --pr N" >&2; exit 64; }
# Placeholder for real CLI call:
# coderabbit review --repo "$REPO" --pr "$PR" --token "$CODERABBIT_TOKEN"
echo "[coderabbit] (stub) review for $REPO#$PR" >&2
echo "{}"
