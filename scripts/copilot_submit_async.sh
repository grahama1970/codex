#!/usr/bin/env bash
set -euo pipefail
# Open/update a PR from current branch, print PR number/URL, and start a background
# watch loop that notifies when Copilot posts comments.

load_env_file() {
  local _envf="$1"
  [ -f "$_envf" ] || return 0
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
BASE=${BASE:-$(gh repo view --json defaultBranchRef -q .defaultBranchRef.name)}
HEAD=${HEAD:-$(git rev-parse --abbrev-ref HEAD)}
BODY_FILE=${BODY_FILE:-REVIEW_REQUEST.md}
OUT_DIR=${OUT_DIR:-local}
INTERVAL=${INTERVAL:-30}

mkdir -p "$OUT_DIR"

if ! gh pr view -R "$REPO" --json number -q .number >/dev/null 2>&1; then
  if [ -f "$BODY_FILE" ]; then
    gh pr create -R "$REPO" -B "$BASE" -H "$HEAD" -t "Comprehensive review request" -F "$BODY_FILE"
  else
    gh pr create -R "$REPO" -B "$BASE" -H "$HEAD" -t "Comprehensive review request" -b "Automated review request."
  fi
else
  if [ -f "$BODY_FILE" ]; then
    gh pr edit -R "$REPO" -F "$BODY_FILE" || true
  fi
fi

PR=$(gh pr view -R "$REPO" --json number -q '.number')
URL=$(gh pr view -R "$REPO" --json url -q .url)
printf 'PR #%s: %s\n' "$PR" "$URL"

# Start background watcher loop tied to this PR
PIDFILE="$OUT_DIR/review_watch_${PR}.pid"
LOG="$OUT_DIR/copilot_pr${PR}_review.txt"

nohup bash -c "while true; do REPO='$REPO' PR='$PR' OUT='$LOG' STATE_FILE='$OUT_DIR/.copilot_watch_${PR}.state' ./scripts/copilot_watch.sh; sleep $INTERVAL; done" \
  >/dev/null 2>&1 & echo $! > "$PIDFILE"

./scripts/notify.sh "Watching PR #$PR for Copilot comments" "Review Started"
echo "Watcher PID: $(cat "$PIDFILE") (interval=${INTERVAL}s)"
