# TODO - Zed Mermaid Preview Extension

## ✅ MVP Completed (2026-02-08)

The 3-week MVP is complete with:
- [x] `/mermaid-preview` slash command
- [x] Secure input validation
- [x] Content-addressed caching
- [x] Mock renderer for all 13 diagram types
- [x] 13 unit tests (all passing)
- [x] Comprehensive documentation

## Phase 2 - Post-MVP Features

### High Priority

#### 1. Replace Mock Renderer with mermaid-rs-renderer
**Status:** Deferred from MVP
**Effort:** 2-3 days
**Why:** Currently using mock renderer. Real renderer provides 500-1000x performance improvement.

**Tasks:**
- [ ] Add `mermaid-rs-renderer` dependency to Cargo.toml
- [ ] Implement `RealMermaidRenderer` struct
- [ ] Add timeout handling (5 seconds)
- [ ] Add error conversion from mermaid-rs errors
- [ ] Update tests to work with real renderer
- [ ] Benchmark performance improvements

**Reference:**
- https://github.com/1jehuang/mermaid-rs-renderer
- Dependency: `mermaid-rs-renderer = { git = "https://github.com/1jehuang/mermaid-rs-renderer", default-features = false }`

#### 2. Tree-sitter Integration
**Status:** Deferred from MVP (Task #2)
**Effort:** 3-4 days
**Why:** Syntax highlighting makes the extension much more pleasant to use.

**Tasks:**
- [ ] Research tree-sitter-mermaid grammar availability
- [ ] Add grammar to extension.toml
- [ ] Configure language detection for .mmd, .mermaid files
- [ ] Configure Markdown fenced block detection
- [ ] Test syntax highlighting in Zed
- [ ] Document supported file types

**Research needed:**
- Does tree-sitter-mermaid exist?
- Is it compatible with Zed's tree-sitter version?
- How to bundle grammar with extension?

#### 3. LSP Server for Diagnostics
**Status:** Deferred from MVP (Task #7)
**Effort:** 5-7 days
**Why:** Real-time validation improves developer experience.

**Tasks:**
- [ ] Set up tower-lsp infrastructure
- [ ] Implement `did_open` handler
- [ ] Implement `did_change` handler with debouncing
- [ ] Add syntax validation
- [ ] Publish diagnostics to editor
- [ ] Add document state management
- [ ] Test with large files
- [ ] Test with rapid typing

**Technical decisions:**
- Use tower-lsp (already planned)
- Debounce validation (300ms)
- Store document state in DashMap for thread-safety
- Validate on open and on change

### Medium Priority

#### 4. Auto-Preview on Save
**Effort:** 2 days
**Tasks:**
- [ ] Implement file watcher using notify crate
- [ ] Add 300ms debouncing
- [ ] Auto-trigger preview for .mmd, .mermaid files
- [ ] Add user setting to enable/disable
- [ ] Test with rapid saves

#### 5. Enhanced Caching
**Effort:** 2 days
**Tasks:**
- [ ] Add LRU in-memory cache (20 entries)
- [ ] Implement cache cleanup on workspace close
- [ ] Add cache statistics
- [ ] Add manual cache clear command

#### 6. PNG Export
**Effort:** 1 day
**Tasks:**
- [ ] Add resvg dependency
- [ ] Implement SVG to PNG conversion
- [ ] Add `/mermaid-export-png` command
- [ ] Add width/height parameters

### Low Priority

#### 7. Agent-Native MCP Tools
**Effort:** 3-4 days
**Tasks:**
- [ ] Implement 3 core MCP tools (preview, validate, list_cache)
- [ ] Add system prompt documentation
- [ ] Test with Claude Code
- [ ] Consider adding remaining 8 tools based on usage

#### 8. Inline Rendering (Post-Zed 1.0)
**Status:** Blocked on Zed API
**Effort:** Unknown
**Tasks:**
- [ ] Wait for Zed custom views API
- [ ] Research GPUI integration
- [ ] Implement inline preview panel
- [ ] Add pan/zoom controls

### Phase 3 - Self-Healing (Future)

#### 9. AI-Powered Error Correction
**Status:** Intentionally deferred per reviewers
**Effort:** 2 weeks
**Why:** Solve a problem that doesn't exist yet. Wait for user requests.

**Wait for:**
- 50+ user requests for this feature
- Evidence that syntax errors are a major pain point

## Known Issues

None - all tests passing, MVP works as designed.

## User Feedback Needed

Please test and provide feedback on:
- [ ] Extension loading in Zed
- [ ] Slash command usability
- [ ] Error messages clarity
- [ ] Cache performance
- [ ] Documentation completeness
- [ ] Most requested missing features

## Performance Targets

- [x] Cache hit: <10ms (implemented, needs verification)
- [x] Cache miss with mock: <500ms (implemented, needs verification)
- [ ] Cache miss with real renderer: <100ms (pending real renderer)

## Documentation Updates Needed

- [ ] Add installation video/GIF
- [ ] Add usage examples for all 13 diagram types
- [ ] Add troubleshooting for common errors
- [ ] Add contributor guide
- [ ] Add CHANGELOG.md

## Development Environment

- Rust 1.70+
- wasm32-wasip1 target
- Zed editor latest stable

## Questions for Users

1. Which diagram type do you use most?
2. Is syntax highlighting important?
3. Do you prefer manual preview or auto-preview?
4. Would you use AI-powered error correction?
5. What output formats do you need? (SVG, PNG, PDF)

---

Last updated: 2026-02-08
