#!/usr/bin/env python3
"""
SciLLM self-check: prints JSON diagnostics without making network calls.
Reports: litellm import path, Router/parallel availability, router models configured,
and a resolved model+source based on environment precedence.
"""
from __future__ import annotations
import json, os, sys, importlib, types


def import_path(mod: types.ModuleType) -> str:
    try:
        import inspect
        return inspect.getfile(mod)
    except Exception:
        return str(mod)


def try_import(name: str):
    try:
        return importlib.import_module(name)
    except Exception:
        return None


def resolve_model() -> tuple[str|None, str]:
    # Precedence: CLI flag (not known here) → GM_LLM_MODEL → SCILLM_* → LITELLM_* → None
    env = os.environ
    candidates = [
        (env.get("GM_LLM_MODEL"), "env:GM_LLM_MODEL"),
        (env.get("SCILLM_SMALL_TEXT_MODEL"), "env:SCILLM_SMALL_TEXT_MODEL"),
        (env.get("SCILLM_DEFAULT_MODEL"), "env:SCILLM_DEFAULT_MODEL"),
        (env.get("LITELLM_SMALL_TEXT_MODEL"), "env:LITELLM_SMALL_TEXT_MODEL"),
        (env.get("LITELLM_DEFAULT_MODEL"), "env:LITELLM_DEFAULT_MODEL"),
    ]
    for model, source in candidates:
        if model and model.strip():
            return model.strip(), source
    return None, "fallback:none"


def main() -> int:
    out: dict[str, object] = {}

    litellm = try_import("litellm")
    out["litellm_present"] = bool(litellm)
    out["litellm_path"] = import_path(litellm) if litellm else None

    router = None
    if litellm:
        router = getattr(litellm, "Router", None)
        par = getattr(litellm, "parallel_acompletions", None)
        out["router_available"] = router is not None
        out["parallel_available"] = par is not None
    else:
        out["router_available"] = False
        out["parallel_available"] = False

    models_env = os.environ.get("SCILLM_ROUTER_MODELS", "").strip()
    out["router_models_configured"] = bool(models_env)
    out["router_models_raw"] = models_env

    model, source = resolve_model()
    out["selected_model"] = model
    out["selected_source"] = source

    # Simple notes
    notes = []
    if os.environ.get("SCILLM_DEV_PATH"):
        notes.append("SCILLM_DEV_PATH active (dev override)")
    out["notes"] = notes

    print(json.dumps(out, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    sys.exit(main())

