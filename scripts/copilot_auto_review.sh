#!/usr/bin/env bash
set -euo pipefail
# Open or update a PR and wait for Copilot's review, saving output locally.

REPO=${REPO:-$(gh repo view --json nameWithOwner -q .nameWithOwner)}
BASE=${BASE:-$(gh repo view --json defaultBranchRef -q .defaultBranchRef.name)}
HEAD=${HEAD:-$(git rev-parse --abbrev-ref HEAD)}
BODY_FILE=${BODY_FILE:-REVIEW_REQUEST.md}
OUT_DIR=${OUT_DIR:-local}
OUT_FILE=$OUT_DIR/copilot_pr_review.txt

mkdir -p "$OUT_DIR"

if ! gh pr view -R "$REPO" --json number -q .number >/dev/null 2>&1; then
  gh pr create -R "$REPO" -B "$BASE" -H "$HEAD" -t "Comprehensive review request" -F "$BODY_FILE"
else
  gh pr edit -R "$REPO" -F "$BODY_FILE"
fi

PR=$(gh pr view -R "$REPO" --json number -q .number)
echo "PR #$PR opened/updated for $HEAD -> $BASE"

echo "Waiting for Copilot review comments (repo ruleset must enable auto reviews)…"
for i in $(seq 1 60); do
  authors=$(gh pr view "$PR" -R "$REPO" --json comments -q '.comments[].author.login' | tr -d '\r' || true)
  echo "[$(date +%H:%M:%S)] commenters: ${authors:-none}"
  if echo "$authors" | rg -qi 'copilot'; then
    break
  fi
  sleep 10
done

{
  echo "# PR summary";
  gh pr view "$PR" -R "$REPO" --json url,title,author,headRefName,baseRefName,isDraft,createdAt -q \
    '{url: .url, title: .title, head: .headRefName, base: .baseRefName, draft: .isDraft, createdAt: .createdAt, author: .author.login}';
  echo "\n# Comments";
  gh pr view "$PR" -R "$REPO" --comments;
} > "$OUT_FILE"

echo "Saved Copilot review to $OUT_FILE"

