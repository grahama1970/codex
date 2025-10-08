#!/usr/bin/env bash
set -euo pipefail
# Poll a PR for new Copilot review comments and send a local notification when they appear.

REPO=${REPO:-$(gh repo view --json nameWithOwner -q .nameWithOwner)}
PR=${PR:-$(gh pr view -R "$REPO" --json number -q .number)}
STATE_FILE=${STATE_FILE:-local/.copilot_watch.state}
OUT=${OUT:-local/copilot_pr_review.txt}

mkdir -p "$(dirname "$STATE_FILE")"

prev_count=$(cat "$STATE_FILE" 2>/dev/null || echo 0)

authors=$(gh pr view "$PR" -R "$REPO" --json comments -q '.comments[].author.login' | tr -d '\r' || true)
count=$(echo "$authors" | rg -ci 'copilot' || true)

if [ "$count" -gt "$prev_count" ]; then
  gh pr view "$PR" -R "$REPO" --comments > "$OUT"
  echo "$count" > "$STATE_FILE"
  msg="Copilot posted new review comments on PR #$PR"
  if command -v notify-send >/dev/null 2>&1; then
    notify-send "Copilot Review" "$msg"
  elif command -v osascript >/dev/null 2>&1; then
    osascript -e "display notification \"$msg\" with title \"Copilot Review\""
  fi
  echo "$msg"
else
  echo "$count" > "$STATE_FILE"
  echo "No new Copilot comments. (count=$count)"
fi
