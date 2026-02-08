use anyhow::{anyhow, Result};
use chrono::Local;
use log::{error, info, warn};
use lsp_server::{Connection, Message, Notification, Request, Response};
use lsp_types::*;
use serde_json::Value;
use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    fs,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
};
use url::Url;

mod render;

fn main() -> Result<()> {
    env_logger::init();
    info!("Starting Mermaid LSP server");

    let (connection, io_threads) = Connection::stdio();

    let server_capabilities = ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(
            TextDocumentSyncKind::FULL,
        )),
        code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
        execute_command_provider: Some(ExecuteCommandOptions {
            commands: vec![
                "mermaid.renderSingle".to_string(),
                "mermaid.renderAllLightweight".to_string(),
                "mermaid.editSingleSource".to_string(),
                "mermaid.editAllSources".to_string(),
            ],
            ..Default::default()
        }),
        ..Default::default()
    };

    let init_params = connection.initialize(serde_json::to_value(server_capabilities)?)?;
    let _init: InitializeParams = serde_json::from_value(init_params)?;

    info!("Mermaid LSP initialized");
    main_loop(connection)?;
    io_threads.join()?;

    Ok(())
}

/// Main message loop
fn main_loop(connection: Connection) -> Result<()> {
    let mut documents: HashMap<Url, String> = HashMap::new();

    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }
                if let Err(e) = handle_request(&connection, &req, &documents) {
                    error!("Error handling request {}: {e}", req.method);
                }
            }
            Message::Notification(not) => {
                handle_notification(&not, &mut documents);
            }
            Message::Response(_) => {}
        }
    }

    Ok(())
}

// ─── Notification handlers ──────────────────────────────────────────────────

fn handle_notification(not: &Notification, documents: &mut HashMap<Url, String>) {
    match not.method.as_str() {
        "textDocument/didOpen" => {
            if let Ok(params) = serde_json::from_value::<DidOpenTextDocumentParams>(not.params.clone()) {
                info!("Document opened: {}", params.text_document.uri);
                documents.insert(params.text_document.uri, params.text_document.text);
            }
        }
        "textDocument/didChange" => {
            if let Ok(params) = serde_json::from_value::<DidChangeTextDocumentParams>(not.params.clone()) {
                if let Some(change) = params.content_changes.first() {
                    documents.insert(params.text_document.uri, change.text.clone());
                }
            }
        }
        "textDocument/didClose" => {
            if let Ok(params) = serde_json::from_value::<DidCloseTextDocumentParams>(not.params.clone()) {
                documents.remove(&params.text_document.uri);
            }
        }
        _ => {}
    }
}

// ─── Request handlers ───────────────────────────────────────────────────────

fn handle_request(
    connection: &Connection,
    req: &Request,
    documents: &HashMap<Url, String>,
) -> Result<()> {
    match req.method.as_str() {
        "textDocument/codeAction" => handle_code_action(connection, req, documents),
        "workspace/executeCommand" => handle_execute_command(connection, req, documents),
        _ => {
            let resp = Response::new_ok(req.id.clone(), Value::Null);
            connection.sender.send(Message::Response(resp))?;
            Ok(())
        }
    }
}

// ─── Code Actions ───────────────────────────────────────────────────────────

