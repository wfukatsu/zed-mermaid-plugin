#!/bin/bash
set -e

echo "Setting up development environment..."

# Check Rust
if ! command -v rustc &>/dev/null; then
    echo "Error: Rust is not installed."
    echo "Install from: https://rustup.rs"
    exit 1
fi
echo "  Rust: $(rustc --version)"

# Check Node.js
if ! command -v node &>/dev/null; then
    echo "Warning: Node.js not found. Required for mermaid-cli."
    echo "Install from: https://nodejs.org"
else
    echo "  Node.js: $(node --version)"
fi

# Check npm
if ! command -v npm &>/dev/null; then
    echo "Warning: npm not found."
else
    echo "  npm: $(npm --version)"
fi

# Check/install mmdc
if command -v mmdc &>/dev/null; then
    echo "  mmdc: $(mmdc --version 2>/dev/null || echo 'installed')"
else
    echo "  mmdc not found. Installing..."
    if command -v npm &>/dev/null; then
        npm install -g @mermaid-js/mermaid-cli
        echo "  mmdc installed successfully"
    else
        echo "  Error: Cannot install mmdc without npm"
        exit 1
    fi
fi

echo ""
echo "Development environment ready!"
echo ""
echo "Build:   ./scripts/build.sh"
echo "Install: ./scripts/install.sh"
echo "Test:    cd lsp && cargo test"
