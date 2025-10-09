#!/usr/bin/env bash
set -euo pipefail
: "${MAILBOX:=.codex/mailbox.jsonl}"
: "${FROM:=cli}"
: "${CHANNEL:=review}"
: "${PRIORITY:=5}"
: "${TTL:=3600}"
: "${BODY:=}"
: "${TURN_ID:=}"
mkdir -p "$(dirname "$MAILBOX")"
ts="$(date -u +%s)"
# Portable sha256 sum
if command -v shasum >/dev/null 2>&1; then
  hash="$(printf '%s' "$BODY" | shasum -a 256 | awk '{print $1}')"
else
  hash="$(printf '%s' "$BODY" | sha256sum | awk '{print $1}')"
fi
id="${TURN_ID:-$ts}-$hash"
# Idempotency: skip if id already present
if [ -f "$MAILBOX" ] && grep -q "\"id\":\"$id\"" "$MAILBOX" 2>/dev/null; then
  echo "[mailbox] duplicate id, skipping ($id)"
  exit 0
fi
jq -cn --arg id "$id" --arg turn_id "${TURN_ID:-}" --arg from "$FROM" --arg channel "$CHANNEL" \
  --argjson priority "${PRIORITY}" --argjson ttl "${TTL}" --arg body "$BODY" --arg created_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" '
  {
    id:$id, turn_id:$turn_id, from:$from, channel:$channel,
    priority:$priority, ttl:$ttl, body:$body, created_at:$created_at
  }' | tee -a "$MAILBOX" >/dev/null
echo "[mailbox] appended $id"
