#!/usr/bin/env bash
set -euo pipefail
# Build a structured Copilot Web prompt from repo metadata and optional template,
# then optionally invoke the AppleScript sender.
#
# Usage:
#   scripts/copilot_prompt_build.sh \
#     -o local/copilot_prompt.txt \
#     [-t REVIEW_REQUEST.md] \
#     [-s "Short context header"] \
#     [--send] [--browser safari|chrome] [--url https://github.com/copilot] [--tabs 1]

out_file=""
template_file=""
short_header=""
do_send=0
browser="safari"
url="https://github.com/copilot"
tabs=1

while [[ $# -gt 0 ]]; do
  case "$1" in
    -o|--out) out_file="$2"; shift 2;;
    -t|--template) template_file="$2"; shift 2;;
    -s|--summary) short_header="$2"; shift 2;;
    --send) do_send=1; shift;;
    --browser) browser="$2"; shift 2;;
    --url) url="$2"; shift 2;;
    --tabs) tabs="$2"; shift 2;;
    -h|--help) grep '^# ' "$0" | sed 's/^# \{0,1\}//'; exit 0;;
    *) echo "Unknown arg: $1" >&2; exit 2;;
  esac
done

[[ -n "$out_file" ]] || { echo "-o/--out is required" >&2; exit 2; }

mkdir -p "$(dirname "$out_file")"

# Collect repo metadata
repo_url=$(git remote get-url --push origin 2>/dev/null || echo "(no origin)")
branch=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "(no branch)")
path_ref="${repo_url}#${branch}"

{
  echo "Fork/Repo: ${repo_url}"
  echo "Branch: ${branch}"
  echo "Path: ${path_ref}"
  echo
  if [[ -n "$short_header" ]]; then
    echo "Context: ${short_header}"
    echo
  fi
  if [[ -n "$template_file" && -f "$template_file" ]]; then
    echo "### Review Request"
    cat "$template_file"
    echo
  fi
  echo "### Relevant Files"
  # Default to recently changed files; fall back to tracked files
  if git diff --name-only --staged >/dev/null 2>&1 && [[ -n "$(git diff --name-only --staged)" ]]; then
    git diff --name-only --staged
  else
    git ls-files | head -n 200
  fi | sed 's/^/- /'
  echo
  echo "### Clarifying Questions"
  echo "- Please identify any determinism gaps between Chat and Responses paths."
  echo "- Call out brittle Chutes edge cases and propose minimal diffs."
  echo "- Provide unified diffs for any high-value fixes."
} > "$out_file"

echo "Wrote prompt: $out_file"

if [[ "$do_send" -eq 1 ]]; then
  if [[ "${OSTYPE:-}" != darwin* ]]; then
    echo "--send is supported on macOS only. Generated prompt at $out_file" >&2
    exit 0
  fi
  "$(dirname "$0")/copilot_web_send.sh" -f "$out_file" -b "$browser" -u "$url" -t "$tabs"
fi

