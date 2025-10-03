# Prehook Augment Injection (cx‑plus)

Goal: inject memory context safely and predictably into the LLM call when the prehook returns `decision=augment`.

Principles
- Safety first: inject as a system preamble, not as user text; strip prompt steering.
- Budgets: cap injected context to ~512–768 tokens; truncate with ellipses; de‑dup similar titles.
- Deterministic: preserve item order; normalize fields `{title, content, why}`.

Proposed default (safe)
- Inject as a system block:
  ```
  System:
  Memory context (top {N}):
  1) [title] — why\ncontent…
  2) …
  ```
- Token cap: `augment_max_tokens = 512` (approximate tokenizer or byte fallback with margin).
- Sanitization: remove control strings (e.g., `You are`, `System:` lines in content); redact common secret patterns; strip URL queries.

Config (in ~/.cx-plus/config.toml)
```
[prehook]
augment_inject = true
augment_max_tokens = 512
augment_style = "system"   # or "user" (not recommended)
```

Implementation notes
- Prefer a proper tokenizer (e.g., cl100k) when available; otherwise byte-limit each item to keep total ≤ 64 KiB.
- De‑dup by normalized title (case/space folded) before injection.
- Never log the injected text; log only counts and `latency_ms`.

Tests (deterministic)
- Given 6 items, ensure only top 5 are injected in order.
- Ensure control strings are stripped and secrets are redacted.
- Enforce token/byte truncation; end with `…`.

