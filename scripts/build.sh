#!/bin/bash
set -e

echo "Building Mermaid Preview Extension..."

# Build LSP server (native binary)
echo "==> Building LSP server..."
cd "$(dirname "$0")/../lsp"
cargo build --release
cd ..

# Build extension (cdylib for WASM)
echo "==> Building extension..."
cargo build --release

echo ""
echo "Build complete!"
echo "  LSP binary: target/release/mermaid-lsp"
echo "  Extension:  target/release/libmermaid_preview.dylib"
