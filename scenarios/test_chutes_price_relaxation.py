import os
import subprocess
import sys
import pathlib
import pytest


@pytest.mark.live
def test_chutes_price_nan_relaxation(tmp_path):
    """Uses the all-NaN fixture to force price-cap relaxation and asserts the stderr notice."""
    repo_root = pathlib.Path(__file__).resolve().parents[1]
    codex_bin = str(repo_root / "dist" / "bin" / "codex")
    assert pathlib.Path(codex_bin).exists(), "compiled codex binary not found; run make build"

    env = os.environ.copy()
    env["CHUTES_CATALOG_FIXTURE"] = str(repo_root / "tests" / "fixtures" / "chutes_catalog_all_nan.json")
    env["CHUTES_DISCOVERY_DEBUG"] = "1"
    # Set a price cap to trigger the relaxation when all output prices are NaN
    cmd = [
        codex_bin,
        "chutes",
        "recommend",
        "--min-params",
        "10000000000",
        "--max-output-ppm",
        "2.0",
    ]
    proc = subprocess.run(cmd, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    # We expect a non-empty stdout (a model id) after relaxation and the stderr notice line
    assert proc.returncode == 0, f"unexpected exit: {proc.returncode}, stderr={proc.stderr}"
    assert proc.stdout.strip() != "", "no model printed after relaxation"
    assert "relaxing price cap" in proc.stderr.lower(), f"relaxation notice missing: {proc.stderr}"
