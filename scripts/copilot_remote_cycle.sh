#!/usr/bin/env bash
set -euo pipefail
# Run the Copilot Web send → wait → fetch → process cycle on a remote macOS host
# via SSH (no browser automation on this Linux box). Requires:
# - macOS with Safari/Chrome signed in to GitHub
# - SSH access enabled (System Settings → General → Sharing → Remote Login)
# - Accessibility permission granted for Terminal/iTerm on macOS
#
# Usage (example):
#   MAC_HOST=mbp.local MAC_USER=you \
#   scripts/copilot_remote_cycle.sh \
#     --prompt local/copilot_prompt.txt \
#     --browser safari \
#     --url https://github.com/copilot \
#     --tabs 1 \
#     --send-mode auto \
#     --wait-method copy \
#     --out-dir local/remote_review
#
# If --prompt is not provided, create one locally via scripts/copilot_prompt_build.sh.

MAC_HOST=${MAC_HOST:-}
MAC_USER=${MAC_USER:-}
SSH_OPTS=${SSH_OPTS:-}
PROMPT_FILE=""
BROWSER="safari"
URL="https://github.com/copilot"
TABS="1"
SEND_MODE="manual"
WAIT_METHOD="js"   # or copy
OUT_DIR="local/remote_review"
INTERVAL=1
STABLE=3
MAX_WAIT=90

while [[ $# -gt 0 ]]; do
  case "$1" in
    --prompt) PROMPT_FILE="$2"; shift 2;;
    --browser) BROWSER="$2"; shift 2;;
    --url) URL="$2"; shift 2;;
    --tabs) TABS="$2"; shift 2;;
    --send-mode) SEND_MODE="$2"; shift 2;;
    --wait-method) WAIT_METHOD="$2"; shift 2;;
    --out-dir) OUT_DIR="$2"; shift 2;;
    --interval) INTERVAL="$2"; shift 2;;
    --stable) STABLE="$2"; shift 2;;
    --max-wait) MAX_WAIT="$2"; shift 2;;
    *) echo "Unknown arg: $1" >&2; exit 2;;
  esac
done

[[ -n "$MAC_HOST" && -n "$MAC_USER" ]] || { echo "Set MAC_HOST and MAC_USER env vars" >&2; exit 1; }

mkdir -p "$OUT_DIR"

# Build prompt if not provided
if [[ -z "$PROMPT_FILE" ]]; then
  PROMPT_FILE="local/copilot_prompt.txt"
  ./scripts/copilot_prompt_build.sh -o "$PROMPT_FILE" -t REVIEW_REQUEST.md -s "Omnibus review" || true
fi
[[ -f "$PROMPT_FILE" ]] || { echo "Prompt missing: $PROMPT_FILE" >&2; exit 1; }

echo "Creating remote temp dir on $MAC_USER@$MAC_HOST …"
REMOTE_TMP=$(ssh $SSH_OPTS "$MAC_USER@$MAC_HOST" 'mktemp -d /tmp/copilot.XXXX')
echo "Remote tmp: $REMOTE_TMP"

echo "Copying scripts and prompt …"
scp $SSH_OPTS \
  scripts/copilot_web_send.sh \
  scripts/copilot_web_wait_fetch.sh \
  scripts/copilot_process_review.sh \
  "$MAC_USER@$MAC_HOST:$REMOTE_TMP/"
scp $SSH_OPTS "$PROMPT_FILE" "$MAC_USER@$MAC_HOST:$REMOTE_TMP/prompt.txt"

echo "Running send on remote macOS …"
ssh $SSH_OPTS "$MAC_USER@$MAC_HOST" \
  "cd '$REMOTE_TMP' && chmod +x copilot_web_send.sh copilot_web_wait_fetch.sh copilot_process_review.sh && \
   ./copilot_web_send.sh -f prompt.txt -b '$BROWSER' -u '$URL' -t '$TABS' --send-mode=$SEND_MODE"

echo "Waiting and fetching output …"
ssh $SSH_OPTS "$MAC_USER@$MAC_HOST" \
  "cd '$REMOTE_TMP' && ./copilot_web_wait_fetch.sh -o review.txt -b '$BROWSER' -u '$URL' -i '$INTERVAL' -s '$STABLE' -m '$MAX_WAIT' --method=$WAIT_METHOD"

echo "Processing review text …"
ssh $SSH_OPTS "$MAC_USER@$MAC_HOST" \
  "cd '$REMOTE_TMP' && ./copilot_process_review.sh -i review.txt -t todos.md -p patch.diff"

echo "Copying results back …"
scp $SSH_OPTS "$MAC_USER@$MAC_HOST:$REMOTE_TMP/review.txt" "$OUT_DIR/copilot_review.txt" || true
scp $SSH_OPTS "$MAC_USER@$MAC_HOST:$REMOTE_TMP/todos.md" "$OUT_DIR/copilot_todos.md" || true
scp $SSH_OPTS "$MAC_USER@$MAC_HOST:$REMOTE_TMP/patch.diff" "$OUT_DIR/copilot_extracted.patch" || true

echo "Cleaning remote tmp …"
ssh $SSH_OPTS "$MAC_USER@$MAC_HOST" "rm -rf '$REMOTE_TMP'"

echo "Remote cycle complete. Outputs under $OUT_DIR"

