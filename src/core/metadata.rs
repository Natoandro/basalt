//! Metadata storage and retrieval
//!
//! This module handles persisting and loading basalt metadata.
//! Metadata is stored in `.git/basalt/metadata.yml` in YAML format.
//!
//! # Metadata Format
//!
//! ```yaml
//! version: "1"
//! provider: gitlab
//! base_branch: main
//!
//! branches:
//!   feature-part-1:
//!     review_id: "!123"
//!     review_url: "https://gitlab.com/..."
//!     parent: main
//!     created_at: "2024-01-01T00:00:00Z"
//!
//!   feature-part-2:
//!     review_id: "!124"
//!     review_url: "https://gitlab.com/..."
//!     parent: feature-part-1
//!     created_at: "2024-01-01T00:00:00Z"
//! ```
//!
//! # Design Principles
//!
//! - Always validate version on load
//! - Provide migration path for version changes
//! - Never trust metadata without validating against git
//! - Handle missing/corrupted metadata gracefully
//!
//! # Example
//!
//! ```rust,ignore
//! use crate::core::metadata::{Metadata, load_metadata, save_metadata};
//!
//! // Load existing metadata
//! let metadata = load_metadata()?;
//! println!("Provider: {}", metadata.provider);
//!
//! // Create new metadata
//! let mut metadata = Metadata::new(ProviderType::GitLab, "main".to_string());
//! save_metadata(&metadata)?;
//! ```

#![allow(dead_code)] // Allow during early development

use crate::core::environment;
use crate::error::{Error, Result};
use crate::providers::ProviderType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Current metadata version
pub const METADATA_VERSION: &str = "1";

/// Metadata file name
const METADATA_FILENAME: &str = "metadata.yml";

/// Top-level metadata structure
///
/// This contains all basalt metadata for a repository.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    /// Metadata format version
    pub version: String,

    /// Provider type
    pub provider: ProviderType,

    /// Base branch for the repository (e.g., "main" or "master")
    pub base_branch: String,

    /// Per-branch metadata
    #[serde(default)]
    pub branches: HashMap<String, BranchMetadata>,
}

/// Metadata for a single branch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchMetadata {
    /// Review ID from provider (e.g., "!123" for GitLab MR, "456" for GitHub PR)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_id: Option<String>,

    /// Review URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_url: Option<String>,

    /// Parent branch name
    pub parent: String,

    /// When this branch metadata was created (ISO 8601 format)
    pub created_at: String,

    /// When this branch metadata was last updated (ISO 8601 format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

impl Metadata {
    /// Create a new metadata structure
    ///
    /// # Arguments
    ///
    /// * `provider` - Provider type (GitLab, GitHub, etc.)
    /// * `base_branch` - Base branch name (e.g., "main")
    pub fn new(provider: ProviderType, base_branch: String) -> Self {
        Self {
            version: METADATA_VERSION.to_string(),
            provider,
            base_branch,
            branches: HashMap::new(),
        }
    }

    /// Validate metadata version
    ///
    /// # Errors
    ///
    /// Returns an error if the metadata version is not supported
    pub fn validate_version(&self) -> Result<()> {
        if self.version != METADATA_VERSION {
            return Err(Error::UnsupportedMetadataVersion {
                version: self.version.clone(),
                supported_version: METADATA_VERSION.to_string(),
            });
        }
        Ok(())
    }

    /// Add or update branch metadata
    ///
    /// # Arguments
    ///
    /// * `branch_name` - Name of the branch
    /// * `metadata` - Branch metadata to store
    pub fn set_branch(&mut self, branch_name: String, metadata: BranchMetadata) {
        self.branches.insert(branch_name, metadata);
    }

    /// Get branch metadata
    ///
    /// # Arguments
    ///
    /// * `branch_name` - Name of the branch
    ///
    /// # Returns
    ///
    /// The branch metadata if it exists, None otherwise
    pub fn get_branch(&self, branch_name: &str) -> Option<&BranchMetadata> {
        self.branches.get(branch_name)
    }

    /// Remove branch metadata
    ///
    /// # Arguments
    ///
    /// * `branch_name` - Name of the branch
    ///
    /// # Returns
    ///
    /// The removed metadata if it existed, None otherwise
    pub fn remove_branch(&mut self, branch_name: &str) -> Option<BranchMetadata> {
        self.branches.remove(branch_name)
    }

    /// Check if metadata exists for a branch
    ///
    /// # Arguments
    ///
    /// * `branch_name` - Name of the branch
    pub fn has_branch(&self, branch_name: &str) -> bool {
        self.branches.contains_key(branch_name)
    }
}

impl BranchMetadata {
    /// Create new branch metadata
    ///
    /// # Arguments
    ///
    /// * `parent` - Parent branch name
    pub fn new(parent: String) -> Self {
        Self {
            review_id: None,
            review_url: None,
            parent,
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: None,
        }
    }

    /// Update the review information
    ///
    /// # Arguments
    ///
    /// * `review_id` - Review ID from provider
    /// * `review_url` - Review URL
    pub fn set_review(&mut self, review_id: String, review_url: String) {
        self.review_id = Some(review_id);
        self.review_url = Some(review_url);
        self.updated_at = Some(chrono::Utc::now().to_rfc3339());
    }

    /// Mark the metadata as updated
    pub fn touch(&mut self) {
        self.updated_at = Some(chrono::Utc::now().to_rfc3339());
    }
}

