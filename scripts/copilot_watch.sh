#!/usr/bin/env bash
set -euo pipefail
# Poll a PR for new Copilot review comments and send a local notification when they appear.

load_env_file() {
  local _envf="$1"
  [ -f "$_envf" ] || return 0
  # Read key=value lines without clobbering existing exports
  while IFS='=' read -r k v; do
    [ -z "${k:-}" ] && continue
    case "$k" in \#*) continue;; esac
    [ "${!k+x}" = x ] && continue
    export "$k=$v"
  done < <(grep -v '^[[:space:]]*#' "$_envf" | grep '=')
}
load_env_file ".env"
load_env_file "$(dirname "$0")/../.env"

REPO=${REPO:-$(gh repo view --json nameWithOwner -q .nameWithOwner)}
PR=${PR:-$(gh pr view -R "$REPO" --json number -q .number)}
STATE_FILE=${STATE_FILE:-local/.copilot_watch.state}
OUT=${OUT:-local/copilot_pr_review.txt}

mkdir -p "$(dirname "$STATE_FILE")"

prev_count=$(cat "$STATE_FILE" 2>/dev/null || echo 0)

authors=$(gh pr view "$PR" -R "$REPO" --json comments -q '.comments[].author.login' | tr -d '\r' || true)
count=$(echo "$authors" | rg -ci 'copilot' 2>/dev/null || echo 0)
if [ -z "$count" ]; then count=0; fi

if [ "$count" -gt "$prev_count" ]; then
  gh pr view "$PR" -R "$REPO" --comments > "$OUT"
  echo "$count" > "$STATE_FILE"
  msg="Copilot posted new review comments on PR #$PR"
  ./scripts/notify.sh "$msg" "Copilot Review" || true
  echo "$msg"
else
  echo "$count" > "$STATE_FILE"
  echo "No new Copilot comments. (count=$count)"
fi
