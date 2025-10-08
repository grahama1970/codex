#!/usr/bin/env bash
set -euo pipefail
# Best-effort helper to paste a prepared prompt into GitHub Copilot Web
# via AppleScript UI automation on macOS. Requires Accessibility access for
# Terminal/iTerm (System Settings → Privacy & Security → Accessibility).
#
# Usage:
#   scripts/copilot_web_send.sh -f prompt.txt [-b safari|chrome] [-u URL] [-t TABS]
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

while getopts ":f:b:u:t:" opt; do
  case "$opt" in
    f) PROMPT_FILE="$OPTARG" ;;
    b) BROWSER="$OPTARG" ;;
    u) COPILOT_URL="$OPTARG" ;;
    t) FOCUS_TABS="$OPTARG" ;;
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

# Copy content to clipboard (safe paste)
printf "%s" "$PROMPT_CONTENT" | pbcopy

# shellcheck disable=SC2005
osascript - "$COPILOT_URL" "$FOCUS_TABS" "$APP" <<'APPLESCRIPT'
on run argv
  set targetURL to item 1 of argv
  set focusTabs to (item 2 of argv) as integer
  set appName to item 3 of argv

  tell application appName
    activate
    open location targetURL
  end tell

  delay 2 -- allow page to load
  tell application "System Events"
    tell process appName
      repeat focusTabs times
        key code 48 -- TAB
        delay 0.1
      end repeat
      keystroke "v" using {command down}
      delay 0.2
      key code 36 -- Return
    end tell
  end tell
end run
APPLESCRIPT

echo "Pasted prompt from $PROMPT_FILE into $APP at $COPILOT_URL"

