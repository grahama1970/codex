#!/usr/bin/env python3
import json
import os
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def sh(args, check=True, capture=True):
    p = subprocess.Popen(args, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    out, err = p.communicate()
    if check and p.returncode != 0:
        raise RuntimeError(f"cmd failed: {' '.join(args)}\n{err}\n{out}")
    return p.returncode, out


def categorize(paths):
    cats = {
        "prehook": [],
        "agent_bus": [],
        "tui": [],
        "core": [],
        "protocol": [],
        "exec": [],
        "cli": [],
        "workflows": [],
        "docs": [],
        "other": [],
    }
    for p in paths:
        if p.startswith("codex-rs/prehook/"):
            cats["prehook"].append(p)
        elif p.startswith("scripts/") or p.startswith("docs/feature_recipes/agent-bus/") or (
            p.startswith(".github/workflows/") and ("agent_bus" in p or "review_gate" in p or "monitors" in p)
        ):
            cats["agent_bus"].append(p)
        elif p.startswith("codex-rs/tui/"):
            cats["tui"].append(p)
        elif p.startswith("codex-rs/core/"):
            cats["core"].append(p)
        elif p.startswith("codex-rs/protocol/"):
            cats["protocol"].append(p)
        elif p.startswith("codex-rs/exec/"):
            cats["exec"].append(p)
        elif p.startswith("codex-rs/cli/"):
            cats["cli"].append(p)
        elif p.startswith(".github/workflows/"):
            cats["workflows"].append(p)
        elif p.startswith("docs/"):
            cats["docs"].append(p)
        else:
            cats["other"].append(p)
    return cats


def main():
    base = os.environ.get("UPSTREAM_REF", "upstream/main")
    target = os.environ.get("TARGET_BRANCH", "main")
    # ensure remotes present
    sh(["git", "fetch", "--all", "--prune"], check=False)
    # behind/ahead counts
    _, counts = sh(["git", "rev-list", "--left-right", "--count", f"{target}...{base}"])
    behind, ahead = [int(x) for x in counts.strip().split()]

    # file changes in upstream since target (only upstream side)
    _, files_out = sh(["git", "diff", "--name-only", f"{target}..{base}"])
    changed = [f for f in files_out.strip().splitlines() if f]
    cats = categorize(changed)

    # conflict simulation using a throwaway branch
    ts = int(time.time())
    tmp = f"tmp/upstream-merge-{target}-{ts}"
    conflict_files = []
    rc = 0
    try:
        sh(["git", "checkout", "-B", tmp, target])
        rc, _ = sh(["git", "merge", "--no-commit", "--no-ff", base], check=False)
        if rc != 0:
            # collect unmerged paths
            _, unmerged = sh(["git", "diff", "--name-only", "--diff-filter=U"])
            conflict_files = [x for x in unmerged.strip().splitlines() if x]
            sh(["git", "merge", "--abort"], check=False)
    finally:
        sh(["git", "checkout", target])
        sh(["git", "branch", "-D", tmp], check=False)

    summary = {
        "target": target,
        "base": base,
        "behind": behind,
        "ahead": ahead,
        "conflicts": conflict_files,
        "changed_count": len(changed),
        "categories": {k: len(v) for k, v in cats.items()},
    }

    md = [
        f"Upstream sync report for `{target}` vs `{base}`",
        "",
        f"- Behind: {behind} commits; Ahead: {ahead}",
        f"- Changed files: {len(changed)}",
        f"- Conflicts: {len(conflict_files)}",
        "- Areas touched:",
    ]
    for k, v in summary["categories"].items():
        md.append(f"  - {k}: {v}")
    if conflict_files:
        md.append("")
        md.append("Conflicting files:")
        for f in conflict_files[:50]:
            md.append(f"- {f}")
        if len(conflict_files) > 50:
            md.append(f"… and {len(conflict_files)-50} more")

    print("\n".join(md))
    # write JSON for artifacts
    out_dir = Path(os.environ.get("GITHUB_WORKSPACE", ROOT)) / "_upstream_report"
    out_dir.mkdir(parents=True, exist_ok=True)
    (out_dir / f"report_{target.replace('/', '_')}.json").write_text(json.dumps(summary, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

