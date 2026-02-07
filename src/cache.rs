use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Cache-related errors
#[derive(Debug, Error)]
pub enum CacheError {
    #[error("Failed to read from cache: {0}")]
    ReadError(String),

    #[error("Failed to write to cache: {0}")]
    WriteError(String),

    #[error("Invalid cache path: {0}")]
    InvalidPath(String),
}

/// Content hash for cache keys
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ContentHash(String);

impl ContentHash {
    /// Generate content hash from source code
    pub fn from_source(source: &str) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(source.as_bytes());
        // Include extension version in hash to invalidate cache on updates
        hasher.update(env!("CARGO_PKG_VERSION").as_bytes());

        ContentHash(format!("{:x}", hasher.finalize()))
    }

    /// Get the hash as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Validate that the hash is safe for use in filenames
    pub fn validate(&self) -> Result<(), CacheError> {
        // Must be exactly 64 hex characters (SHA256)
        if self.0.len() != 64 {
            return Err(CacheError::InvalidPath(
                "Hash must be 64 characters".to_string(),
            ));
        }

        // Must only contain hex characters
        if !self.0.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(CacheError::InvalidPath(
                "Hash must only contain hex characters".to_string(),
            ));
        }

        Ok(())
    }
}

/// Simple file-based cache for rendered diagrams
pub struct DiagramCache {
    cache_dir: PathBuf,
}

impl DiagramCache {
    /// Create a new cache with the given directory
    pub fn new(cache_dir: PathBuf) -> Self {
        Self { cache_dir }
    }

    /// Get cache directory path
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Generate path for a cache entry
    pub fn get_path(&self, hash: &ContentHash) -> Result<PathBuf, CacheError> {
        // Validate hash to prevent path traversal
        hash.validate()?;

        // Build safe path
        let filename = format!("{}.svg", hash.as_str());
        let path = self.cache_dir.join(&filename);

        // Verify path is within cache directory (prevent path traversal)
        let canonical_cache = self.cache_dir.canonicalize().map_err(|e| {
            CacheError::InvalidPath(format!("Cannot canonicalize cache dir: {}", e))
        })?;

        let canonical_path = path.canonicalize().or_else(|_| {
            // If path doesn't exist yet, canonicalize parent and append filename
            self.cache_dir
                .canonicalize()
                .map(|p| p.join(&filename))
        }).map_err(|e| CacheError::InvalidPath(format!("Cannot canonicalize path: {}", e)))?;

        if !canonical_path.starts_with(&canonical_cache) {
            return Err(CacheError::InvalidPath(
                "Path traversal attempt detected".to_string(),
            ));
        }

        Ok(path)
    }

    /// Check if a cached entry exists
    pub fn exists(&self, hash: &ContentHash) -> bool {
        self.get_path(hash).ok().map_or(false, |path| path.exists())
    }

    /// Read cached content (placeholder for WASM-compatible implementation)
    pub fn get(&self, hash: &ContentHash) -> Result<Option<String>, CacheError> {
        let path = self.get_path(hash)?;

        if !path.exists() {
            return Ok(None);
        }

        // TODO: In real implementation, use zed::fs::read()
        // For now, use std::fs as placeholder until we integrate with Zed API
        std::fs::read_to_string(&path)
            .map(Some)
            .map_err(|e| CacheError::ReadError(e.to_string()))
    }

    /// Write content to cache (placeholder for WASM-compatible implementation)
    pub fn put(&self, hash: ContentHash, content: String) -> Result<(), CacheError> {
        let path = self.get_path(&hash)?;

        // Ensure cache directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| CacheError::WriteError(format!("Cannot create cache dir: {}", e)))?;
        }

        // TODO: In real implementation, use zed::fs::write()
        // For now, use std::fs as placeholder until we integrate with Zed API
        std::fs::write(&path, content.as_bytes())
            .map_err(|e| CacheError::WriteError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_content_hash_generation() {
        let source1 = "graph TD\n    A --> B";
        let source2 = "graph TD\n    A --> B";
        let source3 = "graph TD\n    C --> D";

        let hash1 = ContentHash::from_source(source1);
        let hash2 = ContentHash::from_source(source2);
        let hash3 = ContentHash::from_source(source3);

        // Same content should produce same hash
        assert_eq!(hash1, hash2);

        // Different content should produce different hash
        assert_ne!(hash1, hash3);

        // Hash should be 64 hex characters
        assert_eq!(hash1.as_str().len(), 64);
        assert!(hash1.as_str().chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_hash_validation() {
        let valid = ContentHash("a".repeat(64));
        assert!(valid.validate().is_ok());

        let invalid_length = ContentHash("abc".to_string());
        assert!(invalid_length.validate().is_err());

        let invalid_chars = ContentHash("g".repeat(64));
        assert!(invalid_chars.validate().is_err());
    }

    #[test]
    fn test_path_traversal_prevention() {
        let temp_dir = TempDir::new().unwrap();
        let cache = DiagramCache::new(temp_dir.path().to_path_buf());

        // Attempt path traversal
        let malicious_hash = ContentHash("../../../etc/passwd".to_string());
        assert!(cache.get_path(&malicious_hash).is_err());
    }

    #[test]
    fn test_cache_operations() {
        let temp_dir = TempDir::new().unwrap();
        let cache = DiagramCache::new(temp_dir.path().to_path_buf());

        let source = "graph TD\n    A --> B";
        let hash = ContentHash::from_source(source);
        let content = "<svg>test</svg>".to_string();

        // Initially not in cache
        assert!(!cache.exists(&hash));
        assert!(cache.get(&hash).unwrap().is_none());

        // Write to cache
        cache.put(hash.clone(), content.clone()).unwrap();

        // Now should exist
        assert!(cache.exists(&hash));
        assert_eq!(cache.get(&hash).unwrap(), Some(content));
    }
}
