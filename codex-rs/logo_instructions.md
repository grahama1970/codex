cxplus logo animation spec (maintainer reference)

Files
- Animated: `codex-rs/logo4.svg`
- Static: `codex-rs/logo4_static.svg` (no animation; use for README or places that strip SMIL)

Design goals
- Wordmark text first: large “cx” followed by “plus”.
- Slight spacing between “cx” and “plus” (default ≈ 1.0em; tunable).
- Cyan sweep fills each glyph completely, one letter at a time, left→right.
- When the final “s” turns cyan, a cyan “+” pops in snug to the top‑right of the “s”, spins L→R for 1s, then R→L for 1s, then pops out. The whole cycle repeats.

Implementation notes
- The animated SVG uses SMIL with per‑glyph `clipPath`s so the glyph outlines act as the mask. Each letter has a cyan rectangle that expands within its own clipPath.
- We avoid CSS variables in SMIL attributes (dur/values/begin) because many renderers don’t resolve them.
- The `<style>` block is wrapped in CDATA to keep CSS characters valid in XML.
- Reduced‑motion: animation layers are hidden when the OS preference is set, unless the root has `data-animate="force"`. A separate `logo4_static.svg` is provided.

Key coordinates (default)
- Baseline origin for the group: `translate(20, 56)`.
- Glyph positions (x): `c=0`, `x=28`, then a gap before `plus`.
- Default gap between `cx` and `plus`: `~1.0em` → `plus` starts at `x=90`.
- Per‑glyph x for overlay (must match the base): `p=90`, `l=112`, `u=126`, `s=148`.
- Plus icon position: `translate(172, -26)` (snug to the ‘s’ top‑right). Tweak this if you change the gap.

Timing
- Character sweep: six steps at 0.20s each → total ≈ 1.20s.
  - Begins at `cycle.begin`, then `+0.20s` per next glyph.
- “+” sequence (begins at `sReveal.end`):
  - Pop‑in (opacity + scale): 0.30s.
  - Spin L→R: 1.00s.
  - Spin R→L: 1.00s.
  - Pop‑out (opacity + scale): 0.20s.
- Cycle repeats indefinitely via `repeatCount="indefinite"` on the invisible controller `#cycle`.

How to tweak
- Gap between “cx” and “plus”
  1) Move the base word “plus”: search `id="plusBase"` and change `x`.
  2) Update the clipPath x values for `clipP`, `clipL`, `clipU`, `clipS` to the new letter starts.
  3) Update the cyan rect `x` for each of those letters in the `#cyanOverlay` group.
  4) Nudge the plus icon position (`id="plusAcc"` `translate(X, -26)`) so it remains snug to the ‘s’.

- Sweep duration (e.g., 1.50s total)
  - Increase each letter’s `dur` from `0.20s` to `0.25s` (six letters).
  - Optionally add a small pause by offsetting the `begin` of the “+” sequence (e.g., `+0.10s`).

- Plus placement
  - Adjust the translate in `id="plusAcc"`. Each unit is px in the group’s user space; typical nudges are 2–6 px.

Reduced‑motion and forcing animation
- If the OS has “Reduce Motion” enabled, animation layers are hidden.
- To show animation anyway when embedding inline, add `data-animate="force"` to the root `<svg>`.

Troubleshooting
- “Cyan looks like a thin bar”: ensure we are using `clipPath` per glyph (not a text mask). The animated file already uses clipPaths.
- “Everything is static”: check (a) the viewer supports SMIL, (b) OS “Reduce Motion” isn’t hiding layers, or (c) add `data-animate="force"` when embedding.
- “SVG fails to render”: XML parsers are strict — keep the `<style>` wrapped in CDATA; avoid raw `&` in comments; don’t use CSS variables in SMIL.

Acceptance checklist
- [ ] Letters fill fully with cyan, one by one from left to right.
- [ ] Gap between “cx” and “plus” feels slight (≈ 1em by default).
- [ ] When “s” completes, the “+” pops in snug to the ‘s’, spins L→R 1s, spins R→L 1s, then pops out.
- [ ] The whole sequence repeats smoothly.

Editing tips
- Open `logo4.svg` in a browser while editing and refresh to preview.
- Keep coordinates integers where possible to avoid sub‑pixel blurring.
- If changing font size or weight, update both the base text and all clipPath text elements to match.
