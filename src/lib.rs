use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use zed_extension_api::{
    self as zed, Result, SlashCommand, SlashCommandOutput, SlashCommandOutputSection,
};

/// Main extension struct
struct MermaidExtension {
    cache_dir: PathBuf,
}

impl MermaidExtension {
    /// Handle /mermaid-preview command
    fn handle_preview(&self, args: Vec<String>) -> Result<SlashCommandOutput> {
        if args.is_empty() {
            return Err(
                "No Mermaid source provided. Usage: /mermaid-preview <mermaid code>".to_string(),
            );
        }

        // Join all arguments to get full source
        let source = args.join(" ");

        // Validate input
        validate_mermaid_source(&source)?;
        validate_no_html_injection(&source)?;

        // Check cache
        let key = cache_key(&source);
        let cached_path = cache_path(&self.cache_dir, &key);

        if cached_path.exists() {
            return self.output_success(&cached_path, true);
        }

        // Render via CLI
        let svg = render_via_cli(&source)?;

        // Sanitize SVG (TODO: Add ammonia crate for production)
        let safe_svg = svg; // Will add sanitization in security hardening phase

        // Write to cache
        std::fs::create_dir_all(&self.cache_dir)
            .map_err(|e| format!("Failed to create cache directory: {}", e))?;
        std::fs::write(&cached_path, safe_svg)
            .map_err(|e| format!("Failed to write cache: {}", e))?;

        self.output_success(&cached_path, false)
    }

    /// Output success message with path
    fn output_success(&self, path: &Path, from_cache: bool) -> Result<SlashCommandOutput> {
        let cache_note = if from_cache {
            " (from cache)"
        } else {
            ""
        };

        let text = format!(
            "✅ Diagram rendered successfully{}\n\n\
             Preview: {}\n\n\
             Open with your system viewer (Preview.app on macOS).",
            cache_note,
            path.display()
        );

        Ok(SlashCommandOutput {
            text: text.clone(),
            sections: vec![SlashCommandOutputSection {
                range: (0..text.len()).into(),
                label: "Mermaid Preview".to_string(),
            }],
        })
    }
}

impl zed::Extension for MermaidExtension {
    fn new() -> Self {
        let cache_dir = std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(".cache")
            .join("zed")
            .join("mermaid");

        Self { cache_dir }
    }

    fn run_slash_command(
        &self,
        command: SlashCommand,
        args: Vec<String>,
        _worktree: Option<&zed::Worktree>,
    ) -> Result<SlashCommandOutput> {
        match command.name.as_str() {
            "mermaid-preview" => self.handle_preview(args),
            cmd => Err(format!("unknown slash command: \"{}\"", cmd)),
        }
    }
}

// ============================================================================
// Validation Functions
// ============================================================================

/// Validate Mermaid source code
fn validate_mermaid_source(source: &str) -> Result<()> {
    // 1. Empty check
    if source.trim().is_empty() {
        return Err("Empty diagram source".to_string());
    }

    // 2. Size limit check (1MB)
    if source.len() > 1_000_000 {
        return Err(format!(
            "Diagram too large: {} bytes (max 1MB)",
            source.len()
        ));
    }

    // 3. Line count check (5000 lines)
    let line_count = source.lines().count();
    if line_count > 5000 {
        return Err(format!(
            "Too many lines: {} (max 5000)",
            line_count
        ));
    }

    // 4. Basic syntax check - must start with diagram type
    let valid_types = [
        "graph",
        "flowchart",
        "sequenceDiagram",
        "classDiagram",
        "stateDiagram",
        "erDiagram",
        "journey",
        "gantt",
        "pie",
        "gitGraph",
        "mindmap",
        "timeline",
        "quadrantChart",
        "requirementDiagram",
        "zenuml",
        "sankey",
        "xyChart",
        "block",
    ];

    let first_word = source
        .split_whitespace()
        .next()
        .ok_or_else(|| "Invalid diagram: no content found".to_string())?;

    if !valid_types.iter().any(|t| first_word.starts_with(t)) {
        return Err(format!(
            "Unknown diagram type: '{}'. Must start with: {}",
            first_word,
            valid_types.join(", ")
        ));
    }

    Ok(())
}

/// Validate against XSS patterns
fn validate_no_html_injection(source: &str) -> Result<()> {
    let dangerous_patterns = [
        "<script",
        "javascript:",
        "onerror=",
        "onload=",
        "<iframe",
        "<embed",
        "<object",
    ];

    let lower = source.to_lowercase();
    for pattern in dangerous_patterns {
        if lower.contains(pattern) {
            return Err(format!("Blocked dangerous pattern: {}", pattern));
        }
    }

    Ok(())
}

// ============================================================================
// Cache Functions
// ============================================================================

/// Generate cache key from source
fn cache_key(source: &str) -> String {
    let mut hasher = DefaultHasher::new();
    source.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Get cache file path
fn cache_path(cache_dir: &Path, key: &str) -> PathBuf {
    cache_dir.join(format!("{}.svg", key))
}

// ============================================================================
// Renderer Function
// ============================================================================

/// Render Mermaid diagram via mmdc CLI
fn render_via_cli(source: &str) -> Result<String> {
    // Check if mmdc is installed
    let mmdc_path = which::which("mmdc").map_err(|_| {
        "Mermaid CLI (mmdc) not found.\n\n\
         Install with:\n\
         npm install -g @mermaid-js/mermaid-cli\n\n\
         Then restart Zed."
            .to_string()
    })?;

    // Create temporary files
    let temp_dir = tempfile::tempdir()
        .map_err(|e| format!("Failed to create temp directory: {}", e))?;

    let input_path = temp_dir.path().join("input.mmd");
    let output_path = temp_dir.path().join("output.svg");
    let config_path = temp_dir.path().join("config.json");

    // Write input file
    std::fs::write(&input_path, source)
        .map_err(|e| format!("Failed to write input file: {}", e))?;

    // Write config file with strict security
    let config = r#"{
  "securityLevel": "strict",
  "theme": "default"
}"#;
    std::fs::write(&config_path, config)
        .map_err(|e| format!("Failed to write config file: {}", e))?;

    // Spawn mmdc process
    let output = Command::new(&mmdc_path)
        .arg("-i")
        .arg(&input_path)
        .arg("-o")
        .arg(&output_path)
        .arg("-c")
        .arg(&config_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("Failed to execute mmdc: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Mermaid rendering failed:\n{}", stderr));
    }

    // Read SVG output
    let svg = std::fs::read_to_string(&output_path)
        .map_err(|e| format!("Failed to read rendered SVG: {}", e))?;

    Ok(svg)
}

// ============================================================================
// Extension Registration
// ============================================================================

zed::register_extension!(MermaidExtension);
