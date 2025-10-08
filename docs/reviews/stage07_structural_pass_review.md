# Stage 07 Structural Pass — Comprehensive Review Deliverable

This document captures direct answers to clarifying questions, concrete patch proposals (as unified diffs), rationale, and optional future enhancements for Stage 07 (Structural Pass, Orchestrator, Requirements Miner). It is implementation‑agnostic and can be applied to the target project when the corresponding files/paths exist.

---

## Direct Answers to the 7 Clarifying Questions

1) Structural pass sufficiency & edge cases
- Baseline is OK for first‑level reproducibility, but does not cover multi‑column reflow, rotated blocks, sidebars, math blocks, or header/footer de‑duplication.
- Edge cases to add to tests:
  - Multi‑column PDF reading order
  - Paragraphs exceeding `STAGE07_MAX_PARAGRAPH_CHARS`
  - Sections with only tables (no text)
  - Tables with empty `pandas_df` vs. rows/cols fallback
  - Rotated page blocks (preserve or flag)
  - Mixed list/paragraph blocks (don’t lose bullets)
  - Math‑heavy passages (avoid over‑segmentation)
- Recommendation: Add explicit tests and note baseline limitations in docs.

2) Table merge heuristics & scoring
- Suggested deterministic refinements:
  - Minimum column count ≥ 2 before considering merge.
  - Penalize generic headers (all ≤ 3 chars) so header similarity doesn’t dominate.
  - Optional “text gap” heuristic (vertical gap normalized by page height) – penalize large gaps.
  - Negative feature for extreme row_count disparity (e.g., 1 vs. 200+).
  - Keep a global row cap and guard on column count mismatch after merge.
- Keep strict/assist/llm modes; deterministic early rejection is correct.

3) Schema shape adequacy
- Standardize section object:
  - Required: `id`, `title`, `section_hash`, `section_content_hash` (alias), `reflowed_text`, `blocks[]`, `tables[]`, `figures[]`, `table_merge.{mode,candidates[],auto_merged[],thresholds}`.
  - Paragraph block: `{ type: "paragraph", text: str, source: {pages: [int], block_ids:[str]} }`.
  - Table: `{ table_id, table_hash, row_count, col_count, page_span[], columns[], pandas_df[], pandas_metrics }`.
- Emit a schema descriptor artifact `07_reflow_schema_manifest.json`.

4) Plugin interface
- Keep decorator registry; add:
  - Typed Protocol and runtime check for `PipelineState` in/out.
  - Deterministic plugin ordering via `--plugins` or default.
  - Invariant validator after each plugin to ensure required keys remain.
  - `plugin_versions` map persisted in artifacts.
  - (Optional) per‑plugin timeout hook.

5) Resume gating
- Compare: manifest hash, deterministic hash, plugin list, and plugin_versions between runs.
- Add `07_resume_token.json` with `{ hash, plugins, plugin_versions, sizes, generated_at }`.
- `STAGE07_FORCE_RERUN=1` env override for operator control.

6) Legacy transition
- Shim delegates to orchestrator via subprocess; log DeprecationWarning and plan removal after one release.
- Ensure micro‑steps remain read‑only; all new logic in plugins or structural pass.

7) Tests (additional recommended)
- `test_07_section_hash_stability` (same input → same hash)
- `test_07_plugin_version_mismatch_forces_rerun`
- `test_07_false_positive_guard` (same headers, wide gap → not merged)
- `test_07_low_conf_table_skip` (density 0.0 & row_count 1)
- `test_07_requirements_plugin_vs_standalone`
- `test_07_llm_mode_decisions_file_skip`
- `test_07_resume_token_regression`

---

## Unified Diffs (proposed patches)

> NOTE: Paths are illustrative. Apply to your project when these modules exist.

### A) Structural pass refinements

```diff
*** Begin Patch
*** Update File: src/extractor/pipeline/steps/07_structural_pass.py
@@
-    score = (
+    # Penalize generic, very short headers so they don’t dominate matches
+    short_cols_penalty = 0.0
+    def _generic(cols):
+        if not cols: return 0
+        short = sum(1 for c in cols if isinstance(c,str) and len(c.strip()) <= 3)
+        return short / len(cols)
+    gen_factor = max(_generic((t1.get("pandas_metrics") or {}).get("columns") or t1.get("columns") or []),
+                     _generic((t2.get("pandas_metrics") or {}).get("columns") or t2.get("columns") or []))
+    if gen_factor > 0.6:
+        short_cols_penalty = 0.15
+
+    score = (
         0.40 * header_sim +
         0.25 * iou_x +
         0.20 * page_prox +
         0.15 * role_score
     )
+    if short_cols_penalty: score -= short_cols_penalty
+    if score < 0: score = 0.0
@@
-        decision = "reject"
+        decision = "reject"
         r1,c1 = _rows_cols(t1); r2,c2 = _rows_cols(t2)
-        # existing logic...
+        if feats["page_delta"] > 1 or c1 != c2 or c1 < 1:
+            decision = "reject"
+        elif max(r1,r2) > 0 and min(r1,r2) == 0:
+            decision = "reject"
+        else:
+            # existing thresholds → strict/assist/llm
             ...
@@
-        out_sec["section_hash"] = sha256(...)
+        out_sec["section_hash"] = sha256(...)
+        out_sec["section_content_hash"] = out_sec.get("section_hash")
*** End Patch
```