fn handle_code_action(
    connection: &Connection,
    req: &Request,
    documents: &HashMap<Url, String>,
) -> Result<()> {
    let params: CodeActionParams = serde_json::from_value(req.params.clone())?;
    let uri = &params.text_document.uri;
    let cursor_line = params.range.start.line as usize;

    let doc = documents
        .get(uri)
        .ok_or_else(|| anyhow!("Document not found: {uri}"))?;
    let lines: Vec<&str> = doc.lines().collect();

    let mut actions: Vec<CodeActionOrCommand> = Vec::new();

    // Check if cursor is inside a ```mermaid block
    if let Some(fence) = find_mermaid_fence(&lines, cursor_line) {
        // Offer "Render Mermaid Diagram"
        if let Some(edit) = create_render_edit(uri, doc, &lines, &fence) {
            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: "Render Mermaid Diagram".to_string(),
                kind: Some(CodeActionKind::QUICKFIX),
                edit: Some(edit),
                ..Default::default()
            }));
        }
    }

    // Check if cursor is on a mermaid-source-file comment or image reference
    if let Some(edit) = find_source_edit_at_cursor(uri, doc, &lines, cursor_line) {
        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
            title: "Edit Mermaid Source".to_string(),
            kind: Some(CodeActionKind::REFACTOR),
            edit: Some(edit),
            ..Default::default()
        }));
    }

    // Always offer bulk operations if the document has mermaid content
    let has_mermaid_blocks = lines
        .iter()
        .any(|l| l.trim_start().starts_with("```mermaid"));
    let has_rendered = lines
        .iter()
        .any(|l| l.contains("<!-- mermaid-source-file:"));

    if has_mermaid_blocks {
        if let Some(edit) = create_render_all_edit(uri, doc, &lines) {
            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: "Render All Mermaid Diagrams".to_string(),
                kind: Some(CodeActionKind::SOURCE),
                edit: Some(edit),
                ..Default::default()
            }));
        }
    }

    if has_rendered {
        if let Some(edit) = create_edit_all_sources(uri, doc, &lines) {
            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: "Edit All Mermaid Sources".to_string(),
                kind: Some(CodeActionKind::SOURCE),
                edit: Some(edit),
                ..Default::default()
            }));
        }
    }

    let resp = Response::new_ok(req.id.clone(), serde_json::to_value(actions)?);
    connection.sender.send(Message::Response(resp))?;
    Ok(())
}

// ─── Execute Command ────────────────────────────────────────────────────────

fn handle_execute_command(
    connection: &Connection,
    req: &Request,
    documents: &HashMap<Url, String>,
) -> Result<()> {
    let params: ExecuteCommandParams = serde_json::from_value(req.params.clone())?;

    match params.command.as_str() {
        "mermaid.renderSingle" | "mermaid.renderAllLightweight" => {
            if let Some(uri_val) = params.arguments.first() {
                let uri: Url = serde_json::from_value(uri_val.clone())?;
                if let Some(doc) = documents.get(&uri) {
                    let lines: Vec<&str> = doc.lines().collect();
                    let edit = if params.command == "mermaid.renderAllLightweight" {
                        create_render_all_edit(&uri, doc, &lines)
                    } else {
                        // Find first mermaid block
                        find_all_mermaid_fences(&lines)
                            .first()
                            .and_then(|fence| create_render_edit(&uri, doc, &lines, fence))
                    };

                    if let Some(workspace_edit) = edit {
                        apply_edit(connection, workspace_edit)?;
                    }
                }
            }
        }
        "mermaid.editSingleSource" | "mermaid.editAllSources" => {
            if let Some(uri_val) = params.arguments.first() {
                let uri: Url = serde_json::from_value(uri_val.clone())?;
                if let Some(doc) = documents.get(&uri) {
                    let lines: Vec<&str> = doc.lines().collect();
                    let edit = if params.command == "mermaid.editAllSources" {
                        create_edit_all_sources(&uri, doc, &lines)
                    } else {
                        find_all_rendered_blocks(&lines)
                            .first()
                            .and_then(|rb| create_source_edit(&uri, doc, &lines, rb))
                    };

                    if let Some(workspace_edit) = edit {
                        apply_edit(connection, workspace_edit)?;
                    }
                }
            }
        }
        _ => {
            warn!("Unknown command: {}", params.command);
        }
    }

    let resp = Response::new_ok(req.id.clone(), Value::Null);
    connection.sender.send(Message::Response(resp))?;
    Ok(())
}

/// Send workspace/applyEdit request to the client
fn apply_edit(connection: &Connection, edit: WorkspaceEdit) -> Result<()> {
    let params = ApplyWorkspaceEditParams {
        label: Some("Mermaid".to_string()),
        edit,
    };

    let req = Request::new(
        lsp_server::RequestId::from(format!("apply-edit-{}", Local::now().timestamp_millis())),
        "workspace/applyEdit".to_string(),
        serde_json::to_value(params)?,
    );

    connection.sender.send(Message::Request(req))?;
    Ok(())
}

// ─── Mermaid block detection ────────────────────────────────────────────────

/// A detected ```mermaid ... ``` code fence
#[derive(Debug, Clone)]
struct MermaidFence {
    /// Line index of the opening ```mermaid
    start_line: usize,
    /// Line index of the closing ```
    end_line: usize,
    /// The mermaid code content (without the fences)
    code: String,
}

