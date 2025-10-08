#!/usr/bin/env bash
set -euo pipefail
# Process Copilot Web page text into actionable artifacts.
# - Extract simple TODO bullets
# - Extract any Codex-style patch blocks between *** Begin Patch / *** End Patch
#
# Usage:
#   scripts/copilot_process_review.sh -i local/copilot_review.txt \
#     -t local/copilot_todos.md -p local/copilot_extracted.patch

IN_FILE=""
TODO_OUT="local/copilot_todos.md"
PATCH_OUT="local/copilot_extracted.patch"

while getopts ":i:t:p:" opt; do
  case "$opt" in
    i) IN_FILE="$OPTARG" ;;
    t) TODO_OUT="$OPTARG" ;;
    p) PATCH_OUT="$OPTARG" ;;
    *) echo "Usage: $0 -i <input> [-t todos.md] [-p patch]" >&2; exit 2 ;;
  esac
done

[[ -f "$IN_FILE" ]] || { echo "Input not found: $IN_FILE" >&2; exit 1; }
mkdir -p "$(dirname "$TODO_OUT")" "$(dirname "$PATCH_OUT")"

# Extract TODO-like lines
awk '/^(\s*[-*•]|\s*TODO\b|\s*Fix\b|\s*Action\b)/{print}' "$IN_FILE" > "$TODO_OUT" || true

# Extract Codex-style patch blocks
awk 'BEGIN{keep=0}
  /\*\*\* Begin Patch/{keep=1}
  { if(keep){print} }
  /\*\*\* End Patch/{if(keep){keep=0; print ""}}' "$IN_FILE" > "$PATCH_OUT" || true

echo "Wrote TODOs: $TODO_OUT"
echo "Wrote extracted patch (if any): $PATCH_OUT"

