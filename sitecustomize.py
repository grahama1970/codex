"""
Dev override: if SCILLM_DEV_PATH is set, prepend it to sys.path so
`import litellm` resolves to your local checkout of SciLLM (litellm pkg under scillm dist).
Unset SCILLM_DEV_PATH to fall back to whichever distribution is installed in the venv.
"""
from __future__ import annotations
import os, sys

_dev = os.environ.get("SCILLM_DEV_PATH")
if _dev and os.path.isdir(_dev) and _dev not in sys.path:
    sys.path.insert(0, _dev)