/// Find a mermaid fence that contains the given cursor line
fn find_mermaid_fence(lines: &[&str], cursor_line: usize) -> Option<MermaidFence> {
    find_all_mermaid_fences(lines)
        .into_iter()
        .find(|fence| cursor_line >= fence.start_line && cursor_line <= fence.end_line)
}

/// Find all ```mermaid fences in the document
fn find_all_mermaid_fences(lines: &[&str]) -> Vec<MermaidFence> {
    let mut fences = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let trimmed = lines[i].trim_start();
        if trimmed.starts_with("```mermaid") && !trimmed.starts_with("````") {
            let start = i;
            i += 1;
            // Find closing ```
            while i < lines.len() {
                let t = lines[i].trim_start();
                if t == "```" || t.starts_with("```\r") {
                    let code = lines[start + 1..i].join("\n");
                    fences.push(MermaidFence {
                        start_line: start,
                        end_line: i,
                        code,
                    });
                    break;
                }
                i += 1;
            }
        }
        i += 1;
    }

    fences
}

/// A rendered mermaid block (comment + image reference)
#[derive(Debug, Clone)]
struct RenderedBlock {
    /// Line of <!-- mermaid-source-file:... -->
    comment_line: usize,
    /// Line of the last line of this rendered block (image ref or blank line)
    end_line: usize,
    /// Path to the .mmd source file
    source_file: String,
}

/// Find all rendered mermaid blocks in the document
fn find_all_rendered_blocks(lines: &[&str]) -> Vec<RenderedBlock> {
    let mut blocks = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        if let Some(source_file) = extract_source_file_path(lines[i]) {
            let comment_line = i;
            let mut end_line = i;

            // Look ahead for blank line + image reference
            let mut j = i + 1;
            while j < lines.len() {
                let trimmed = lines[j].trim();
                if trimmed.is_empty() {
                    j += 1;
                    continue;
                }
                if trimmed.starts_with("![") && trimmed.contains("(.mermaid/") {
                    end_line = j;
                }
                break;
            }

            blocks.push(RenderedBlock {
                comment_line,
                end_line,
                source_file,
            });

            i = end_line + 1;
        } else {
            i += 1;
        }
    }

    blocks
}

/// Extract the source file path from a mermaid comment line
fn extract_source_file_path(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.starts_with("<!-- mermaid-source-file:") && trimmed.ends_with("-->") {
        let inner = trimmed
            .strip_prefix("<!-- mermaid-source-file:")?
            .strip_suffix("-->")?
            .trim();
        Some(inner.to_string())
    } else {
        None
    }
}

// ─── Rendering edits ────────────────────────────────────────────────────────

/// Compute a hash for caching purposes
fn code_hash(code: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    code.hash(&mut hasher);
    hasher.finish()
}

/// Get the document's base directory (where .mermaid/ will be created)
fn doc_base_dir(uri: &Url) -> Option<PathBuf> {
    uri.to_file_path().ok().and_then(|p| p.parent().map(|d| d.to_path_buf()))
}

/// Get a short name for the document (without extension)
fn doc_short_name(uri: &Url) -> String {
    uri.to_file_path()
        .ok()
        .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().to_string()))
        .unwrap_or_else(|| "document".to_string())
}

/// Ensure the .mermaid directory exists
fn ensure_mermaid_dir(base_dir: &Path) -> Result<PathBuf> {
    let mermaid_dir = base_dir.join(".mermaid");
    fs::create_dir_all(&mermaid_dir)?;
    Ok(mermaid_dir)
}

