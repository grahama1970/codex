import json
import os
import re
import time
from typing import Any, Dict, List, Optional, Tuple

try:
    import requests  # type: ignore
except Exception:  # pragma: no cover
    requests = None
    import urllib.request
    import urllib.parse


CATALOG_URL_DEFAULT = "https://api.chutes.ai/chutes/"

_UNITS = {"M": 1_000_000, "B": 1_000_000_000, "T": 1_000_000_000_000}
_ACTIVATED_RE = re.compile(r"(\d+(?:\.\d+)?)\s*([MBT])\s*(?:activated)", re.IGNORECASE)
_TOTAL_PARAM_RE = re.compile(r"(\d+(?:\.\d+)?)\s*([MBT])\s*(?:param|params|parameter|parameters)\b", re.IGNORECASE)


def _param_value(num: str, unit: str) -> Optional[int]:
    try:
        return int(float(num) * _UNITS[unit.upper()])
    except Exception:
        return None


def _safe_get(d: Dict[str, Any], path: List[str], default=None):
    cur = d
    for k in path:
        if not isinstance(cur, dict):
            return default
        cur = cur.get(k)
    return default if cur is None else cur


def _parse_params_from_tagline(tagline: str) -> Dict[str, Any]:
    if not tagline:
        return {}
    out: Dict[str, Any] = {}
    act = _ACTIVATED_RE.search(tagline)
    if act:
        n, u = act.groups()
        out["activated"] = {"str": f"{n}{u.upper()}", "value": _param_value(n, u)}
    tot = _TOTAL_PARAM_RE.search(tagline)
    if tot:
        n, u = tot.groups()
        out["total"] = {"str": f"{n}{u.upper()}", "value": _param_value(n, u)}
    if out:
        out["effective"] = out.get("activated") or out.get("total")
        out["source"] = "tagline"
    return out


def _parse_params_from_readme(readme: str) -> Dict[str, Any]:
    if not readme:
        return {}
    head = readme[:4000]
    out: Dict[str, Any] = {}
    act = _ACTIVATED_RE.search(head)
    tot = _TOTAL_PARAM_RE.search(head)
    if act:
        n, u = act.groups()
        out["activated"] = {"str": f"{n}{u.upper()}", "value": _param_value(n, u)}
    if tot:
        n, u = tot.groups()
        out["total"] = {"str": f"{n}{u.upper()}", "value": _param_value(n, u)}
    if out:
        out["effective"] = out.get("activated") or out.get("total")
        out["source"] = "readme"
    return out


def effective_params_block(m: Dict[str, Any]) -> Dict[str, Any]:
    return _parse_params_from_tagline(m.get("tagline") or "") or _parse_params_from_readme(
        m.get("readme") or ""
    )


def fetch_chutes_catalog(api_key: str, base_url: Optional[str] = None, timeout: int = 20) -> List[Dict[str, Any]]:
    url = (base_url or CATALOG_URL_DEFAULT).rstrip("/") + "/"
    headers = {"Authorization": f"Bearer {api_key}", "Accept": "application/json"}
    params = {"include_public": "true", "include_schemas": "false", "limit": "10000"}
    if requests is not None:
        with requests.Session() as s:  # type: ignore
            s.headers.update(headers)
            r = s.get(url, params=params, timeout=timeout)
            r.raise_for_status()
            payload = r.json()
    else:  # pragma: no cover
        qs = urllib.parse.urlencode(params)
        req = urllib.request.Request(url + "?" + qs, headers=headers)
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            payload = json.loads(resp.read().decode("utf-8"))
    return payload.get("items", [])


def pick_best_cheapest_multimodal(chutes: List[Dict[str, Any]], min_params: int = 70_000_000_000) -> Optional[Tuple[str, Dict[str, Any]]]:
    """
    Heuristic: among models with multi-modal support (>=2 modalities, must include text)
    and effective params >= min_params, pick the lowest output USD/1M tokens,
    tie-break by higher effective params, then larger context window.
    Returns (catalog_id, model_info) or None.
    """
    candidates = []
    for m in chutes:
        modalities = m.get("modalities") or []
        if not isinstance(modalities, list) or "text" not in modalities or len(modalities) < 2:
            continue
        params_blk = effective_params_block(m)
        eff_val = _safe_get(params_blk, ["effective", "value"]) or 0
        if eff_val < min_params:
            continue
        out_ppm = _safe_get(m, ["current_estimated_price", "per_million_tokens", "output", "usd"])
        in_ppm = _safe_get(m, ["current_estimated_price", "per_million_tokens", "input", "usd"])
        try:
            out_ppm_f = float(out_ppm)
        except Exception:
            out_ppm_f = float("inf")
        try:
            in_ppm_f = float(in_ppm)
        except Exception:
            in_ppm_f = float("inf")
        ctx = m.get("max_input_tokens") or m.get("context_length") or _safe_get(m, ["limits", "max_input_tokens"]) or 0
        catalog_id = m.get("name")
        if not catalog_id:
            continue
        candidates.append((catalog_id, m, out_ppm_f, in_ppm_f, eff_val, int(ctx)))

    if not candidates:
        return None

    candidates.sort(key=lambda x: (x[2], -x[4], -x[5]))
    top = candidates[0]
    return top[0], top[1]

