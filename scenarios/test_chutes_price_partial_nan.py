import os
import subprocess
import pathlib
import json
import copy
import pytest


@pytest.mark.live
def test_chutes_partial_nan_no_relax(tmp_path):
    """When at least one valid priced candidate remains, do NOT relax price cap."""
    repo_root = pathlib.Path(__file__).resolve().parents[1]
    codex_bin = repo_root / "dist" / "bin" / "codex"
    assert codex_bin.exists(), "codex binary missing; run make build"

    base_fixture = json.loads((repo_root / "tests" / "fixtures" / "chutes_catalog_sample.json").read_text())
    modified = copy.deepcopy(base_fixture)
    # Force first item to NaN-like output price; keep second valid
    modified["items"][0]["current_estimated_price"]["per_million_tokens"]["output"]["usd"] = "   "
    fixture_path = tmp_path / "partial_nan_catalog.json"
    fixture_path.write_text(json.dumps(modified))

    env = os.environ.copy()
    env["CHUTES_CATALOG_FIXTURE"] = str(fixture_path)
    env["CHUTES_DISCOVERY_DEBUG"] = "1"
    cmd = [
        str(codex_bin),
        "chutes",
        "recommend",
        "--min-params",
        "10000000000",
        "--max-output-ppm",
        "2.0",
    ]
    proc = subprocess.run(cmd, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    assert proc.returncode == 0, proc.stderr
    assert proc.stdout.strip() != ""
    assert "relaxing price cap" not in proc.stderr.lower(), f"unexpected relaxation triggered: {proc.stderr}"

