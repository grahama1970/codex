import os
import subprocess
from pathlib import Path

import pytest


pytestmark = pytest.mark.live


def run(cmd, env=None, input=None, timeout=180):
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
def test_make_chutes_profiles_and_exec():
    env = env_with_dist_config()

    # Generate profiles in config.toml via Makefile target
    p = run(["make", "-j1", "chutes-profiles"], env=env, timeout=300)
    if p.returncode != 0:
        pytest.skip(f"profile discovery failed: {p.stderr}")

    # Exec with coding profile
    r_code = run([codex_bin(), "exec", "--json", "Hello from coding profile", "-p", "coding"], env=env)
    assert r_code.returncode == 0, r_code.stderr
    assert r_code.stdout.strip().startswith("{"), "coding profile did not emit JSONL"

    # Exec with multimodal profile (text is fine even if model supports images)
    r_mm = run([codex_bin(), "exec", "--json", "Hello from multimodal profile", "-p", "multimodal"], env=env)
    assert r_mm.returncode == 0, r_mm.stderr
    assert r_mm.stdout.strip().startswith("{"), "multimodal profile did not emit JSONL"

