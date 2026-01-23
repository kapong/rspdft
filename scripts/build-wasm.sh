#!/usr/bin/env bash
# Build rspdft WASM packages

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
WASM_CRATE="$PROJECT_ROOT/crates/wasm"

echo "=== Building rspdft WASM ==="
echo "Project root: $PROJECT_ROOT"
echo ""

# Check wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "Error: wasm-pack not found. Install with: cargo install wasm-pack"
    exit 1
fi

# Build for web (ES modules)
echo "Building for web target..."
cd "$WASM_CRATE"
wasm-pack build --target web --release
echo "  ✓ Built: $WASM_CRATE/pkg/ (web)"
echo ""

# Build for Node.js (CommonJS)
echo "Building for nodejs target..."
wasm-pack build --target nodejs --release --out-dir pkg-node
echo "  ✓ Built: $WASM_CRATE/pkg-node/ (nodejs)"
echo ""

# Setup symlinks for examples
echo "Setting up example symlinks..."

# Web example
WEB_PKG="$PROJECT_ROOT/examples/web/pkg"
if [ ! -L "$WEB_PKG" ]; then
    ln -sf "$WASM_CRATE/pkg" "$WEB_PKG"
    echo "  ✓ Linked: examples/web/pkg -> crates/wasm/pkg"
else
    echo "  - Skipped: examples/web/pkg (already linked)"
fi

# Node.js example
NODE_PKG="$PROJECT_ROOT/examples/node/pkg"
if [ ! -L "$NODE_PKG" ]; then
    ln -sf "$WASM_CRATE/pkg-node" "$NODE_PKG"
    echo "  ✓ Linked: examples/node/pkg -> crates/wasm/pkg-node"
else
    echo "  - Skipped: examples/node/pkg (already linked)"
fi

echo ""
echo "=== Build Complete ==="
echo ""
echo "Next steps:"
echo "  Browser: python3 -m http.server 8080 && open http://localhost:8080/examples/web/"
echo "  Node.js: cd examples/node && node render.mjs --help"