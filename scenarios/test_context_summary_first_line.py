import pathlib
import subprocess
import json
import pytest

BIN = pathlib.Path("dist/bin/codex")


@pytest.mark.skipif(not BIN.exists(), reason="codex binary missing (run make package)")
def test_context_summary_is_first_ndjson_line():
    runs_dir = pathlib.Path(".codex/runs")
    before = set(runs_dir.glob("*-events.ndjson")) if runs_dir.exists() else set()

    proc = subprocess.run([str(BIN), "exec", "hello"], capture_output=True, text=True)
    assert proc.returncode in (0, 1, 5)

    after = set(runs_dir.glob("*-events.ndjson")) if runs_dir.exists() else set()
    new_files = sorted(after - before)
    assert new_files, "expected a new events.ndjson file"

    with open(new_files[-1], "r", encoding="utf-8") as f:
        first = f.readline().strip()
    assert first, "events.ndjson is empty"
    obj = json.loads(first)
    assert obj.get("kind") == "context.summary", "first NDJSON line should be context.summary"
    assert obj.get("version") == 2