/// Create a workspace edit that renders a single mermaid fence to SVG
fn create_render_edit(
    uri: &Url,
    _doc: &str,
    lines: &[&str],
    fence: &MermaidFence,
) -> Option<WorkspaceEdit> {
    let base_dir = doc_base_dir(uri)?;
    let mermaid_dir = ensure_mermaid_dir(&base_dir).ok()?;
    let doc_name = doc_short_name(uri);
    let hash = code_hash(&fence.code);

    // Check cache
    let cache_dir = mermaid_dir.join(".cache");
    let _ = fs::create_dir_all(&cache_dir);
    let cache_path = cache_dir.join(format!("mermaid_{hash}.svg"));

    let svg = if cache_path.is_file() {
        info!("Using cached SVG for hash {hash}");
        fs::read_to_string(&cache_path).ok()?
    } else {
        info!("Rendering mermaid diagram...");
        match render::render_mermaid(&fence.code) {
            Ok(svg) => {
                // Save to cache
                let _ = fs::write(&cache_path, &svg);
                svg
            }
            Err(e) => {
                error!("Rendering failed: {e}");
                return None;
            }
        }
    };

    // Generate unique file names
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let svg_filename = format!("{doc_name}_diagram_{timestamp}.svg");
    let mmd_filename = format!("{doc_name}_{timestamp}.mmd");

    let svg_path = mermaid_dir.join(&svg_filename);
    let mmd_path = mermaid_dir.join(&mmd_filename);

    // Save files
    if fs::write(&svg_path, &svg).is_err() {
        error!("Failed to write SVG file");
        return None;
    }
    if fs::write(&mmd_path, &fence.code).is_err() {
        error!("Failed to write .mmd file");
        return None;
    }

    // Build the replacement text
    let relative_svg = format!(".mermaid/{svg_filename}");
    let relative_mmd = format!(".mermaid/{mmd_filename}");
    let replacement = format!(
        "<!-- mermaid-source-file:{relative_mmd} -->\n\n![Mermaid Diagram]({relative_svg})"
    );

    // Create text edit replacing the code fence
    let start_pos = Position::new(fence.start_line as u32, 0);
    let end_line = fence.end_line;
    let end_char = lines.get(end_line).map(|l| l.len()).unwrap_or(0) as u32;
    let end_pos = Position::new(end_line as u32, end_char);

    let text_edit = TextEdit::new(Range::new(start_pos, end_pos), replacement);

    let mut changes = HashMap::new();
    changes.insert(uri.clone(), vec![text_edit]);

    Some(WorkspaceEdit::new(changes))
}

/// Create a workspace edit that renders all mermaid fences
fn create_render_all_edit(
    uri: &Url,
    doc: &str,
    lines: &[&str],
) -> Option<WorkspaceEdit> {
    let fences = find_all_mermaid_fences(lines);
    if fences.is_empty() {
        return None;
    }

    let mut all_edits = Vec::new();

    // Process in reverse order so line numbers remain valid
    for fence in fences.iter().rev() {
        if let Some(edit) = create_render_edit(uri, doc, lines, fence) {
            if let Some(changes) = &edit.changes {
                if let Some(edits) = changes.get(uri) {
                    all_edits.extend(edits.clone());
                }
            }
        }
    }

    if all_edits.is_empty() {
        return None;
    }

    let mut changes = HashMap::new();
    changes.insert(uri.clone(), all_edits);
    Some(WorkspaceEdit::new(changes))
}

// ─── Source editing (restore code blocks) ───────────────────────────────────

/// Find a rendered block at the cursor position and create an edit to restore source
fn find_source_edit_at_cursor(
    uri: &Url,
    doc: &str,
    lines: &[&str],
    cursor_line: usize,
) -> Option<WorkspaceEdit> {
    find_all_rendered_blocks(lines)
        .iter()
        .find(|rb| cursor_line >= rb.comment_line && cursor_line <= rb.end_line)
        .and_then(|rb| create_source_edit(uri, doc, lines, rb))
}

/// Create a workspace edit that restores a rendered block to its mermaid source
fn create_source_edit(
    uri: &Url,
    _doc: &str,
    lines: &[&str],
    block: &RenderedBlock,
) -> Option<WorkspaceEdit> {
    let base_dir = doc_base_dir(uri)?;
    let mmd_path = base_dir.join(&block.source_file);

    // Read the original mermaid source
    let mermaid_code = fs::read_to_string(&mmd_path).ok()?;
    let replacement = format!("```mermaid\n{mermaid_code}\n```");

    let start_pos = Position::new(block.comment_line as u32, 0);
    let end_char = lines.get(block.end_line).map(|l| l.len()).unwrap_or(0) as u32;
    let end_pos = Position::new(block.end_line as u32, end_char);

    let text_edit = TextEdit::new(Range::new(start_pos, end_pos), replacement);

    let mut changes = HashMap::new();
    changes.insert(uri.clone(), vec![text_edit]);

    Some(WorkspaceEdit::new(changes))
}

