import os
import re
import subprocess
from pathlib import Path


def run(cmd, env=None):
    proc = subprocess.run(
        cmd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        env=env or os.environ.copy(),
    )
    return proc


def codex_bin() -> str:
    # Prefer explicit path set by Makefile
    path = os.environ.get("CODEX_BIN")
    if path and Path(path).exists():
        return path
    return "codex"


def with_codex_home(tmpdir: Path):
    env = os.environ.copy()
    env["CODEX_HOME"] = str(tmpdir)
    return env


def test_help_displays_usage(tmp_path):
    env = with_codex_home(tmp_path)
    p = run([codex_bin(), "--help"], env=env)
    assert p.returncode == 0
    assert "Usage:" in p.stdout
    assert "codex" in p.stdout


def test_version_prints_version(tmp_path):
    env = with_codex_home(tmp_path)
    p = run([codex_bin(), "--version"], env=env)
    assert p.returncode == 0
    assert re.search(r"\d+\.\d+\.\d+", p.stdout)


def test_completion_bash_outputs_script(tmp_path):
    env = with_codex_home(tmp_path)
    p = run([codex_bin(), "completion", "bash"], env=env)
    assert p.returncode == 0
    assert len(p.stdout) > 0


def test_completion_other_shells(tmp_path):
    env = with_codex_home(tmp_path)
    for sh in ["zsh", "fish", "powershell"]:
        p = run([codex_bin(), "completion", sh], env=env)
        assert p.returncode == 0
        assert len(p.stdout) > 0


def test_subcommands_help(tmp_path):
    env = with_codex_home(tmp_path)
    for sub in [
        ["exec", "--help"],
        ["login", "--help"],
        ["logout", "--help"],
        ["mcp", "--help"],
        ["apply", "--help"],
        ["cloud", "--help"],
    ]:
        p = run([codex_bin()] + sub, env=env)
        assert p.returncode == 0
        assert "Usage:" in p.stdout or "Usage:" in p.stderr


def test_login_status_not_logged_in_with_clean_home(tmp_path):
    env = with_codex_home(tmp_path)
    p = run([codex_bin(), "login", "status"], env=env)
    # expects stderr message and non‑zero exit
    assert p.returncode != 0
    # Fresh CODEX_HOME has no auth.json, so the CLI reports an error opening it.
    assert (
        "Not logged in" in p.stderr
        or "Error checking login status" in p.stderr
        or "No such file or directory" in p.stderr
    )


def test_exec_without_prompt_errors(tmp_path):
    env = with_codex_home(tmp_path)
    p = run([codex_bin(), "exec"], env=env)
    assert p.returncode != 0
    assert "No prompt provided" in p.stderr