/// Get the path to the metadata file
///
/// # Errors
///
/// Returns an error if not in a git repository
fn get_metadata_path() -> Result<PathBuf> {
    let basalt_dir = environment::get_basalt_dir()?;
    Ok(basalt_dir.join(METADATA_FILENAME))
}

/// Check if metadata file exists
///
/// # Errors
///
/// Returns an error if not in a git repository
pub fn metadata_exists() -> Result<bool> {
    let path = get_metadata_path()?;
    Ok(path.exists())
}

/// Load metadata from disk
///
/// # Errors
///
/// Returns an error if:
/// - Not in a git repository
/// - Metadata file doesn't exist
/// - Metadata is corrupted or invalid YAML
/// - Metadata version is not supported
pub fn load_metadata() -> Result<Metadata> {
    let path = get_metadata_path()?;

    if !path.exists() {
        return Err(Error::MetadataNotFound);
    }

    let contents = fs::read_to_string(&path).map_err(|e| {
        Error::metadata(format!(
            "Failed to read metadata file at {}: {}",
            path.display(),
            e
        ))
    })?;

    let metadata: Metadata = serde_yaml::from_str(&contents)?;

    // Validate version
    metadata.validate_version()?;

    Ok(metadata)
}

/// Save metadata to disk
///
/// Creates the basalt directory if it doesn't exist.
///
/// # Arguments
///
/// * `metadata` - Metadata to save
///
/// # Errors
///
/// Returns an error if:
/// - Not in a git repository
/// - Failed to create basalt directory
/// - Failed to serialize metadata
/// - Failed to write file
pub fn save_metadata(metadata: &Metadata) -> Result<()> {
    // Ensure basalt directory exists
    environment::create_basalt_dir()?;

    let path = get_metadata_path()?;

    // Serialize to YAML
    let yaml = serde_yaml::to_string(metadata)
        .map_err(|e| Error::metadata(format!("Failed to serialize metadata: {}", e)))?;

    // Write to file
    fs::write(&path, yaml).map_err(|e| {
        Error::metadata(format!(
            "Failed to write metadata file at {}: {}",
            path.display(),
            e
        ))
    })?;

    Ok(())
}

/// Delete the metadata file
///
/// # Errors
///
/// Returns an error if:
/// - Not in a git repository
/// - Failed to delete file (other than file not existing)
pub fn delete_metadata() -> Result<()> {
    let path = get_metadata_path()?;

    if path.exists() {
        fs::remove_file(&path).map_err(|e| {
            Error::metadata(format!(
                "Failed to delete metadata file at {}: {}",
                path.display(),
                e
            ))
        })?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_new() {
        let metadata = Metadata::new(ProviderType::GitLab, "main".to_string());
        assert_eq!(metadata.version, METADATA_VERSION);
        assert_eq!(metadata.provider, ProviderType::GitLab);
        assert_eq!(metadata.base_branch, "main");
        assert!(metadata.branches.is_empty());
    }

    #[test]
    fn test_metadata_validate_version() {
        let metadata = Metadata::new(ProviderType::GitLab, "main".to_string());
        assert!(metadata.validate_version().is_ok());

        let mut bad_metadata = metadata.clone();
        bad_metadata.version = "999".to_string();
        assert!(bad_metadata.validate_version().is_err());
    }

    #[test]
    fn test_branch_metadata() {
        let mut metadata = BranchMetadata::new("main".to_string());
        assert_eq!(metadata.parent, "main");
        assert!(metadata.review_id.is_none());
        assert!(metadata.review_url.is_none());

        metadata.set_review("!123".to_string(), "https://example.com".to_string());
        assert_eq!(metadata.review_id, Some("!123".to_string()));
        assert_eq!(metadata.review_url, Some("https://example.com".to_string()));
        assert!(metadata.updated_at.is_some());
    }

    #[test]
    fn test_metadata_branch_operations() {
        let mut metadata = Metadata::new(ProviderType::GitLab, "main".to_string());

        let branch_meta = BranchMetadata::new("main".to_string());
        metadata.set_branch("feature".to_string(), branch_meta);

        assert!(metadata.has_branch("feature"));
        assert!(!metadata.has_branch("nonexistent"));

        assert!(metadata.get_branch("feature").is_some());
        assert!(metadata.get_branch("nonexistent").is_none());

        let removed = metadata.remove_branch("feature");
        assert!(removed.is_some());
        assert!(!metadata.has_branch("feature"));
    }

    #[test]
    fn test_metadata_serialization() {
        let mut metadata = Metadata::new(ProviderType::GitLab, "main".to_string());
        let mut branch_meta = BranchMetadata::new("main".to_string());
        branch_meta.set_review("!123".to_string(), "https://example.com".to_string());
        metadata.set_branch("feature".to_string(), branch_meta);

        // Serialize to YAML
        let yaml = serde_yaml::to_string(&metadata).unwrap();
        assert!(yaml.contains("version:"));
        assert!(yaml.contains("provider:"));
        assert!(yaml.contains("base_branch:"));
        assert!(yaml.contains("branches:"));

        // Deserialize back
        let deserialized: Metadata = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(deserialized.version, metadata.version);
        assert_eq!(deserialized.provider, metadata.provider);
        assert_eq!(deserialized.base_branch, metadata.base_branch);
        assert!(deserialized.has_branch("feature"));
    }
}
