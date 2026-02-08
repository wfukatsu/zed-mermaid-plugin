use anyhow::{anyhow, Result};
use html_escape;
use once_cell::sync::Lazy;
use regex::Regex;
use std::{
    env, fs,
    path::PathBuf,
    process::{Command, Stdio},
};
use tempfile::tempdir;

// Precompiled regex patterns for security sanitization
static EVENT_HANDLER_ATTR: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)\s+on[a-z0-9_.:-]+\s*=\s*(?:"[^"]*"|'[^']*'|[^\s>]+)"#)
        .expect("event handler regex")
});

static JAVASCRIPT_HREF_ATTR: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)\s+(?:xlink:)?href\s*=\s*(?:"\s*javascript:[^"]*"|'\s*javascript:[^']*')"#)
        .expect("javascript href regex")
});

static FOREIGN_OBJECT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"<foreignObject[^>]*>(.*?)</foreignObject>"#).expect("foreignObject regex")
});

static HTML_TAG_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"<[^>]*>").expect("HTML tag regex"));

/// Render Mermaid code to SVG using mmdc CLI
pub fn render_mermaid(mermaid_code: &str) -> Result<String> {
    if mermaid_code.trim().is_empty() {
        return Err(anyhow!("Mermaid code is empty"));
    }

    let mmdc_path = find_mmdc()?;

    let temp_dir = tempdir().map_err(|e| anyhow!("Failed to create temp dir: {e}"))?;
    let input_path = temp_dir.path().join("diagram.mmd");
    let output_path = temp_dir.path().join("diagram.svg");
    let config_path = temp_dir.path().join("mermaid-config.json");

    // Write mermaid code and config to temp files
    fs::write(&input_path, mermaid_code)
        .map_err(|e| anyhow!("Failed to write temp Mermaid file: {e}"))?;
    fs::write(&config_path, include_str!("mermaid-config.json"))
        .map_err(|e| anyhow!("Failed to write temp config file: {e}"))?;

    // Execute mmdc (argument-based, no shell injection)
    let output = Command::new(&mmdc_path)
        .arg("-i")
        .arg(&input_path)
        .arg("-o")
        .arg(&output_path)
        .arg("-c")
        .arg(&config_path)
        .arg("-b")
        .arg("white")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| anyhow!("Failed to execute mmdc: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("mmdc error: {}", stderr.trim()));
    }

    let svg = fs::read_to_string(&output_path)
        .map_err(|e| anyhow!("Failed to read SVG output: {e}"))?;

    sanitize_svg(&svg)
}

/// Find mmdc binary path
fn find_mmdc() -> Result<PathBuf> {
    // Check MMDC_PATH environment variable
    if let Ok(path) = env::var("MMDC_PATH") {
        let candidate = PathBuf::from(&path);
        if candidate.is_file() {
            return Ok(candidate);
        }
        return Err(anyhow!(
            "MMDC_PATH points to '{}', but it is not a file",
            candidate.display()
        ));
    }

    // Search PATH
    if let Ok(path) = which::which("mmdc") {
        return Ok(path);
    }

    Err(anyhow!(
        "mmdc not found. Install it with: npm install -g @mermaid-js/mermaid-cli"
    ))
}

/// Sanitize SVG to prevent XSS attacks
fn sanitize_svg(svg: &str) -> Result<String> {
    // Reject SVGs containing script tags (case-insensitive)
    if svg.to_lowercase().contains("<script") {
        return Err(anyhow!("SVG contains <script> elements - blocked for security"));
    }

    let mut sanitized = svg.to_string();

    // Remove event handler attributes (onclick, onmouseover, etc.)
    sanitized = EVENT_HANDLER_ATTR
        .replace_all(&sanitized, "")
        .into_owned();

    // Remove javascript: protocol in href attributes
    sanitized = JAVASCRIPT_HREF_ATTR
        .replace_all(&sanitized, "")
        .into_owned();

    // Convert <foreignObject> to native SVG <text>
    sanitized = convert_foreign_objects(&sanitized)?;

    Ok(sanitized)
}

