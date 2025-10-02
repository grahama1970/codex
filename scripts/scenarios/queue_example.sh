#!/usr/bin/env bash
set -euo pipefail

# Create an agent-task issue that routes /notify with a small JSON payload.
# Requires gh CLI authenticated and repo write.

TITLE="agent-task: /notify (demo)"
TS=$(date +%s)
PAYLOAD=$(jq -c -n --arg ts "$TS" '{event:"demo",ts:($ts|tonumber),note:"hello from queue_example.sh"}')

BODY=$(printf "/notify\n%s\n" "$PAYLOAD")

gh issue create -t "$TITLE" -b "$BODY" -l agent-task
echo "Created agent-task issue; queue will process on next run."

