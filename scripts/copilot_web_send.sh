#!/usr/bin/env bash
set -euo pipefail
# Best-effort helper to paste a prepared prompt into GitHub Copilot Web
# via AppleScript UI automation on macOS. Requires Accessibility access for
# Terminal/iTerm (System Settings → Privacy & Security → Accessibility).
#
# Usage:
#   scripts/copilot_web_send.sh -f prompt.txt [-b safari|chrome] [-u URL] [-t TABS] [--send-mode auto|manual]
#
# Defaults:
#   BROWSER=safari
#   COPILOT_URL=https://github.com/copilot
#   FOCUS_TABS=1  # number of TABs to move focus to the input box before pasting

usage() {
  echo "Usage: $0 -f <prompt_file> [-b safari|chrome] [-u url] [-t tabs]" >&2
  exit 2
}

if [[ "${OSTYPE:-}" != darwin* ]]; then
  echo "This helper currently supports macOS (AppleScript) only." >&2
  exit 1
}

PROMPT_FILE=""
BROWSER="safari"
COPILOT_URL="https://github.com/copilot"
FOCUS_TABS=1
SEND_MODE="manual"

# Robustness knobs (can be overridden via env)
: "${FOCUS_RETRIES:=6}"
: "${PASTE_CHECK_PREFIX:=200}"
: "${JITTER_BASE_MS:=40}"
: "${JITTER_VAR_MS:=120}"

while getopts ":f:b:u:t:-:" opt; do
  case "$opt" in
    f) PROMPT_FILE="$OPTARG" ;;
    b) BROWSER="$OPTARG" ;;
    u) COPILOT_URL="$OPTARG" ;;
    t) FOCUS_TABS="$OPTARG" ;;
    -)
      case "$OPTARG" in
        send-mode=*) SEND_MODE="${OPTARG#send-mode=}" ;;
        *) echo "Unknown long option --$OPTARG" >&2; exit 2 ;;
      esac
      ;;
    *) usage ;;
  esac
done

[[ -f "$PROMPT_FILE" ]] || { echo "Prompt file not found: $PROMPT_FILE" >&2; exit 1; }

case "$BROWSER" in
  safari|Safari) APP="Safari" ;;
  chrome|Chrome|GoogleChrome|google-chrome) APP="Google Chrome" ;;
  *) echo "Unsupported browser: $BROWSER (use safari|chrome)" >&2; exit 1 ;;
esac

PROMPT_CONTENT=$(cat "$PROMPT_FILE")
PROMPT_PREFIX_CHECK="${PROMPT_CONTENT:0:$PASTE_CHECK_PREFIX}"

# Copy content to clipboard (safe paste)
printf "%s" "$PROMPT_CONTENT" | pbcopy

# Jitter helper (Python for portability)
rand_sleep() {
python3 - <<PY
import os,random,time
base=int(os.environ.get('JITTER_BASE_MS','40'))
var=int(os.environ.get('JITTER_VAR_MS','120'))
time.sleep((base+random.randint(0,var))/1000)
PY
}

fail() { echo "[copilot-send][error] $*" >&2; exit 1; }

# Focus/activate with retries and navigate to URL
for i in $(seq 1 "$FOCUS_RETRIES"); do
  osascript - <<APPLESCRIPT 2>/dev/null || true
tell application "$APP"
  activate
  open location "$COPILOT_URL"
end tell
delay 0.6
tell application "System Events"
  keystroke "l" using {command down}
  delay 0.05
  keystroke "$COPILOT_URL"
  key code 36
end tell
APPLESCRIPT
  rand_sleep
  FRONT=$(osascript -e 'tell application "System Events" to get name of first process whose frontmost is true' 2>/dev/null || true)
  if [ "$FRONT" = "$APP" ]; then break; fi
  if [ "$i" = "$FOCUS_RETRIES" ]; then fail "unable to focus $APP"; fi
done

# Move focus to input via TABs (best effort)
osascript - <<APPLESCRIPT 2>/dev/null || true
tell application "System Events" to tell process "$APP"
  repeat $FOCUS_TABS times
    key code 48
    delay 0.08
  end repeat
end tell
APPLESCRIPT

# Save clipboard, set prompt, paste
OLD_CLIP=$(pbpaste 2>/dev/null || true)
printf '%s' "$PROMPT_CONTENT" | pbcopy
rand_sleep
osascript - <<'APPLESCRIPT'
tell application "System Events"
  keystroke "a" using {command down}
  delay 0.05
  keystroke "v" using {command down}
end tell
APPLESCRIPT
rand_sleep

# Verification: copy back and compare prefix
osascript - <<'APPLESCRIPT'
tell application "System Events"
  keystroke "a" using {command down}
  delay 0.05
  keystroke "c" using {command down}
end tell
APPLESCRIPT
rand_sleep
COPIED_BACK=$(pbpaste 2>/dev/null || true)
if [ "${COPIED_BACK:0:$PASTE_CHECK_PREFIX}" != "$PROMPT_PREFIX_CHECK" ]; then
  printf '%s' "$OLD_CLIP" | pbcopy || true
  fail "paste verification failed (prefix mismatch)"
fi

# Auto send if requested
if [ "$SEND_MODE" = "auto" ]; then
  rand_sleep
  osascript - <<'APPLESCRIPT'
tell application "System Events"
  key code 36
end tell
APPLESCRIPT
fi

# Restore clipboard
printf '%s' "$OLD_CLIP" | pbcopy || true
echo "[copilot-send] success (chars=${#PROMPT_CONTENT}, mode=$SEND_MODE)"
