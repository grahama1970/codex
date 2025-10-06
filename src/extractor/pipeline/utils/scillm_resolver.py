from __future__ import annotations
import os
from typing import Optional, Tuple


def resolve_model(preferred: Optional[str] = None) -> Tuple[Optional[str], str]:
    """Resolve model per guide precedence.
    Returns (model, source).
    """
    if preferred and preferred.strip():
        return preferred.strip(), "arg:preferred"
    env = os.environ
    order = [
        (env.get("GM_LLM_MODEL"), "env:GM_LLM_MODEL"),
        (env.get("SCILLM_SMALL_TEXT_MODEL"), "env:SCILLM_SMALL_TEXT_MODEL"),
        (env.get("SCILLM_DEFAULT_MODEL"), "env:SCILLM_DEFAULT_MODEL"),
        (env.get("LITELLM_SMALL_TEXT_MODEL"), "env:LITELLM_SMALL_TEXT_MODEL"),
        (env.get("LITELLM_DEFAULT_MODEL"), "env:LITELLM_DEFAULT_MODEL"),
    ]
    for m, src in order:
        if m and m.strip():
            return m.strip(), src
    return None, "fallback:none"


def should_parallelize() -> bool:
    """Enable Router parallel by default when SCILLM_ROUTER_MODELS is set, unless GM_LLM_PARALLEL=0."""
    models = os.environ.get("SCILLM_ROUTER_MODELS", "").strip()
    if not models:
        return False
    if os.environ.get("GM_LLM_PARALLEL", "1") == "0":
        return False
    return True


def log_selection(model: Optional[str], source: str, api_base_present: bool, profile: Optional[str], request_id: Optional[str]) -> str:
    import json
    payload = {
        "type": "llm.select",
        "model": model,
        "source": source,
        "api_base_present": api_base_present,
        "profile": profile,
        "request_id": request_id,
    }
    line = json.dumps(payload, separators=(",", ":"))
    # Print to stderr in your call sites when appropriate
    return line

