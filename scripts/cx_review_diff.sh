#!/usr/bin/env bash
set -euo pipefail
# Render unified diffs from normalized mailbox findings.
# Usage: scripts/cx_review_diff.sh --in .codex/mailbox.jsonl
IN=""
while [ $# -gt 0 ]; do
  case "$1" in
    --in) IN="$2"; shift 2;;
    *) echo "unknown arg: $1" >&2; exit 64;;
  esac
done
[ -n "$IN" ] || { echo "usage: $0 --in mailbox.jsonl" >&2; exit 64; }
[ -f "$IN" ] || { echo "no such file: $IN" >&2; exit 66; }
found=0
while IFS= read -r line; do
  body="$(printf '%s' "$line" | jq -r '.body // empty')"
  [ -z "$body" ] && continue
  if printf '%s\n' "$body" | grep -q "^--- "; then
    found=1
    printf '%s\n' "$body" | awk 'BEGIN{in=0} /^--- /{in=1} {if(in) print}'
  fi
  if printf '%s\n' "$body" | grep -q '```diff'; then
    found=1
    printf '%s\n' "$body" | awk '/```diff/{flag=1;next}/```/{flag=0}flag'
  fi
done < "$IN"
if [ $found -eq 0 ]; then
  echo "[cx review:diff] no diffs found in $IN" >&2
fi
