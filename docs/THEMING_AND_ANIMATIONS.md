# cxplus Theming and Animations

This fork adds tasteful, low‑strain theming and animations designed for long technical sessions.

## Theme selection

- Live session toggles (no restart)
  - `/light` — switch to light theme for this session
  - `/dark` — switch to dark theme for this session
  - `/dark-dim` — lower‑pop dark variant (session)
- Persistent defaults
  - `export CXPLUS_THEME=light|dark|dark-dim`
  - `~/.codex/config.toml` brand overrides:
    ```toml
    [tui.brand]
    title_color = "#E6EDF3"
    accent_color = "#7AA2F7"
    ```
- Role colors used today
  - Banner title/accent, info/error messages, links (dark defaults keep cyan to preserve snapshots)

## Animation controls

- Reduced motion
  - `export CXPLUS_REDUCE_MOTION=1` → static tip lines and low‑motion variants
- Spinner tips (Claude‑style verbs)
  - `/anim-tips-on` or `/anim-tips-off` (session)
  - `CXPLUS_SPINNER_TIPS=1|0` (env)
  - `~/.codex/config.toml`:
    ```toml
    [anim]
    spinner_tips = true
    tip_interval_secs = 2
    tips = ["Analyzing…","Indexing…","Linking…","Optimizing…","Syncing…"]
    ```

## Widgets available

- Orbit spinner (indeterminate): nucleus + orbit glyph; shows a rotating tip when enabled.
- Shimmer gauge (determinate): brand accent bar with subtle shimmer.

Both widgets:
- Honor reduced motion
- Degrade gracefully in non‑TTY output paths (use textual status)
- Prefer 8–12 fps; rendering is driven by a shared tick (no internal timers)

## ANSI palette guidance (dark)

Suggested ANSI mapping to align terminals with cxplus dark theme:
```
0 #0E1216  1 #E16D76  2 #19B28E  3 #E6B450
4 #7AA2F7  5 #A48CF2  6 #2FD4CB  7 #D4D9E1
8 #2A3038  9 #F07F89 10 #22C8A3 11 #F0C364
12 #8CB1FA 13 #C1A3FF 14 #66E0DA 15 #F0F3F8
```
Keep brights moderated; bright-black (8) distinct from background.

## Notes
- Avoid pure black/white to reduce halation (`#12161B` family recommended for backgrounds).
- Accents in the blue‑violet→cyan range are modern and CVD‑robust.
- Use color sparingly: focus, selection, and state cues—avoid large, saturated fills.
