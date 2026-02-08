# Mermaid Preview for Zed

Render Mermaid diagrams inline as SVG in Markdown files.

## Requirements

- [Zed](https://zed.dev/) 0.210.0+
- [Mermaid CLI](https://github.com/mermaid-js/mermaid-cli) (`mmdc`)

```sh
npm install -g @mermaid-js/mermaid-cli
```

## Installation

### Development

```sh
./scripts/build.sh
./scripts/install.sh
```

Then in Zed: `Cmd+Shift+P` → `Extensions: Install Development Extension` → select project directory.

### Usage

1. Open a Markdown file with a mermaid code block
2. Place cursor inside the block
3. Press `Cmd+.` to open Code Actions
4. Select **Render Mermaid Diagram**

The code block is replaced with an inline SVG image. The original source is saved to `.mermaid/` for later editing.

To restore the source: place cursor on the rendered image and select **Edit Mermaid Source**.

## Architecture

```
Zed Editor
  ↓ Code Action request
WASM Extension (src/lib.rs)
  ↓ Starts LSP binary
LSP Server (lsp/src/main.rs)
  ↓ Detects ```mermaid blocks
Renderer (lsp/src/render.rs)
  ↓ Invokes mmdc CLI
SVG output → sanitized → inserted into document
```

## Code Actions

| Action | Trigger |
|---|---|
| Render Mermaid Diagram | Cursor inside a ```` ```mermaid ```` block |
| Edit Mermaid Source | Cursor on a rendered diagram |
| Render All Mermaid Diagrams | Any Markdown with mermaid blocks |
| Edit All Mermaid Sources | Any Markdown with rendered diagrams |

## Security

SVG output is sanitized before insertion:

- `<script>` tags rejected
- Event handler attributes removed (`onclick`, `onmouseover`, etc.)
- `javascript:` protocol URLs removed
- `<foreignObject>` converted to native SVG `<text>`

## License

MIT