/// Create a workspace edit that restores all rendered blocks to mermaid source
fn create_edit_all_sources(
    uri: &Url,
    doc: &str,
    lines: &[&str],
) -> Option<WorkspaceEdit> {
    let blocks = find_all_rendered_blocks(lines);
    if blocks.is_empty() {
        return None;
    }

    let mut all_edits = Vec::new();

    // Process in reverse order
    for block in blocks.iter().rev() {
        if let Some(edit) = create_source_edit(uri, doc, lines, block) {
            if let Some(changes) = &edit.changes {
                if let Some(edits) = changes.get(uri) {
                    all_edits.extend(edits.clone());
                }
            }
        }
    }

    if all_edits.is_empty() {
        return None;
    }

    let mut changes = HashMap::new();
    changes.insert(uri.clone(), all_edits);
    Some(WorkspaceEdit::new(changes))
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_mermaid_fences() {
        let doc = "# Hello\n\n```mermaid\ngraph TD\n  A --> B\n```\n\nSome text\n";
        let lines: Vec<&str> = doc.lines().collect();
        let fences = find_all_mermaid_fences(&lines);

        assert_eq!(fences.len(), 1);
        assert_eq!(fences[0].start_line, 2);
        assert_eq!(fences[0].end_line, 5);
        assert_eq!(fences[0].code, "graph TD\n  A --> B");
    }

    #[test]
    fn finds_multiple_fences() {
        let doc = "```mermaid\ngraph TD\n  A-->B\n```\n\n```mermaid\nsequenceDiagram\n  A->>B: Hi\n```\n";
        let lines: Vec<&str> = doc.lines().collect();
        let fences = find_all_mermaid_fences(&lines);

        assert_eq!(fences.len(), 2);
        assert_eq!(fences[0].code, "graph TD\n  A-->B");
        assert_eq!(fences[1].code, "sequenceDiagram\n  A->>B: Hi");
    }

    #[test]
    fn ignores_non_mermaid_fences() {
        let doc = "```rust\nfn main() {}\n```\n\n```mermaid\ngraph TD\n```\n";
        let lines: Vec<&str> = doc.lines().collect();
        let fences = find_all_mermaid_fences(&lines);

        assert_eq!(fences.len(), 1);
        assert!(fences[0].code.contains("graph TD"));
    }

    #[test]
    fn finds_fence_at_cursor() {
        let doc = "Text\n```mermaid\ngraph TD\n  A-->B\n```\nMore text\n";
        let lines: Vec<&str> = doc.lines().collect();

        assert!(find_mermaid_fence(&lines, 0).is_none());
        assert!(find_mermaid_fence(&lines, 1).is_some());
        assert!(find_mermaid_fence(&lines, 2).is_some());
        assert!(find_mermaid_fence(&lines, 3).is_some());
        assert!(find_mermaid_fence(&lines, 4).is_some());
        assert!(find_mermaid_fence(&lines, 5).is_none());
    }

    #[test]
    fn extracts_source_file_path() {
        assert_eq!(
            extract_source_file_path("<!-- mermaid-source-file:.mermaid/doc_20240101.mmd -->"),
            Some(".mermaid/doc_20240101.mmd".to_string())
        );
        assert_eq!(
            extract_source_file_path("Some random text"),
            None
        );
        assert_eq!(
            extract_source_file_path("<!-- other comment -->"),
            None
        );
    }

    #[test]
    fn finds_rendered_blocks() {
        let doc = "<!-- mermaid-source-file:.mermaid/doc.mmd -->\n\n![Mermaid Diagram](.mermaid/doc.svg)\n";
        let lines: Vec<&str> = doc.lines().collect();
        let blocks = find_all_rendered_blocks(&lines);

        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].comment_line, 0);
        assert_eq!(blocks[0].end_line, 2);
        assert_eq!(blocks[0].source_file, ".mermaid/doc.mmd");
    }

    #[test]
    fn code_hash_deterministic() {
        let code = "graph TD\n  A --> B";
        assert_eq!(code_hash(code), code_hash(code));
    }

    #[test]
    fn code_hash_different_for_different_code() {
        assert_ne!(code_hash("graph TD"), code_hash("graph LR"));
    }
}
