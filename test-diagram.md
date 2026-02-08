# Test Mermaid Diagram

Here's a simple flowchart to test:

```
graph TD
    A[Start] --> B{Decision?}
    B -->|Yes| C[Action 1]
    B -->|No| D[Action 2]
    C --> E[End]
    D --> E
```

## How to Test

1. Open Zed Assistant (Cmd+Shift+A or Ctrl+Shift+A)
2. Type: `/mermaid-preview graph TD A[Start] --> B[End]`
3. The extension will render the diagram and show you the SVG file path
4. Open the SVG file in Preview.app (macOS) or your system viewer

## Expected Output

You should see a message like:
```
✅ Diagram rendered successfully

Preview: /Users/wfukatsu/.cache/zed/mermaid/[hash].svg

Open with your system viewer (Preview.app on macOS).
```
