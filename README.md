# Mermaid Preview for Zed

Preview Mermaid diagrams directly in Zed editor with native Rust rendering and security-hardened validation.

## Features

✅ **Fast Rendering** - Native Rust implementation (mock for MVP, production will use mermaid-rs-renderer for 500-1000x speedup)
✅ **Secure** - Input validation blocks shell injection and DoS attacks
✅ **Smart Caching** - Content-addressed caching for instant re-renders
✅ **Simple UX** - Single slash command interface

## Installation

### From Source

1. Clone this repository:
```bash
git clone https://github.com/yourusername/zed-mermaid-plugin
cd zed-mermaid-plugin
```

2. Build the extension:
```bash
cargo build --release --target wasm32-wasip1
```

3. Install in Zed:
```bash
mkdir -p ~/.config/zed/extensions/mermaid-preview
cp target/wasm32-wasip1/release/mermaid_preview.wasm ~/.config/zed/extensions/mermaid-preview/
cp extension.toml ~/.config/zed/extensions/mermaid-preview/
```

4. Restart Zed

## Usage

### Basic Preview

Use the `/mermaid-preview` slash command in Zed's Assistant panel:

```
/mermaid-preview graph TD
    A[Start] --> B[Process]
    B --> C{Decision}
    C -->|Yes| D[End]
    C -->|No| B
```

The extension will:
1. Validate your Mermaid syntax
2. Render to SVG
3. Cache the result
4. Return a file path you can open in your system viewer

### Supported Diagram Types

- `graph` / `flowchart` - Flowcharts
- `sequenceDiagram` - Sequence diagrams
- `classDiagram` - Class diagrams
- `stateDiagram` - State diagrams
- `erDiagram` - Entity-Relationship diagrams
- `journey` - User journey diagrams
- `gantt` - Gantt charts
- `pie` - Pie charts
- `gitGraph` - Git graphs
- `mindmap` - Mind maps
- `timeline` - Timeline diagrams
- `quadrantChart` - Quadrant charts

## Security

This extension implements multiple security layers:

### Input Validation
- **Size limit**: 1MB maximum
- **Line limit**: 5000 lines maximum
- **Character whitelist**: Only safe characters allowed (no shell metacharacters like `;`, `$`, backticks, pipes)

### Cache Security
- **Content-addressed**: SHA256 hashing prevents collisions
- **Path traversal prevention**: Canonicalized paths with sandbox validation
- **Isolated storage**: Diagrams stored in `~/.cache/zed/mermaid/`

### No Shell Execution
- Pure Rust implementation
- No external process spawning
- No shell commands

## Architecture

```
┌─────────────────┐
│  Slash Command  │
│ /mermaid-preview│
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Input Validator │
│  • Size limits  │
│  • Char filter  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Cache Check    │
│  SHA256 hash    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   Renderer      │
│  (Mock for MVP) │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Cache Storage  │
│   ~/.cache/     │
└─────────────────┘
```

## Development

### Prerequisites
- Rust 1.70+
- wasm32-wasip1 target: `rustup target add wasm32-wasip1`
- Zed editor

### Building
```bash
cargo build --release --target wasm32-wasip1
```

### Testing
```bash
cargo test
```

### Running Tests with Coverage
```bash
cargo test -- --nocapture
```

## Troubleshooting

### "No Mermaid source provided"
Make sure you're passing the diagram code as arguments to the slash command.

### "Validation error: Invalid characters"
Your diagram contains characters not allowed by the security filter. Avoid using:
- Shell metacharacters: `;`, `$`, backticks, `|`, `&`
- Try to use only alphanumeric characters and safe punctuation

### "Cache write error"
Check that `~/.cache/zed/mermaid/` is writable:
```bash
mkdir -p ~/.cache/zed/mermaid
chmod 755 ~/.cache/zed/mermaid
```

### Diagram doesn't render
1. Check that your diagram starts with a valid type (`graph`, `flowchart`, etc.)
2. Verify Mermaid syntax at https://mermaid.live
3. Check file size is under 1MB and under 5000 lines

## Roadmap

### Current (MVP)
- [x] `/mermaid-preview` slash command
- [x] Secure input validation
- [x] Content-addressed caching
- [x] Mock renderer

### Phase 2 (Future)
- [ ] Replace mock with mermaid-rs-renderer (500-1000x faster)
- [ ] LSP server for real-time validation
- [ ] Tree-sitter syntax highlighting
- [ ] Auto-preview on save
- [ ] Multiple output formats (PNG, PDF)

### Phase 3 (Planned)
- [ ] AI-powered syntax error correction
- [ ] Inline rendering (when Zed API supports it)
- [ ] Collaborative editing

## Contributing

Contributions welcome! Please:
1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## License

MIT OR Apache-2.0

## Acknowledgments

- [Zed Editor](https://zed.dev/) for the excellent extension API
- [mermaid-rs-renderer](https://github.com/1jehuang/mermaid-rs-renderer) for blazing-fast native rendering
- [Mermaid](https://mermaid.js.org/) for the diagram specification

## Support

- Report bugs: https://github.com/yourusername/zed-mermaid-plugin/issues
- Questions: Open a discussion
- Security issues: Email security@example.com
