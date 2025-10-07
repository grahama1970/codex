import os
import pathlib
import subprocess
import pytest

BIN = pathlib.Path("dist/bin/codex")


@pytest.mark.skipif(os.environ.get("CONTEXT_FEATURE") != "1", reason="CONTEXT_FEATURE not set")
def test_context_summary_ndjson_emitted():
    assert BIN.exists(), "codex binary missing (run make package)"
    runs_dir = pathlib.Path(".codex/runs")
    before = set(runs_dir.glob("*-events.ndjson")) if runs_dir.exists() else set()

    env = os.environ.copy()
    env["CONTEXT_DEBUG"] = "1"
    # Assume user enabled provider=arango in config.toml when opting into this scenario.
    proc = subprocess.run([str(BIN), "exec", "Summarize this repository"], capture_output=True, text=True, env=env)
    assert proc.returncode in (0, 1, 5), f"unexpected exit code {proc.returncode}"

    after = set(runs_dir.glob("*-events.ndjson")) if runs_dir.exists() else set()
    new_files = list(after - before)
    assert new_files, "expected a new events.ndjson file"
    with open(new_files[-1], "r", encoding="utf-8") as f:
        lines = f.readlines()
    # Ensure a single completed context.summary v2 line exists
    ctx_lines = [ln for ln in lines if '"kind":"context.summary"' in ln]
    assert ctx_lines, "context.summary line missing"
    import json
    obj = json.loads(ctx_lines[0])
    assert obj.get("version") == 2, "expected version=2"
    assert "budget" in obj
    # If fixture present and provider=Arango, evidence_items should be >= 1
    import os as _os
    if _os.environ.get("CONTEXT_MCP_FIXTURE") and "Arango" in obj.get("provider", ""):
        assert obj.get("evidence_items", 0) >= 1
