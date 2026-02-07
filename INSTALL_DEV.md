# Installing Mermaid Preview as a Dev Extension

## Prerequisites

- Zed editor installed
- Rust installed via `rustup` (NOT homebrew)
- wasm32-wasip1 target: `rustup target add wasm32-wasip1`

## Installation Steps

### Method 1: Using Zed's Command Palette (Recommended)

1. **Open Zed**

2. **Open Command Palette**
   - Press `Cmd+Shift+P`

3. **Run Install Dev Extension Command**
   - Type: `zed: install dev extension`
   - Press Enter

4. **Select Extension Directory**
   - Navigate to: `/Users/wfukatsu/work/zed-mermaid-plugin`
   - Click "Open"

5. **Zed will automatically:**
   - Compile your Rust code to WebAssembly Component format
   - Install the extension
   - Load it into the editor

6. **Verify Installation**
   - Open Assistant panel: `Cmd+?`
   - Type `/` to see available commands
   - You should see `/mermaid-preview` in the list

### Method 2: Using Extensions Panel

1. **Open Zed**

2. **Open Extensions Panel**
   - Menu: `Zed` → `Extensions`
   - Or press `Cmd+Shift+X`

3. **Click "Install Dev Extension"**
   - Button should be in the top right

4. **Select Extension Directory**
   - Navigate to: `/Users/wfukatsu/work/zed-mermaid-plugin`
   - Click "Open"

5. **Verify as above**

## Testing

Once installed, test the extension:

```
/mermaid-preview graph TD A[Start] --> B[End]
```

Expected output:
```
✅ Diagram rendered successfully

Preview: /Users/wfukatsu/.cache/zed/mermaid/[hash].svg

Open with your system viewer.
```

## Troubleshooting

### Extension Not Loading

Check Zed logs:
```bash
tail -f ~/Library/Logs/Zed/Zed.log | grep mermaid
```

### Rust Not Found

Ensure Rust is installed via rustup:
```bash
rustup --version
cargo --version
```

### WASM Target Missing

Install wasm32-wasip1 target:
```bash
rustup target add wasm32-wasip1
```

### Rebuild Extension

If you make changes to the code:

1. Open Command Palette: `Cmd+Shift+P`
2. Run: `zed: reload extensions`
3. Or restart Zed

## Development Workflow

1. Make changes to code
2. Reload extensions or restart Zed
3. Test changes
4. Repeat

## Notes

- Zed compiles extensions in Component Model format automatically
- No need to manually run `cargo build`
- Zed handles all WASM compilation and installation
- Dev extensions are stored separately from published extensions

## References

- [Zed Extension Development Guide](https://zed.dev/docs/extensions/developing-extensions)
- [Life of a Zed Extension](https://zed.dev/blog/zed-decoded-extensions)
