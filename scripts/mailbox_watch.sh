#!/usr/bin/env bash
set -euo pipefail
: "${MAILBOX:=.codex/mailbox.jsonl}"
: "${SINCE:=0}"
: "${ON_MSG:=echo}"
: "${ONLY_STATUS:=}"
mkdir -p "$(dirname "$MAILBOX")"
touch "$MAILBOX"
# portable date parsing: try GNU date then BSD
parse_ts() {
  local iso="$1"
  if date -u -d "$iso" +%s >/dev/null 2>&1; then
    date -u -d "$iso" +%s
  else
    date -u -j -f "%Y-%m-%dT%H:%M:%SZ" "$iso" +%s 2>/dev/null || echo 0
  fi
}

tail -Fn +1 "$MAILBOX" | while read -r line; do
  [ -z "$line" ] && continue
  ts=$(printf '%s' "$line" | jq -r '.created_at' | awk '{print $1}')
  ts_num=$(parse_ts "$ts")
  [ "$ts_num" -ge "$SINCE" ] || continue
  if [ -n "$ONLY_STATUS" ]; then
    st=$(printf '%s' "$line" | jq -r '.status // empty')
    [ "$st" = "$ONLY_STATUS" ] || continue
  fi
  eval "$ON_MSG" "'$line'"
done
