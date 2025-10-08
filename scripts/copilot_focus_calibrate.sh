#!/usr/bin/env bash
set -euo pipefail
# Help choose the correct number of TAB presses to focus Copilot chat input.
# macOS only.

if [[ "${OSTYPE:-}" != darwin* ]]; then
  echo "macOS only" >&2; exit 1
fi

BROWSER="safari"
URL="https://github.com/copilot"
MAX_TABS=5

while getopts ":b:u:n:" opt; do
  case "$opt" in
    b) BROWSER="$OPTARG" ;;
    u) URL="$OPTARG" ;;
    n) MAX_TABS="$OPTARG" ;;
    *) echo "Usage: $0 [-b safari|chrome] [-u url] [-n max_tabs]" >&2; exit 2 ;;
  esac
done

case "$BROWSER" in
  safari|Safari) APP="Safari" ;;
  chrome|Chrome|GoogleChrome|google-chrome) APP="Google Chrome" ;;
  *) echo "Unsupported browser: $BROWSER" >&2; exit 1 ;;
esac

marker="### FOCUS TEST ###"
echo "Starting focus calibration in $APP at $URL"

osascript - "$URL" "$APP" "$MAX_TABS" "$marker" <<'APPLESCRIPT'
on run argv
  set targetURL to item 1 of argv
  set appName to item 2 of argv
  set maxTabs to (item 3 of argv) as integer
  set marker to item 4 of argv
  tell application appName
    activate
    open location targetURL
  end tell
  delay 2
  tell application "System Events"
    tell process appName
      repeat with i from 0 to maxTabs
        -- reset focus by clicking body area (at roughly center)
        keystroke "l" using {command down} -- focus URL bar
        delay 0.2
        key code 36 -- Return to leave URL bar
        delay 0.2
        repeat i times
          key code 48 -- TAB
          delay 0.1
        end repeat
        keystroke marker
        delay 0.5
        -- select typed marker and delete to clean up
        keystroke "a" using {command down}
        delay 0.1
        keystroke (ASCII character 8) -- backspace
        display dialog ("TABS=" & (i as string) & " tested. If the marker appeared inside the chat input, press OK to save this value; otherwise Cancel to continue.") buttons {"Cancel", "OK"} default button 1 giving up after 2
        set btn to button returned of result
        if btn is equal to "OK" then
          -- write chosen value into a temp file readable by the shell script
          do shell script "mkdir -p local && echo " & (i as string) & " > local/copilot_tabs"
          exit repeat
        end if
      end repeat
    end tell
  end tell
end run
APPLESCRIPT

if [[ -f local/copilot_tabs ]]; then
  echo "Saved focus TABS value: $(cat local/copilot_tabs)"
else
  echo "Calibration not saved; you can rerun or set TABS manually."
fi

