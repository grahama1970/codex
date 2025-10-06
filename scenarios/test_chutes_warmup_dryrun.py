import os
import subprocess
import pathlib
import pytest


@pytest.mark.live
def test_chutes_warmup_dryrun():
    repo_root = pathlib.Path(__file__).resolve().parents[1]
    codex_bin = repo_root / "dist" / "bin" / "codex"
    assert codex_bin.exists(), "codex binary missing; run make package"

    env = os.environ.copy()
    # No API key required
    env.pop("CHUTES_API_KEY", None)
    env["CHUTES_WARMUP_DRYRUN"] = "1"
    cmd = [
        str(codex_bin),
        "chutes",
        "warmup",
        "--secs",
        "1",
        "--model",
        "openai/alpha/model-small",
    ]
    proc = subprocess.run(cmd, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    assert proc.returncode == 0, proc.stderr
    assert proc.stdout.strip().startswith("warmup: ok"), proc.stdout

