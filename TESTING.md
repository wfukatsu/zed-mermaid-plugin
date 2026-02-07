# Testing Guide for Zed Mermaid Preview Extension

## Pre-Test Setup

### ✅ Installation Checklist
- [x] Extension built for WASM target
- [x] Extension installed in Zed extensions directory
- [x] Test file created with sample diagrams
- [x] Cache directory accessible

### Extension Location
```
~/Library/Application Support/Zed/extensions/work/mermaid-preview
  -> /Users/wfukatsu/work/zed-mermaid-plugin
```

## Test Cases

### Test 1: Basic Flowchart ✓
**Input:**
```
/mermaid-preview graph TD A[Start] --> B[End]
```

**Expected Output:**
- Success message with file path
- SVG file created in `~/.cache/zed/mermaid/`
- File contains mock diagram

**Validation:**
```bash
ls -la ~/.cache/zed/mermaid/
# Should show .svg files
```

### Test 2: Input Validation ✓
**Input:**
```
/mermaid-preview graph TD; rm -rf /
```

**Expected Output:**
- Error: "Invalid characters detected"
- No file created
- Shell command blocked

### Test 3: Cache Hit ✓
**Input (run twice):**
```
/mermaid-preview graph TD A-->B
```

**Expected Output:**
- First run: "rendered successfully"
- Second run: "rendered (from cache)"
- Same file path returned

**Validation:**
```bash
# Check file timestamp doesn't change on second run
stat ~/.cache/zed/mermaid/*.svg
```

### Test 4: Large Input ✓
**Input:**
```
/mermaid-preview <paste 10000 lines>
```

**Expected Output:**
- Error: "Too many lines: 10000 (max 5000)"

### Test 5: Empty Input ✓
**Input:**
```
/mermaid-preview
```

**Expected Output:**
- Error: "No Mermaid source provided"

### Test 6: Invalid Diagram Type ✓
**Input:**
```
/mermaid-preview not a real diagram
```

**Expected Output:**
- Error: "Syntax error: Diagram must start with a valid type"

### Test 7: All Diagram Types ✓

Test each type from `test-diagram.md`:
- [ ] `graph TD` - Flowchart
- [ ] `sequenceDiagram` - Sequence
- [ ] `classDiagram` - Class
- [ ] `stateDiagram` - State
- [ ] `erDiagram` - Entity-Relationship
- [ ] `journey` - User Journey
- [ ] `gantt` - Gantt Chart
- [ ] `pie` - Pie Chart
- [ ] `gitGraph` - Git Graph

### Test 8: Security ✓

**Shell Metacharacters (should all fail):**
```
/mermaid-preview graph TD A[`whoami`]
/mermaid-preview graph TD A[$HOME]
/mermaid-preview graph TD A[test | grep]
/mermaid-preview graph TD A[test & bg]
```

**Expected:** All blocked with "Invalid characters" error

## Performance Tests

### Test 9: Render Speed
```bash
time /mermaid-preview graph TD A-->B
```

**Expected:** < 1 second for cache miss

### Test 10: Cache Performance
```bash
# First render
time /mermaid-preview graph TD A-->B

# Second render (should be faster)
time /mermaid-preview graph TD A-->B
```

**Expected:** Second render < 100ms

## Integration Tests

### Test 11: Zed Extension Loading
**Steps:**
1. Open Zed
2. Check Extensions panel
3. Look for "mermaid-preview"

**Expected:** Extension listed and active

### Test 12: Slash Command Autocomplete
**Steps:**
1. Open Assistant
2. Type `/mer`
3. Check autocomplete

**Expected:** `/mermaid-preview` appears in suggestions

## Regression Tests

### Test 13: WASM Compatibility
```bash
cargo build --target wasm32-wasip1
```

**Expected:** Builds without errors

### Test 14: Unit Tests
```bash
cargo test
```

**Expected:** All 13 tests pass

## Manual Verification

### File System
```bash
# Check cache directory
ls -la ~/.cache/zed/mermaid/

# Check file names (should be SHA256 hashes)
# Format: [64-char-hex].svg

# Verify file contents
cat ~/.cache/zed/mermaid/*.svg | head -20
```

### Extension Logs
```bash
tail -f ~/Library/Logs/Zed/Zed.log | grep mermaid
```

**Watch for:**
- Extension loading messages
- Errors or warnings
- Command execution logs

## Cleanup After Testing

```bash
# Remove test cache files
rm -rf ~/.cache/zed/mermaid/*

# Check no files left
ls -la ~/.cache/zed/mermaid/
```

## Test Results Summary

Date: 2026-02-08

| Test | Status | Notes |
|------|--------|-------|
| Basic Flowchart | ⏳ Pending | User manual test required |
| Input Validation | ✅ Pass | Unit tests verify |
| Cache Hit | ⏳ Pending | User manual test required |
| Large Input | ✅ Pass | Unit tests verify |
| Empty Input | ⏳ Pending | User manual test required |
| Invalid Type | ✅ Pass | Unit tests verify |
| All Diagram Types | ⏳ Pending | User manual test required |
| Security | ✅ Pass | Unit tests verify |
| Render Speed | ⏳ Pending | User manual test required |
| Cache Performance | ⏳ Pending | User manual test required |
| Extension Loading | ⏳ Pending | User manual test required |
| Autocomplete | ⏳ Pending | User manual test required |
| WASM Build | ✅ Pass | Verified |
| Unit Tests | ✅ Pass | All 13 passing |

## Known Issues

None identified in automated tests. Manual testing will reveal any GUI-specific issues.

## Next Steps

1. Complete manual tests in Zed
2. Report any failures
3. If all pass, mark MVP as complete
4. Consider Phase 2 features based on feedback
