import os
import pathlib
import subprocess
import json
import pytest

BIN = pathlib.Path("dist/bin/codex")


@pytest.mark.skipif(os.environ.get("CONTEXT_FEATURE") != "1", reason="CONTEXT_FEATURE not set")
def test_reflow_metrics_when_plan_empty():
    assert BIN.exists(), "codex binary missing (run make package)"
    runs_dir = pathlib.Path(".codex/runs")
    before = set(runs_dir.glob("*-events.ndjson")) if runs_dir.exists() else set()

    env = os.environ.copy()
    env["CONTEXT_DEBUG"] = "1"
    # Assume provider=arango with a fixture and non-zero plan_pct in config
    prompt = "Quick check: print hello world"
    proc = subprocess.run([str(BIN), "exec", prompt], capture_output=True, text=True, env=env)
    assert proc.returncode in (0, 1, 5)

    after = set(runs_dir.glob("*-events.ndjson")) if runs_dir.exists() else set()
    new_files = sorted(after - before)
    assert new_files, "expected a new events.ndjson file"
    with open(new_files[-1], "r", encoding="utf-8") as f:
        lines = f.readlines()
    ctx = [json.loads(ln) for ln in lines if '"kind":"context.summary"' in ln]
    assert ctx, "missing context.summary"
    obj = ctx[0]
    # If plan tokens are 0 but plan_pct > 0, reflowed_from.plan should be non-zero
    plan_pct = int(obj.get("budget", {}).get("plan_pct", 0))
    section = obj.get("section_tokens", {})
    reflowed = obj.get("reflowed_from", {})
    if plan_pct > 0 and int(section.get("plan", 0)) == 0:
        assert int(reflowed.get("plan", 0)) > 0
