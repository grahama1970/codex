#!/usr/bin/env bash
set -euo pipefail
# Notify locally (desktop) and/or Slack if SLACK_WEBHOOK is set.
# Usage: notify.sh "Message text" [title]

MSG=${1:-"(no message)"}
TITLE=${2:-"Copilot Review"}

if command -v notify-send >/dev/null 2>&1; then
  notify-send "$TITLE" "$MSG" || true
elif command -v osascript >/dev/null 2>&1; then
  osascript -e "display notification \"$MSG\" with title \"$TITLE\"" || true
fi

if [ -n "${SLACK_WEBHOOK:-}" ]; then
  payload=$(jq -n --arg text "$TITLE: $MSG" '{text:$text}')
  curl -sS -X POST -H 'Content-type: application/json' --data "$payload" "$SLACK_WEBHOOK" >/dev/null || true
fi

echo "$TITLE: $MSG"

