import os
from pathlib import Path
import subprocess
import json

import pytest

from chutes_utils import fetch_chutes_catalog, pick_best_cheapest_multimodal


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


def with_codex_home(tmpdir: Path):
    env = os.environ.copy()
    env["CODEX_HOME"] = str(tmpdir)
    return env


@pytest.mark.skipif("CHUTES_API_KEY" not in os.environ, reason="requires CHUTES_API_KEY for catalog fetch")
def test_autodiscover_and_exec_through_chutes(tmp_path):
    env = with_codex_home(tmp_path)
    api_key = os.environ["CHUTES_API_KEY"]
    catalog_base = os.environ.get("CHUTES_CATALOG_BASE")  # optional override if needed

    items = fetch_chutes_catalog(api_key, base_url=catalog_base)
    picked = pick_best_cheapest_multimodal(items, min_params=70_000_000_000)
    if picked is None:
        pytest.skip("No suitable Chutes model found (>=70B, multi-modal) in catalog")

    catalog_id, meta = picked
    # Chutes expects OpenAI-compatible id prefixed with 'openai/'
    model_id = f"openai/{catalog_id}"

    prompt = "Say 'hello from autodiscover' and stop."
    r = run(
        [
            codex_bin(),
            "exec",
            "--json",
            prompt,
            "-c",
            "model_provider=\"chutes\"",
            "-c",
            f"model=\"{model_id}\"",
        ],
        env=env,
        timeout=240,
    )
    assert r.returncode == 0, r.stderr
    lines = [ln for ln in r.stdout.splitlines() if ln.strip()]
    assert lines and lines[0].startswith("{"), "did not emit JSONL events"
