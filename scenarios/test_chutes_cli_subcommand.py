import json
import os
import subprocess
from pathlib import Path

import pytest


pytestmark = pytest.mark.live


def run(cmd, env=None, input=None, timeout=120):
    proc = subprocess.run(
        cmd,
        input=input,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        env=env or os.environ.copy(),
        timeout=timeout,
    )
    return proc


def codex_bin() -> str:
    return os.environ.get("CODEX_BIN") or "codex"


def env_with_dist_config():
    env = os.environ.copy()
    env["CODEX_HOME"] = str(Path("dist/config").resolve())
    return env


@pytest.mark.skipif("CHUTES_API_KEY" not in os.environ, reason="requires CHUTES_API_KEY")
def test_chutes_recommend_and_exec(tmp_path):
    env = env_with_dist_config()

    # First try strict (>=70B, multi-modal). If no candidate, fallback to a smaller min.
    rec = run([codex_bin(), "chutes", "recommend", "--json"], env=env)
    if rec.returncode != 0:
        rec = run([codex_bin(), "chutes", "recommend", "--json", "--min-params", "10000000"], env=env)
    if rec.returncode != 0:
        pytest.skip("No suitable Chutes model found by CLI recommend")
    data = json.loads(rec.stdout)
    assert isinstance(data, dict)
    name = data.get("name")
    assert isinstance(name, str) and name

    # Run exec via subcommand (force chat wire API). Use small min to avoid catalog fluctuations.
    exe = run([codex_bin(), "chutes", "exec", "--json", "--wire-api", "chat", "--min-params", "10000000", "Say 'hello from chutes subcommand'"], env=env, timeout=240)
    assert exe.returncode == 0, exe.stderr
    lines = [ln for ln in exe.stdout.splitlines() if ln.strip()]
    assert lines and lines[0].startswith("{"), "did not emit JSONL events"
