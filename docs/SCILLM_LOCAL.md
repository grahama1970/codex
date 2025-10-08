# Using scillm (litellm) from a local checkout

This project can interoperate with your local scillm (distribution) whose Python import name remains `litellm`. Downstream Python projects can point at your local checkout during development or pin to a file URL.

Key points
- Distribution name: `scillm` (what appears in `pyproject.toml` dependencies and `pip/uv` installs).
- Import name: `litellm` (e.g., `from litellm import Router`).
- Console scripts: `scillm`, `scillm-proxy` (installed into the venv).

Recommended patterns

1) Editable development in a venv (live edits reflect immediately)
```
uv venv .venv
source .venv/bin/activate
uv pip install -U pip
uv pip install -e /home/graham/workspace/experiments/litellm
# or
uv add --editable /home/graham/workspace/experiments/litellm
```

2) Declarative in a downstream `pyproject.toml` (PEP 508 direct URL)
```toml
[project]
dependencies = [
  "scillm @ file:///home/graham/workspace/experiments/litellm",
]
# With extras
# dependencies = ["scillm[proxy] @ file:///home/graham/workspace/experiments/litellm"]
```
Then:
```
uv sync
```

3) One‑off non‑editable (wheel built from local path)
```
uv pip install /home/graham/workspace/experiments/litellm
```

Verification quick checks
```
python -c "import litellm,importlib.metadata as m;print(litellm.__file__, m.version('scillm'))"
which scillm || where scillm
which scillm-proxy || where scillm-proxy
```

Notes
- Use file URLs with three slashes: `file:///home/...` (not four).
- On Windows, prefer `file:///C:/path/to/litellm` with forward slashes.
- For reproducibility across machines, publish a wheel or serve a local index (devpi/simple) instead of a machine‑local file URL.

