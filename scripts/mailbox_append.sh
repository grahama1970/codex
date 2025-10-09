#!/usr/bin/env bash
set -euo pipefail
: "${MAILBOX:=.codex/mailbox.jsonl}"
: "${FROM:=cli}"
: "${CHANNEL:=review}"
: "${PRIORITY:=5}"
: "${TTL:=3600}"
: "${BODY:=}"
: "${TURN_ID:=}"
: "${RUN_ID:=}"
: "${STATUS:=queued}"
: "${TAGS:=}"
mkdir -p "$(dirname "$MAILBOX")"
ts="$(date -u +%s)"
# Portable sha256 sum
if command -v shasum >/dev/null 2>&1; then
  hash="$(printf '%s' "$BODY" | shasum -a 256 | awk '{print $1}')"
else
  hash="$(printf '%s' "$BODY" | sha256sum | awk '{print $1}')"
fi
id_base="${RUN_ID:-$ts}"
id="${id_base}-$hash"
# Idempotency: skip if id already present
if [ -f "$MAILBOX" ] && grep -q "\"id\":\"$id\"" "$MAILBOX" 2>/dev/null; then
  echo "[mailbox] duplicate id, skipping ($id)"
  exit 0
fi
tags_json="[]"
if [ -n "$TAGS" ]; then
  tags_json="$(printf '%s' "$TAGS" | awk -F, '{print "["; for(i=1;i<=NF;i++){gsub(/^ +| +$/,"",$i); printf "%s\"%s\"", (i>1?",":""), $i} print "]"}')"
fi
jq -cn --arg id "$id" --arg turn_id "${TURN_ID:-}" --arg run_id "${RUN_ID:-}" \
  --arg from "$FROM" --arg channel "$CHANNEL" --arg status "$STATUS" \
  --argjson priority "${PRIORITY}" --argjson ttl "${TTL}" --arg body "$BODY" --arg created_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" --argjson tags "$tags_json" '
  {
    id:$id, turn_id:$turn_id, run_id:$run_id, from:$from, channel:$channel,
    status:$status, priority:$priority, ttl:$ttl, tags:$tags,
    body:$body, created_at:$created_at
  }' | tee -a "$MAILBOX" >/dev/null
echo "[mailbox] appended $id"
