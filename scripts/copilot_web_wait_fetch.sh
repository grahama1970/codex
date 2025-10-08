#!/usr/bin/env bash
set -euo pipefail
# Wait for Copilot Web chat output to stabilize, then fetch the page text.
# macOS only (uses Accessibility or page JS via AppleScript).
#
# Usage:
#   scripts/copilot_web_wait_fetch.sh \
#     -o local/copilot_review.txt \
#     -b safari|chrome \
#     -u https://github.com/copilot \
#     -i 1   # poll interval seconds
#     -s 3   # stable loops (unchanged hash)
#     -m 60  # max loops
#     --method copy|js (default copy)
## Optional fallback method (no JS): --method copy

if [[ "${OSTYPE:-}" != darwin* ]]; then
  echo "This helper supports macOS only." >&2
  exit 1
fi

OUT_FILE=""
BROWSER="safari"
URL="https://github.com/copilot"
INTERVAL=1
STABLE=3
MAX_WAIT=60
METHOD="js"

METHOD="js"
while getopts ":o:b:u:i:s:m:-:" opt; do
  case "$opt" in
    o) OUT_FILE="$OPTARG" ;;
    b) BROWSER="$OPTARG" ;;
    u) URL="$OPTARG" ;;
    i) INTERVAL="$OPTARG" ;;
    s) STABLE="$OPTARG" ;;
    m) MAX_WAIT="$OPTARG" ;;
    -)
      case "$OPTARG" in
        method=*) METHOD="${OPTARG#method=}" ;;
        *) echo "Unknown long option --$OPTARG" >&2; exit 2 ;;
      esac
      ;;
    *) echo "bad args" >&2; exit 2 ;;
  esac
done

[[ -n "$OUT_FILE" ]] || { echo "-o OUT_FILE required" >&2; exit 2; }
mkdir -p "$(dirname "$OUT_FILE")"

case "$BROWSER" in
  safari|Safari) APP="Safari" ;;
  chrome|Chrome|GoogleChrome|google-chrome) APP="Google Chrome" ;;
  *) echo "Unsupported browser: $BROWSER (use safari|chrome)" >&2; exit 1 ;;
esac

# Ensure correct page is open
osascript - "$URL" "$APP" <<'APPLESCRIPT'
on run argv
  set targetURL to item 1 of argv
  set appName to item 2 of argv
  tell application appName
    activate
    open location targetURL
  end tell
end run
APPLESCRIPT

sleep 1

js_len() {
  if [[ "$APP" == "Safari" ]]; then
    osascript -e 'tell application "Safari" to do JavaScript "document.body.innerText.length" in document 1' 2>/dev/null || echo 0
  else
    osascript -e 'tell application "Google Chrome" to tell front window to tell active tab to execute javascript "document.body.innerText.length"' 2>/dev/null || echo 0
  fi
}

js_dump() {
  if [[ "$APP" == "Safari" ]]; then
    osascript -e 'tell application "Safari" to do JavaScript "document.body.innerText" in document 1' 2>/dev/null || true
  else
    osascript -e 'tell application "Google Chrome" to tell front window to tell active tab to execute javascript "document.body.innerText"' 2>/dev/null || true
  fi
}

content_copy() {
  osascript - "$APP" <<'APPLESCRIPT'
on run argv
  set appName to item 1 of argv
  tell application "System Events"
    tell process appName
      keystroke "a" using {command down}
      delay 0.15
      keystroke "c" using {command down}
    end tell
  end tell
end run
APPLESCRIPT
  sleep 0.2
  pbpaste
}

prev_hash=""
stable_loops=0
loops=0
while true; do
  loops=$((loops+1))
  RAW=""
  if [[ "$METHOD" == "copy" ]]; then
    RAW="$(content_copy || true)"
  else
    RAW="$(js_dump || true)"
  fi
  HASH="$(printf '%s' "$RAW" | shasum -a 256 | cut -d' ' -f1)"
  if [[ -n "$HASH" && "$HASH" == "$prev_hash" ]]; then
    stable_loops=$((stable_loops+1))
  else
    stable_loops=0
    prev_hash="$HASH"
  fi
  if (( stable_loops >= STABLE )); then
    printf '%s' "$RAW" > "$OUT_FILE"
    echo "Saved Copilot page text: $OUT_FILE (chars=$(printf '%s' "$RAW" | wc -c))"
    exit 0
  fi
  if (( loops >= MAX_WAIT )); then
    echo "[copilot-wait] timeout after $loops loops" >&2
    exit 1
  fi
  sleep "$INTERVAL"
done
