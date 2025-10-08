import os
import subprocess
from pathlib import Path

import pytest


pytestmark = pytest.mark.live


def run(cmd, env=None, input=None, timeout=60):
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


@pytest.mark.skip(reason="covered by chutes subcommand tests; use explicit provider only when configured")
def test_exec_through_chutes(tmp_path):
    env = env_with_dist_config()
    # Allow overriding base via env; config writes provider with default base.
    prompt = "Say 'hello from chutes' and stop."
    r = run(
        [
            codex_bin(),
            "exec",
            "--json",
            prompt,
            "-c",
            "model_provider=\"chutes\"",
            "-c",
            f"model=\"{os.environ['CHUTES_MODEL']}\"",
        ],
        env=env,
        timeout=180,
    )
    assert r.returncode == 0, r.stderr
    lines = [ln for ln in r.stdout.splitlines() if ln.strip()]
    assert lines and lines[0].startswith("{")
