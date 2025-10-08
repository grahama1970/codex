# Feature Recipe: Copilot Web Prompt Delivery (Reduce Manual Copy/Paste)

## Why
Reduce friction and human error when sending large, structured review prompts to Copilot Web. Keep the workflow user‑initiated and ToS‑safe while providing fast, repeatable ergonomics.

## Goals
- One‑command, user‑local action to deliver a prepared prompt into Copilot Web.
- Reuse existing PR‑based Copilot workflows; this is a convenience path, not CI.
- Non‑destructive: no secrets stored; no headless login.

## Non‑Goals
- Bypassing GitHub ToS, anti‑automation, or rate limits.
- Full headless operation against protected pages.

## Users/Stories
- As a maintainer, I want to send a curated, multi‑file review prompt to Copilot Web without copy/paste.
- As a contributor, I want a repeatable prompt template that references my feature branch and file paths.

## Constraints & Guardrails
- Local machine only (macOS first). Explicit user action (no background loops).
- Provide visible docs + warnings. Fail closed on missing permissions.

## Success Criteria (MVP)
- `make copilot-web-send FILE=REVIEW_REQUEST.md` opens Copilot Web, focuses input, pastes, submits.
- Works in Safari/Chrome when the user is already logged in.
- Logs a clear success/failure message.

## Architecture (MVP)
- Prompt assembly: curate in `REVIEW_REQUEST.md` with anchors: branch, file paths, questions.
- Delivery: `scripts/copilot_web_send.sh` (AppleScript) — paste and submit.
- Optional: a tiny helper to synthesize a prompt from repo metadata.

## Slices
- v1 (MVP): AppleScript sender + docs (DONE).
- v1.1: Prompt templater `scripts/copilot_prompt_build.sh` that pulls branch, repo, and file list, writes to `local/copilot_prompt.txt`, then calls sender.
- v2 (optional): Linux (xdotool) sender variant.

## Risks
- UI is brittle; TAB count may change per user. Mitigate with `-t` parameter and docs.
- Anti‑automation heuristics: keep usage infrequent and human‑initiated.

## Telemetry
- Local log lines only (no analytics). Optionally record a timestamp in `local/` for the last send.

## Rollout
- Behind explicit make target. Documented in `docs/copilot_web_automation.md`.

## Acceptance Checklist
- [ ] Works on macOS Safari and Chrome with Accessibility enabled.
- [ ] Clear error message if Accessibility permission not granted.
- [ ] No plaintext secrets written to disk.
- [ ] README/docs updated.

## Open Questions
- Should we add a Slack notify on send? (opt‑in)
- Add a safe “dry‑run” that only focuses input but doesn’t press Enter?

