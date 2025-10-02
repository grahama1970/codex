#!/usr/bin/env bash
set -euo pipefail

# Auto-sync target branch with upstream/main when conflict-free, then open a PR.
# Requires GH_TOKEN for opening PRs.

TARGET_BRANCH="${TARGET_BRANCH:-main}"
BASE_REF="${UPSTREAM_REF:-upstream/main}"
SYNC_BRANCH="sync/upstream-$(date -u +%Y%m%d)-${TARGET_BRANCH//\//-}"

echo "[autosync] fetching remotes"
git fetch --all --prune

echo "[autosync] creating sync branch from $TARGET_BRANCH"
git checkout -B "$SYNC_BRANCH" "$TARGET_BRANCH"

echo "[autosync] dry-run merge to detect conflicts with $BASE_REF"
if ! git merge --no-commit --no-ff "$BASE_REF"; then
  echo "[autosync] conflicts detected; aborting merge"
  git merge --abort || true
  exit 0
fi

echo "[autosync] recording merge commit"
git commit -m "chore(sync): merge $BASE_REF into $TARGET_BRANCH"

echo "[autosync] running fast deterministic tests"
python3 -m pytest -q tests || true

echo "[autosync] pushing branch $SYNC_BRANCH"
git push -u origin "$SYNC_BRANCH"

title="Sync $TARGET_BRANCH with upstream/main ($(date -u +%Y-%m-%d))"
body="Automated upstream sync.\n- Base: $BASE_REF\n- Target: $TARGET_BRANCH\n- Branch: $SYNC_BRANCH\n\nTests: minimal Python tests executed (non-blocking)."

echo "[autosync] opening PR"
gh pr create -B "$TARGET_BRANCH" -H "$SYNC_BRANCH" -t "$title" -b "$body" || true

echo "[autosync] done"

