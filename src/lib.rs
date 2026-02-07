use std::path::PathBuf;
use zed_extension_api::{
    self as zed, Result, SlashCommand, SlashCommandOutput, SlashCommandOutputSection,
};

mod cache;
mod renderer;
mod validator;

use cache::{ContentHash, DiagramCache};
use renderer::MermaidRenderer;
use validator::InputValidator;

struct MermaidExtension {
    validator: InputValidator,
    renderer: MermaidRenderer,
    cache: DiagramCache,
}

impl MermaidExtension {
    /// Get cache directory path
    fn get_cache_dir() -> PathBuf {
        // Use system cache directory
        let cache_dir = std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(".cache")
            .join("zed")
            .join("mermaid");

        cache_dir
    }

    /// Render a Mermaid diagram
    fn render_diagram(&self, source: &str) -> Result<String> {
        // 1. Validate input
        self.validator
            .validate(source)
            .map_err(|e| format!("Validation error: {}", e))?;

        // 2. Generate content hash
        let hash = ContentHash::from_source(source);

        // 3. Check cache
        if self.cache.exists(&hash) {
            if let Ok(Some(_cached_svg)) = self.cache.get(&hash) {
                let path = self
                    .cache
                    .get_path(&hash)
                    .map_err(|e| format!("Cache path error: {}", e))?;
                return Ok(format!(
                    "✅ Diagram rendered (from cache)\n\nPreview: {}\n\nOpen with your system viewer.",
                    path.display()
                ));
            }
        }

        // 4. Render diagram
        let svg = self
            .renderer
            .render(source)
            .map_err(|e| format!("Render error: {}", e))?;

        // 5. Save to cache
        self.cache
            .put(hash.clone(), svg)
            .map_err(|e| format!("Cache write error: {}", e))?;

        // 6. Return success message
        let path = self
            .cache
            .get_path(&hash)
            .map_err(|e| format!("Cache path error: {}", e))?;

        Ok(format!(
            "✅ Diagram rendered successfully\n\nPreview: {}\n\nOpen with your system viewer.",
            path.display()
        ))
    }
}

impl zed::Extension for MermaidExtension {
    fn new() -> Self {
        let cache_dir = Self::get_cache_dir();

        Self {
            validator: InputValidator::new(),
            renderer: MermaidRenderer::new(),
            cache: DiagramCache::new(cache_dir),
        }
    }

    fn run_slash_command(
        &self,
        command: SlashCommand,
        args: Vec<String>,
        _worktree: Option<&zed::Worktree>,
    ) -> Result<SlashCommandOutput> {
        match command.name.as_str() {
            "mermaid-preview" => {
                // Get source from args
                if args.is_empty() {
                    return Err("No Mermaid source provided. Usage: /mermaid-preview <mermaid code>".to_string());
                }

                // Join all arguments to get the full source
                let source = args.join(" ");

                // Render the diagram
                let text = self.render_diagram(&source)?;

                Ok(SlashCommandOutput {
                    text: text.clone(),
                    sections: vec![SlashCommandOutputSection {
                        range: (0..text.len()).into(),
                        label: "Mermaid Preview".to_string(),
                    }],
                })
            }
            command => Err(format!("unknown slash command: \"{command}\"")),
        }
    }
}

zed::register_extension!(MermaidExtension);
