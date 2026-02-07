use thiserror::Error;

/// Rendering errors
#[derive(Debug, Error)]
pub enum RenderError {
    #[error("Syntax error: {0}")]
    SyntaxError(String),

    #[error("Rendering failed: {0}")]
    RenderFailed(String),

    #[error("Timeout: rendering took too long")]
    Timeout,

    #[error("Unsupported diagram type: {0}")]
    UnsupportedType(String),
}

/// Mermaid diagram renderer
pub struct MermaidRenderer {
    timeout_seconds: u64,
}

impl MermaidRenderer {
    /// Create a new renderer with default timeout (5 seconds)
    pub fn new() -> Self {
        Self {
            timeout_seconds: 5,
        }
    }

    /// Render Mermaid source code to SVG
    ///
    /// # Arguments
    /// * `source` - Mermaid diagram source code
    ///
    /// # Returns
    /// SVG string on success
    ///
    /// # Note
    /// This is a MOCK implementation for MVP development.
    /// In production, this will use mermaid-rs-renderer for native Rust rendering.
    pub fn render(&self, source: &str) -> Result<String, RenderError> {
        // TODO: Replace with real mermaid-rs-renderer implementation
        // For now, return mock SVG

        // Basic validation - check if it starts with a diagram type
        let diagram_types = [
            "graph", "flowchart", "sequenceDiagram", "classDiagram",
            "stateDiagram", "erDiagram", "journey", "gantt",
            "pie", "gitGraph", "mindmap", "timeline", "quadrantChart",
        ];

        let has_valid_start = diagram_types
            .iter()
            .any(|&dtype| source.trim().starts_with(dtype));

        if !has_valid_start {
            return Err(RenderError::SyntaxError(
                "Diagram must start with a valid type (graph, flowchart, etc.)".to_string(),
            ));
        }

        // Generate mock SVG
        let mock_svg = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"400\" height=\"300\">\n\
  <rect width=\"400\" height=\"300\" fill=\"#f9f9f9\"/>\n\
  <text x=\"200\" y=\"150\" text-anchor=\"middle\" font-size=\"16\" fill=\"#333\">\n\
    Mock Mermaid Diagram\n\
  </text>\n\
  <text x=\"200\" y=\"180\" text-anchor=\"middle\" font-size=\"12\" fill=\"#666\">\n\
    {} chars\n\
  </text>\n\
  <text x=\"200\" y=\"200\" text-anchor=\"middle\" font-size=\"10\" fill=\"#999\">\n\
    (Replace with mermaid-rs-renderer for production)\n\
  </text>\n\
</svg>",
            source.len()
        );

        Ok(mock_svg)
    }

    /// Validate Mermaid syntax without rendering
    pub fn validate(&self, source: &str) -> Result<(), RenderError> {
        // For MVP, just attempt to render
        self.render(source)?;
        Ok(())
    }
}

impl Default for MermaidRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_diagram() {
        let renderer = MermaidRenderer::new();
        let source = "graph TD\n    A --> B";
        assert!(renderer.render(source).is_ok());
    }

    #[test]
    fn test_invalid_diagram() {
        let renderer = MermaidRenderer::new();
        let source = "invalid diagram";
        assert!(matches!(
            renderer.render(source),
            Err(RenderError::SyntaxError(_))
        ));
    }

    #[test]
    fn test_various_diagram_types() {
        let renderer = MermaidRenderer::new();

        let diagrams = vec![
            ("graph TD\n    A --> B", true),
            ("flowchart LR\n    A --> B", true),
            ("sequenceDiagram\n    A->>B: Hello", true),
            ("pie title Pets\n    \"Dogs\" : 42", true),
            ("not a diagram", false),
        ];

        for (source, should_succeed) in diagrams {
            let result = renderer.render(source);
            assert_eq!(result.is_ok(), should_succeed, "Failed for: {}", source);
        }
    }

    #[test]
    fn test_svg_output() {
        let renderer = MermaidRenderer::new();
        let source = "graph TD\n    A --> B";
        let svg = renderer.render(source).unwrap();

        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("xmlns"));
    }
}
