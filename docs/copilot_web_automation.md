# Copilot Web Automation (Experimental)

Goal: reduce manual copy/paste from the agent into GitHub Copilot Web by driving a logged‑in browser on macOS via AppleScript UI automation. This approach is brittle and user‑environment dependent; prefer native PR reviews when possible.

## Requirements

- macOS with Safari or Google Chrome
- Enable Accessibility for your terminal:
  - System Settings → Privacy & Security → Accessibility → allow Terminal/iTerm
- You must already be logged into GitHub in the chosen browser

## Usage

```
scripts/copilot_web_send.sh -f path/to/prompt.txt \
  -b safari                # or chrome
  -u https://github.com/copilot
  -t 1                     # TAB presses to focus input
```

The script copies your prompt file to the clipboard, brings the browser to the foreground, opens the Copilot page, sends `TAB` (to focus the input), pastes, and presses `Return`.

Notes:
- UI structure occasionally changes; adjust `-t` to find the right focus chain.
- Keep content sizes reasonable to avoid paste truncation rate limits.
- This script never stores credentials and runs only on your local machine.

## Safer Alternatives

- Prefer GitHub’s native Copilot PR Reviews (add Copilot as a reviewer on PRs). Our `scripts/copilot_submit_async.sh` and `copilot_watch.sh` already automate opening/updating PRs and watching for Copilot comments.

## Linux/Windows

- Linux: you could re‑implement the same idea using `xdotool` or `ydotool` to paste and submit. Not provided here due to distro variance.
- Windows: use PowerShell + UIAutomation or AutoHotkey. Not provided.

## Caveats

- UI automation is brittle. Treat this as a convenience tool, not a CI path.
- Respect GitHub Terms of Service. Avoid rapid, repeated automation that resembles botting.

