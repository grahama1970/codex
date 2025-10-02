#!/usr/bin/env python3
"""
Knowledge First prehook for Codex: fetch Memory context before any LLM call.

Strategy
- Try MCP stdio server (memory-agent) → list tools → call memory_search.
- Tight timeout (default 2s). If MCP fails, fall back to CLI (uv run lessons-recall).
- Return a compact JSON with titles/snippets suitable for system/tool context.

Env knobs
- MEMORY_MCP_NODE: path to node binary (default: `node` in PATH)
- MEMORY_AGENT_SERVER: path to memory server.js (default: memory/mcp/memory-agent-node/server.js)
- MEMORY_SCOPE: default scope for search (default: tabbed)
"""
from __future__ import annotations
import json, os, shlex, subprocess, sys, time
from typing import Any, Dict, List, Optional

DEF_SCOPE = os.getenv("MEMORY_SCOPE", "tabbed")


def _now_ms() -> int:
    return int(time.time() * 1000)


def _spawn_mcp() -> subprocess.Popen:
    node = os.getenv("MEMORY_MCP_NODE", "node")
    server = os.getenv("MEMORY_AGENT_SERVER", os.path.expanduser("~/workspace/experiments/memory/mcp/memory-agent-node/server.js"))
    env = os.environ.copy()
    env.setdefault("DEV_ALLOW_DEFAULT", "1")
    return subprocess.Popen([node, server], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True)


def _mcp_call_search(prompt: str, scope: str, k: int, timeout_s: float = 2.0) -> Dict[str, Any]:
    p = _spawn_mcp()
    try:
        assert p.stdin and p.stdout
        def send(obj):
            p.stdin.write(json.dumps(obj) + "\n"); p.stdin.flush()
        send({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","clientInfo":{"name":"codex-prehook","version":"1.0"}}})
        send({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}})
        send({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"memory_search","arguments":{"q":prompt,"scope":scope,"k":k}}})
        t0 = time.time()
        lines: List[str] = []
        while time.time() - t0 < timeout_s:
            line = p.stdout.readline()
            if not line:
                break
            lines.append(line)
            try:
                m = json.loads(line)
                if m.get("id") == 3:
                    content = (m.get("result") or {}).get("content") or []
                    # content is array of {type:"text", text:"<json>"}
                    for c in content:
                        if c.get("type") == "text":
                            try:
                                payload = json.loads(c.get("text") or "{}")
                                if isinstance(payload, dict) and payload.get("ok") and payload.get("data"):
                                    return payload["data"]
                            except Exception:
                                continue
                    break
            except Exception:
                continue
        raise TimeoutError("mcp_search_timeout")
    finally:
        try:
            p.kill()
        except Exception:
            pass


def _cli_recall(prompt: str, scope: str, k: int) -> Dict[str, Any]:
    cmd = f"uv run lessons-recall --q {shlex.quote(prompt)} --scope {shlex.quote(scope)} --k {k} --json"
    env = os.environ.copy()
    env.setdefault("PYTHONPATH", os.path.expanduser("~/workspace/experiments/memory/src"))
    try:
        out = subprocess.check_output(cmd, shell=True, text=True, env=env, timeout=2.0)
        return json.loads(out)
    except Exception as e:
        return {"error": str(e)}


def knowledge_first_context(prompt: str, scope: Optional[str] = None, k: int = 5, timeout_s: float = 2.0) -> Dict[str, Any]:
    """Fetch Memory context (titles/snippets) to prepend before LLM calls."""
    sc = scope or DEF_SCOPE
    # Try MCP first
    try:
        data = _mcp_call_search(prompt, sc, k, timeout_s=timeout_s)
        return {"source": "mcp", "scope": sc, "k": k, "items": data.get("items") or data}
    except Exception:
        pass
    # Fallback CLI
    data = _cli_recall(prompt, sc, k)
    return {"source": "cli", "scope": sc, "k": k, "items": (data.get("items") if isinstance(data, dict) else [])}


if __name__ == "__main__":
    q = sys.argv[1] if len(sys.argv) > 1 else "cdp puppeteer"
    ctx = knowledge_first_context(q)
    print(json.dumps(ctx, ensure_ascii=False))

