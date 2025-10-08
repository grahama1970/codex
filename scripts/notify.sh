#!/usr/bin/env bash
set -euo pipefail
# Notify locally (desktop) and/or Slack if a webhook is set.
# Usage: notify.sh "Message text" [title]

MSG=${1:-"(no message)"}
TITLE=${2:-"Copilot Review"}

if command -v notify-send >/dev/null 2>&1; then
  notify-send "$TITLE" "$MSG" || true
elif command -v osascript >/dev/null 2>&1; then
  osascript -e "display notification \"$MSG\" with title \"$TITLE\"" || true
fi

WEBHOOK="${SLACK_WEBHOOK:-${SLACK_WEBHOOK_PATH:-}}"
if [ -n "${WEBHOOK}" ]; then
  payload=$(jq -n --arg text "$TITLE: $MSG" '{text:$text}')
  curl -sS -X POST -H 'Content-type: application/json' --data "$payload" "$WEBHOOK" >/dev/null || true
fi

echo "$TITLE: $MSG"

