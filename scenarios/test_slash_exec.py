import os
import subprocess

import pytest


pytestmark = pytest.mark.live


def run(cmd, env=None, timeout=60):
    return subprocess.run(
        cmd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        env=env or os.environ.copy(),
        timeout=timeout,
    )


def codex_bin():
    return os.environ.get("CODEX_BIN") or "codex"


def env_with_dist_config():
    env = os.environ.copy()
    env["CODEX_HOME"] = os.path.abspath("dist/config")
    return env


def test_slash_help_and_status():
    env = env_with_dist_config()
    p1 = run([codex_bin(), "exec", "/help"], env=env)
    assert p1.returncode == 0
    assert "Slash commands:" in p1.stderr

    p2 = run([codex_bin(), "exec", "/status"], env=env)
    assert p2.returncode == 0
    assert "provider=" in p2.stderr

