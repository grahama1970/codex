import json
import os
import signal
import socket
import subprocess
import sys
import time
from pathlib import Path

import pytest


pytestmark = pytest.mark.live


def run(cmd, env=None, input=None, timeout=30):
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
    path = os.environ.get("CODEX_BIN")
    return path or "codex"


def with_codex_home(tmpdir: Path):
    env = os.environ.copy()
    env["CODEX_HOME"] = str(tmpdir)
    return env


def test_responses_api_proxy_starts_and_shuts_down(tmp_path):
    env = with_codex_home(tmp_path)
    info_path = tmp_path / "server.json"

    # Start the proxy with an ephemeral port, write server info, and enable shutdown endpoint.
    p = subprocess.Popen(
        [codex_bin(), "responses-api-proxy", "--server-info", str(info_path), "--http-shutdown"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        env=env,
    )

    try:
        # Send the API key (proxy adds 'Bearer ' itself); only [A-Za-z0-9-_] allowed.
        assert p.stdin is not None
        p.stdin.write("sk-testtoken\n")
        p.stdin.flush()

        # Wait for the server info file to appear.
        for _ in range(100):
            if info_path.exists():
                break
            time.sleep(0.05)
        assert info_path.exists(), p.stderr.read() if p.stderr else "no stderr"

        info = json.loads(info_path.read_text())
        port = info["port"]
        assert isinstance(port, int) and port > 0

        # Call /shutdown to stop the server.
        with socket.create_connection(("127.0.0.1", port), timeout=5) as s:
            req = b"GET /shutdown HTTP/1.1\r\nHost: localhost\r\n\r\n"
            s.sendall(req)
            resp = s.recv(1024)
            assert b" 200 " in resp

        p.wait(timeout=5)
        assert p.returncode == 0
    finally:
        try:
            p.kill()
        except Exception:
            pass


def test_mcp_add_list_get_remove_stdio(tmp_path):
    env = with_codex_home(tmp_path)
    name = "echoer"

    # Add a stdio server using /bin/echo (harmless) with an env var.
    r = run([codex_bin(), "mcp", "add", name, "--env", "FOO=bar", "--", "/bin/echo", "hello"], env=env)
    assert r.returncode == 0, r.stderr
    assert f"Added global MCP server '{name}'." in r.stdout

    # List servers as JSON.
    r = run([codex_bin(), "mcp", "list", "--json"], env=env)
    assert r.returncode == 0, r.stderr
    data = json.loads(r.stdout)
    assert any(entry["name"] == name for entry in data)

    # Get server details
    r = run([codex_bin(), "mcp", "get", name, "--json"], env=env)
    assert r.returncode == 0, r.stderr
    entry = json.loads(r.stdout)
    assert entry["name"] == name
    assert entry["transport"]["type"] == "stdio"

    # Remove it
    r = run([codex_bin(), "mcp", "remove", name], env=env)
    assert r.returncode == 0, r.stderr
    assert f"Removed global MCP server '{name}'." in r.stdout or f"No MCP server named '{name}' found." in r.stdout


def test_app_server_starts_and_exits_on_stdin_close(tmp_path):
    env = with_codex_home(tmp_path)
    p = subprocess.Popen(
        [codex_bin(), "app-server"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        env=env,
    )
    # Immediately close stdin; the server should exit quickly.
    assert p.stdin is not None
    p.stdin.close()
    p.wait(timeout=10)
    assert p.returncode == 0


def test_generate_ts_outputs_files(tmp_path):
    env = with_codex_home(tmp_path)
    out = tmp_path / "ts-out"
    r = run([codex_bin(), "generate-ts", "-o", str(out)], env=env)
    assert r.returncode == 0, r.stderr
    # Expect at least one generated file
    assert any(out.rglob("*.ts")), "no .ts files generated"


@pytest.mark.skipif("OPENAI_API_KEY" not in os.environ, reason="requires OPENAI_API_KEY for live model call")
def test_exec_minimal_prompt_runs_with_openai(tmp_path):
    env = with_codex_home(tmp_path)
    # Minimal prompt that should finish without complex actions.
    prompt = "Say 'hello' and stop."
    # Use JSON mode to avoid TUI and capture deterministic output events.
    r = run([codex_bin(), "exec", "--json", prompt, "-c", "model=\"gpt-4o\""], env=env, timeout=120)
    assert r.returncode == 0, r.stderr
    # Should have emitted at least one JSONL event line.
    lines = [ln for ln in r.stdout.splitlines() if ln.strip()]
    assert lines and lines[0].startswith("{")
