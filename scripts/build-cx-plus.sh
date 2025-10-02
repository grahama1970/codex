#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
BIN_DIR="$ROOT_DIR/tools/bin"
CODEx_RS="$ROOT_DIR/codex-rs"

echo "[cx-plus] building codex CLI (release)"
cd "$CODEx_RS"
cargo build -p codex-cli --release

SRC_BIN="$CODEx_RS/target/release/codex"
mkdir -p "$BIN_DIR"
cp -f "$SRC_BIN" "$BIN_DIR/codex-fork"

# Create a small wrapper 'cx-plus' that exposes fork info
BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo unknown)
SHA=$(git rev-parse --short HEAD 2>/dev/null || echo unknown)
DATE=$(date -u +%Y-%m-%dT%H:%M:%SZ)

cat > "$BIN_DIR/cx-plus" << EOF
#!/usr/bin/env bash
set -euo pipefail
DIR=
DIR=
DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
FORK_INFO="branch:$BRANCH sha:$SHA built:$DATE"
if [[ \
  "${1-}" == "--fork-version" \
 ]]; then
  echo "cx-plus (codex fork) -> $FORK_INFO"
  exit 0
fi
exec "$DIR/codex-fork" "$@"
EOF

chmod +x "$BIN_DIR/cx-plus"

echo "[cx-plus] installed: $BIN_DIR/cx-plus"
echo "[cx-plus] fork binary: $BIN_DIR/codex-fork"
echo "[cx-plus] tip: add to PATH with: export PATH=\"$BIN_DIR:\$PATH\""

