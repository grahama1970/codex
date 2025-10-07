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
    assert any('"kind":"context.summary"' in ln for ln in lines), "context.summary line missing"

