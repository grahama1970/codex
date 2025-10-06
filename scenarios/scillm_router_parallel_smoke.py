"""
Dry smoke for SciLLM Router/parallel. No network calls.
Passes when Router and parallel_acompletions are importable and SCILLM_ROUTER_MODELS is set.
"""
from __future__ import annotations
import os, sys


def main() -> int:
    dry = os.environ.get("SCILLM_SMOKE_DRY") == "1"
    try:
        import litellm  # type: ignore
    except Exception as e:
        print(f"error: cannot import litellm: {e}")
        return 1
    router_ok = hasattr(litellm, "Router")
    parallel_ok = hasattr(litellm, "parallel_acompletions")
    models = os.environ.get("SCILLM_ROUTER_MODELS", "").strip()
    if not router_ok:
        print("error: Router not available from litellm")
        return 1
    if not parallel_ok:
        print("error: parallel_acompletions not available from litellm")
        return 1
    if not models:
        print("error: SCILLM_ROUTER_MODELS not set")
        return 1
    if dry:
        print("ok: dry smoke passed (imports + env)")
        return 0
    # Future: add a mock adapter smoke if desired.
    print("ok: basic smoke passed (no network attempted)")
    return 0


if __name__ == "__main__":
    sys.exit(main())

