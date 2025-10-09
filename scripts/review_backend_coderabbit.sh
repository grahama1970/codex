#!/usr/bin/env bash
set -euo pipefail
# Minimal CodeRabbit CLI adapter (headed/local profile assumed).
# Usage: CODERABBIT_TOKEN=... scripts/review_backend_coderabbit.sh --repo owner/repo --pr 123 --out local/coderabbit_review.jsonl
# Respects local-only posture: denies when CODEX_LOCAL_ONLY=1.

if [ "${CODEX_LOCAL_ONLY:-}" = "1" ]; then
  echo "[coderabbit] denied by local_only (no egress allowed)" >&2
  exit 2
fi
if [ -z "${CODERABBIT_TOKEN:-}" ]; then
  echo "[coderabbit] token not set; export CODERABBIT_TOKEN" >&2
  exit 1
fi
REPO=""; PR=""; OUT="local/coderabbit_review.jsonl"; TAGS="${TAGS:-coderabbit}"
while [ $# -gt 0 ]; do
  case "$1" in
    --repo) REPO="$2"; shift 2;;
    --pr) PR="$2"; shift 2;;
    --out) OUT="$2"; shift 2;;
    *) echo "unknown arg: $1" >&2; exit 64;;
  esac
done
[ -n "$REPO" ] && [ -n "$PR" ] || { echo "usage: $0 --repo owner/repo --pr N [--out path]" >&2; exit 64; }
mkdir -p "$(dirname "$OUT")"
ts="$(date -u +%s)"
run_id="${RUN_ID:-$ts}"
# Invoke CodeRabbit CLI (replace with the actual command if different).
set +e
RAW="$(
  coderabbit review --repo "$REPO" --pr "$PR" --token "$CODERABBIT_TOKEN"
)"
rc=$?
set -e
status="processed"
[ $rc -eq 0 ] || status="error"
body="$(printf '%s' "$RAW" | sed 's/\\/\\\\/g' | sed 's/"/\"/g')"
tags_json="$(printf '%s' "$TAGS" | awk -F, '{print "["; for(i=1;i<=NF;i++){gsub(/^ +| +$/,"",$i); printf "%s"%s"", (i>1?",":""), $i} print "]"}')"
echo "{"id":"$run_id-$(echo -n "$body" | shasum -a 256 | awk '{print $1}')","turn_id":"","run_id":"$run_id","from":"coderabbit","channel":"reviews","status":"$status","priority":5,"ttl":3600,"tags":$tags_json,"body":"$body","created_at":"$(date -u +%Y-%m-%dT%H:%M:%SZ)"}" >> "$OUT"
echo "[coderabbit] wrote $(basename "$OUT") with status=$status"
