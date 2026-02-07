use regex::Regex;
use std::sync::OnceLock;
use thiserror::Error;

/// Maximum allowed input size (1MB)
const MAX_SIZE_BYTES: usize = 1_048_576;

/// Maximum allowed line count (DoS protection)
const MAX_LINES: usize = 5000;

/// Input validation errors
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Input too large: {size} bytes (max {max} bytes)")]
    TooLarge { size: usize, max: usize },

    #[error("Too many lines: {lines} lines (max {max} lines)")]
    TooManyLines { lines: usize, max: usize },

    #[error("Invalid characters detected: {hint}")]
    InvalidCharacters { hint: String },

    #[error("Empty input")]
    EmptyInput,
}

/// Input validator with security constraints
pub struct InputValidator {
    allowed_chars: &'static Regex,
    max_size_bytes: usize,
    max_lines: usize,
}

/// Lazily initialized regex pattern for allowed characters
static ALLOWED_CHARS_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_allowed_chars_regex() -> &'static Regex {
    ALLOWED_CHARS_REGEX.get_or_init(|| {
        // Whitelist: alphanumeric, whitespace, and safe punctuation
        // Excludes shell metacharacters like ; $ ` | & > < etc.
        Regex::new(r"^[a-zA-Z0-9\s\-_\[\]\(\)\{\}:,\.\n\r\t]+$")
            .expect("Valid regex pattern")
    })
}

impl InputValidator {
    /// Create a new validator with default limits
    pub fn new() -> Self {
        Self {
            allowed_chars: get_allowed_chars_regex(),
            max_size_bytes: MAX_SIZE_BYTES,
            max_lines: MAX_LINES,
        }
    }

    /// Validate input according to security constraints
    pub fn validate(&self, source: &str) -> Result<(), ValidationError> {
        // Check for empty input
        if source.trim().is_empty() {
            return Err(ValidationError::EmptyInput);
        }

        // Size check (prevent memory exhaustion)
        if source.len() > self.max_size_bytes {
            return Err(ValidationError::TooLarge {
                size: source.len(),
                max: self.max_size_bytes,
            });
        }

        // Line count check (DoS protection)
        let line_count = source.lines().count();
        if line_count > self.max_lines {
            return Err(ValidationError::TooManyLines {
                lines: line_count,
                max: self.max_lines,
            });
        }

        // Character whitelist check (no shell metacharacters)
        if !self.allowed_chars.is_match(source) {
            return Err(ValidationError::InvalidCharacters {
                hint: "Only alphanumeric, whitespace, and basic punctuation allowed (no shell metacharacters)".to_string(),
            });
        }

        Ok(())
    }
}

impl Default for InputValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_mermaid() {
        let validator = InputValidator::new();
        let source = "graph TD\n    A[Start] --> B[End]";
        assert!(validator.validate(source).is_ok());
    }

    #[test]
    fn test_empty_input() {
        let validator = InputValidator::new();
        let source = "";
        assert!(matches!(
            validator.validate(source),
            Err(ValidationError::EmptyInput)
        ));
    }

    #[test]
    fn test_too_large() {
        let validator = InputValidator::new();
        let source = "A".repeat(MAX_SIZE_BYTES + 1);
        assert!(matches!(
            validator.validate(&source),
            Err(ValidationError::TooLarge { .. })
        ));
    }

    #[test]
    fn test_too_many_lines() {
        let validator = InputValidator::new();
        let source = "line\n".repeat(MAX_LINES + 1);
        assert!(matches!(
            validator.validate(&source),
            Err(ValidationError::TooManyLines { .. })
        ));
    }

    #[test]
    fn test_shell_metacharacters_blocked() {
        let validator = InputValidator::new();

        // Test various shell metacharacters
        let dangerous_inputs = vec![
            "graph TD; rm -rf /",
            "graph TD\n    A[`whoami`]",
            "graph TD\n    A[$SHELL]",
            "graph TD\n    A[test | grep]",
            "graph TD\n    A[test & bg]",
            "graph TD\n    A[test > file]",
        ];

        for input in dangerous_inputs {
            assert!(
                matches!(
                    validator.validate(input),
                    Err(ValidationError::InvalidCharacters { .. })
                ),
                "Should block: {}",
                input
            );
        }
    }
}
