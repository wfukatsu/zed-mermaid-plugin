#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "Installing Mermaid Preview Extension to Zed..."

# Detect platform
if [[ "$OSTYPE" == "darwin"* ]]; then
    ZED_WORK_DIR="$HOME/Library/Application Support/Zed/extensions/work/mermaid-preview"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    ZED_WORK_DIR="$HOME/.config/zed/extensions/work/mermaid-preview"
else
    echo "Unsupported platform: $OSTYPE"
    exit 1
fi

# Copy LSP binary to extension work directory
mkdir -p "$ZED_WORK_DIR"

if [ -f "$PROJECT_DIR/target/release/mermaid-lsp" ]; then
    cp "$PROJECT_DIR/target/release/mermaid-lsp" "$ZED_WORK_DIR/mermaid-lsp"
    chmod +x "$ZED_WORK_DIR/mermaid-lsp"
    echo "  LSP binary installed to: $ZED_WORK_DIR/mermaid-lsp"
elif [ -f "$PROJECT_DIR/lsp/target/release/mermaid-lsp" ]; then
    cp "$PROJECT_DIR/lsp/target/release/mermaid-lsp" "$ZED_WORK_DIR/mermaid-lsp"
    chmod +x "$ZED_WORK_DIR/mermaid-lsp"
    echo "  LSP binary installed to: $ZED_WORK_DIR/mermaid-lsp"
else
    echo "  Warning: LSP binary not found. Run ./scripts/build.sh first."
fi

echo ""
echo "Installation complete!"
echo ""
echo "To use with Zed:"
echo "  1. Cmd+Shift+P -> 'Extensions: Install Development Extension'"
echo "  2. Select: $PROJECT_DIR"
echo "  3. Open a Markdown file with \`\`\`mermaid blocks"
echo "  4. Place cursor inside the block and press Cmd+."