### B) Structural pass: emit candidate file

```diff
*** Begin Patch
*** Update File: src/extractor/pipeline/steps/07_structural_pass.py
@@
     return {
         "sections": processed,
         "metrics": metrics,
         "diagnostics": [],
         "run_id": datetime.now().strftime("%Y%m%d%H%M%S"),
         "hash": h.hexdigest(),
+        "table_merge_candidates": [
+            c for s in processed for c in (s.get("table_merge", {}).get("candidates") or [])
+        ]
     }
*** End Patch
```

### C) Orchestrator: plugin typing, schema validation, candidate persist

```diff
*** Begin Patch
*** Update File: src/extractor/pipeline/steps/07_orchestrator.py
@@
+from typing import Protocol, runtime_checkable
+@runtime_checkable
+class PluginProtocol(Protocol):
+    def __call__(self, state: PipelineState, ctx: Dict[str, Any]) -> PipelineState: ...
@@
     for name in ordered_plugins:
         fn = PLUGIN_REGISTRY.get(name)
         if not fn:
             ...
+        if not isinstance(fn, PluginProtocol):
+            ...  # best‑effort runtime check
         t0 = time.monotonic()
         ...
@@ persist candidates
+    try:
+        all_cands = struct_res.get("table_merge_candidates") or []
+        if all_cands:
+            (json_out/"07_table_merge_candidates.json").write_text(json.dumps({
+                "count": len(all_cands), "mode": os.getenv("STAGE07_TABLE_MERGE_MODE","strict"),
+                "candidates": all_cands}, indent=2))
+    except Exception as e:
+        state.diagnostics.append({"phase":"persist_candidates","error":str(e)})
@@ validate
+    required = {"id","title","blocks","tables","reflowed_text","section_hash"}
+    missing = []
+    for i,s in enumerate(state.sections):
+        miss = sorted(k for k in required if k not in s)
+        if miss: missing.append({"section_index":i,"missing":miss})
+    if missing:
+        (json_out/"07_reflow_schema_issues.json").write_text(json.dumps({"missing":missing}, indent=2))
*** End Patch
```

### D) Orchestrator: resume token (hash, plugins, versions, sizes)

```diff
*** Begin Patch
*** Update File: src/extractor/pipeline/steps/07_orchestrator.py
@@
-        det_side = {"hash": state.deterministic_hash, ...}
+        det_side = {"hash": state.deterministic_hash, "sections": len(state.sections),
+                    "plugins": ordered_plugins,
+                    "plugin_versions": {p: PLUGIN_VERSIONS.get(p) for p in ordered_plugins}}
         (json_out/"deterministic.json").write_text(json.dumps(det_side, indent=2))
+        sizes = {}
+        for fn in ["07_reflowed.json","07_reflow_manifest.json","deterministic.json"]:
+            fp = json_out/fn
+            if fp.exists(): sizes[fn] = fp.stat().st_size
+        (json_out/"07_resume_token.json").write_text(json.dumps({
+            "hash": det_side["hash"], "plugins": det_side["plugins"],
+            "plugin_versions": det_side["plugin_versions"], "sizes": sizes,
+            "generated_at": datetime.now().isoformat()
+        }, indent=2))
*** End Patch
```

### E) run_all: resume gating strengthened

```diff
*** Begin Patch
*** Update File: src/extractor/pipeline/run_all.py
@@
-    if resume and stage_completed(...):
-        try:
-            mf = json.loads(manifest.read_text()); rf = json.loads(reflow.read_text())
-            if mf.get("hash") == rf.get("deterministic_hash"):
-                skip07 = True
+    resume_token = results/stage07/"json_output"/"07_resume_token.json"
+    if resume and stage_completed(...) and resume_token.exists():
+        try:
+            mf = json.loads(manifest.read_text()); rf = json.loads(reflow.read_text())
+            det = json.loads(det_file.read_text()); rt = json.loads(resume_token.read_text())
+            hash_ok = (mf.get("hash") == rf.get("deterministic_hash") == det.get("hash") == rt.get("hash"))
+            env_plugins = os.getenv("STAGE07_PLUGINS",""").strip().split(",") if os.getenv("STAGE07_PLUGINS") else mf.get("plugins",[])
+            versions_ok = rt.get("plugin_versions") == det.get("plugin_versions")
+            skip07 = hash_ok and versions_ok and env_plugins == mf.get("plugins",[])
         except Exception:
             skip07 = False
*** End Patch
```

### F) Shim deprecation & G) Requirements note

See the proposal in the main response for exact diffs.

---

## Tests (example)

See `tests/pipeline/test_07_hash_and_requirements_parity.py` in the main response for a minimal, self‑contained example.

---

## Rationale & Tradeoffs
- Reduced merge false positives; accept slight increase in ambiguous pairs.
- Clear schema invariants; better resume gating.
- Candidate persistence enables offline analysis.

## Optional Future Enhancements
1) Multi‑column reading order
2) Table provenance enrichment
3) Row‑level hashing for incremental diffs
4) Embedding cache keyed by table/section hash
5) Human adjudication export for ambiguous merges