/// Convert <foreignObject> elements to native SVG <text> elements
fn convert_foreign_objects(svg: &str) -> Result<String> {
    let mut result = svg.to_string();

    while let Some(caps) = FOREIGN_OBJECT_REGEX.captures(&result) {
        let full_match = caps.get(0).unwrap().as_str();
        let content = caps.get(1).unwrap().as_str();
        let text = extract_text_from_html(content);

        if text.trim().is_empty() {
            result = result.replace(full_match, "");
            continue;
        }

        let fill = "#333";
        let text_element = if let Some(transform) = extract_attr(full_match, "transform") {
            format!(
                r#"<text transform="{transform}" text-anchor="start" dominant-baseline="hanging" font-family="Arial, sans-serif" font-size="14" fill="{fill}">{text}</text>"#
            )
        } else {
            let x = extract_attr(full_match, "x")
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(0.0);
            let y = extract_attr(full_match, "y")
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(0.0);
            let w = extract_attr(full_match, "width")
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(0.0);
            let h = extract_attr(full_match, "height")
                .and_then(|v| v.parse::<f64>().ok())
                .unwrap_or(0.0);

            if w <= 0.0 || h <= 0.0 {
                result = result.replace(full_match, "");
                continue;
            }

            let cx = x + w / 2.0;
            let cy = y + h / 2.0;
            format!(
                r#"<text x="{cx:.2}" y="{cy:.2}" text-anchor="middle" dominant-baseline="middle" font-family="Arial, sans-serif" font-size="14" fill="{fill}">{text}</text>"#
            )
        };

        result = result.replace(full_match, &text_element);
    }

    Ok(result)
}

/// Extract visible text from HTML content, stripping tags
fn extract_text_from_html(html: &str) -> String {
    let no_tags = HTML_TAG_REGEX.replace_all(html, "");
    let decoded = html_escape::decode_html_entities(&no_tags);
    decoded.trim().to_string()
}

/// Extract an attribute value from an HTML/XML tag
fn extract_attr(tag: &str, attr: &str) -> Option<String> {
    let pattern = format!(r#"{}="([^"]*)""#, regex::escape(attr));
    let re = Regex::new(&pattern).ok()?;
    re.captures(tag).map(|c| c[1].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_script_tags() {
        let svg = "<svg><script>alert('xss')</script></svg>";
        assert!(sanitize_svg(svg).is_err());
    }

    #[test]
    fn rejects_script_tags_case_insensitive() {
        for svg in &[
            "<svg><SCRIPT>alert('xss')</SCRIPT></svg>",
            "<svg><Script>alert('xss')</Script></svg>",
            "<svg><ScRiPt>alert('xss')</ScRiPt></svg>",
        ] {
            assert!(sanitize_svg(svg).is_err());
        }
    }

    #[test]
    fn removes_event_handlers() {
        let svg = r#"<svg><rect onclick="alert()" width="10" /></svg>"#;
        let result = sanitize_svg(svg).unwrap();
        assert!(!result.contains("onclick"));
        assert!(!result.contains("alert()"));
        assert!(result.contains("<rect"));
    }

    #[test]
    fn removes_event_handlers_single_quotes() {
        let svg = r#"<svg><rect onmouseover='doSomething()' width="10" /></svg>"#;
        let result = sanitize_svg(svg).unwrap();
        assert!(!result.contains("onmouseover"));
    }

    #[test]
    fn removes_javascript_hrefs() {
        let svg = r#"<svg><a href="javascript:alert('xss')">link</a></svg>"#;
        let result = sanitize_svg(svg).unwrap();
        assert!(!result.contains("javascript:"));
    }

    #[test]
    fn removes_xlink_javascript_hrefs() {
        let svg = r#"<svg><a xlink:href='javascript:malicious()'>link</a></svg>"#;
        let result = sanitize_svg(svg).unwrap();
        assert!(!result.contains("javascript:"));
    }

    #[test]
    fn converts_foreign_objects() {
        let svg = r#"<svg width="100" height="50"><foreignObject x="10" y="10" width="80" height="30"><div>Hello</div></foreignObject></svg>"#;
        let result = sanitize_svg(svg).unwrap();
        assert!(!result.contains("foreignObject"));
        assert!(result.contains("<text"));
        assert!(result.contains("Hello"));
    }

    #[test]
    fn skips_empty_foreign_objects() {
        let svg = r#"<svg><foreignObject x="0" y="0" width="0" height="0"><div></div></foreignObject></svg>"#;
        let result = sanitize_svg(svg).unwrap();
        assert!(!result.contains("foreignObject"));
        assert!(!result.contains("<text"));
    }

    #[test]
    fn centers_text_in_foreign_object() {
        let svg = r#"<svg><foreignObject x="20" y="30" width="160" height="40"><p>Label</p></foreignObject></svg>"#;
        let result = sanitize_svg(svg).unwrap();
        assert!(result.contains(r#"x="100.00""#));
        assert!(result.contains(r#"y="50.00""#));
        assert!(result.contains("Label"));
    }

    #[test]
    fn strips_html_tags_from_foreign_object() {
        let svg = r#"<svg><foreignObject x="10" y="10" width="80" height="30"><div><p>Label</p></div></foreignObject></svg>"#;
        let result = sanitize_svg(svg).unwrap();
        assert!(result.contains("Label"));
        assert!(!result.contains("<p>"));
        assert!(!result.contains("<div>"));
    }
}
