import os
import subprocess
from pathlib import Path

import pytest


pytestmark = pytest.mark.live


def run(cmd, env=None):
    proc = subprocess.run(
        cmd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        env=env or os.environ.copy(),
        input=None,
    )
    return proc


def run_with_stdin(cmd, data: str, env=None):
    proc = subprocess.run(
        cmd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        env=env or os.environ.copy(),
        input=data,
    )
    return proc


def codex_bin() -> str:
    path = os.environ.get("CODEX_BIN")
    return path or "codex"


def with_codex_home(tmpdir: Path):
    env = os.environ.copy()
    env["CODEX_HOME"] = str(tmpdir)
    return env


def test_login_status_roundtrip_with_api_key(tmp_path):
    env = with_codex_home(tmp_path)
    # Pipe a fake API key into login --with-api-key (writes auth.json only)
    key = "sk-live-testing-key"
    p_login = run_with_stdin([codex_bin(), "login", "--with-api-key"], key, env=env)
    assert p_login.returncode == 0, p_login.stderr
    assert "Successfully logged in" in p_login.stderr

    p_status = run([codex_bin(), "login", "status"], env=env)
    assert p_status.returncode == 0
    assert "Logged in using an API key" in p_status.stderr

    p_logout = run([codex_bin(), "logout"], env=env)
    assert p_logout.returncode == 0
    assert ("Successfully logged out" in p_logout.stderr) or ("Not logged in" in p_logout.stderr)
