//! Provider abstraction for basalt
//!
//! This module defines the core Provider trait that abstracts operations
//! across different Git hosting providers (GitLab, GitHub, etc.).
//!
//! # Architecture
//!
//! The provider abstraction is the core architectural principle that enables basalt
//! to work with multiple Git hosting providers while keeping stack logic provider-agnostic.
//!
//! ## Design Principles
//!
//! - **Isolation**: All provider-specific logic is isolated in provider implementations
//! - **Agnostic core**: Core stack logic remains provider-agnostic
//! - **Direct API access**: Each provider uses REST APIs directly (no CLI dependencies)
//! - **Smart authentication**: Reuse existing tokens or prompt for PAT
//!
//! ## Direct API Pattern
//!
//! All providers call REST APIs directly using HTTP clients (reqwest).
//! This provides:
//! - Full control over API interactions
//! - Better error handling and debugging
//! - No subprocess overhead
//! - Minimal dependencies (no external CLI required)
//!
//! Authentication priority:
//! 1. Read provider CLI token if available (e.g., glab's token)
//! 2. Fallback to git credential helper
//! 3. Prompt user for Personal Access Token (PAT)
//!
//! # Usage Example
//!
//! ```rust,ignore
//! use crate::providers::{create_provider, ProviderType, CreateReviewParams};
//!
//! // Create a provider instance
//! let mut provider = create_provider(ProviderType::GitLab);
//!
//! // Authenticate
//! provider.authenticate()?;
//!
//! // Create a review
//! let params = CreateReviewParams {
//!     source_branch: "feature-branch".to_string(),
//!     target_branch: "main".to_string(),
//!     title: "Add new feature".to_string(),
//!     description: Some("This PR adds...".to_string()),
//!     draft: true,
//! };
//!
//! let review = provider.create_review(params)?;
//! println!("Created review: {}", review.url);
//! ```
//!
//! # Provider Implementations
//!
//! - [`gitlab::GitLabProvider`] - GitLab provider using REST API (in progress)
//! - [`github::GitHubProvider`] - GitHub provider using REST API (planned)
//! - [`mock::MockProvider`] - Mock provider for testing (complete)

#![allow(dead_code)] // Allow during early development

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::fmt;

pub mod github;
pub mod gitlab;
pub mod gitlab_api;
pub mod mock;

/// Supported provider types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderType {
    /// GitLab provider (uses REST API)
    GitLab,
    /// GitHub provider (uses REST API)
    GitHub,
}

impl ProviderType {
    /// Parse provider type from string
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "gitlab" => Ok(ProviderType::GitLab),
            "github" => Ok(ProviderType::GitHub),
            _ => Err(Error::UnknownProvider {
                provider: s.to_string(),
            }),
        }
    }

    /// Detect provider from git remote URL
    pub fn from_remote_url(url: &str) -> Result<Self> {
        if url.contains("gitlab.com") || url.contains("gitlab") {
            Ok(ProviderType::GitLab)
        } else if url.contains("github.com") {
            Ok(ProviderType::GitHub)
        } else {
            Err(Error::ProviderDetectionFailed {
                remote_url: url.to_string(),
            })
        }
    }

    /// Extract base URL from git remote URL
    ///
    /// Converts a git remote URL to the base URL of the hosting service.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // GitLab
    /// extract_base_url("https://gitlab.com/user/repo.git")
    ///   -> "https://gitlab.com"
    /// extract_base_url("git@gitlab.com:user/repo.git")
    ///   -> "https://gitlab.com"
    /// extract_base_url("https://gitlab.example.com/user/repo.git")
    ///   -> "https://gitlab.example.com"
    ///
    /// // GitHub
    /// extract_base_url("https://github.com/user/repo.git")
    ///   -> "https://github.com"
    /// extract_base_url("git@github.com:user/repo.git")
    ///   -> "https://github.com"
    /// ```
    pub fn extract_base_url(remote_url: &str) -> Result<String> {
        // Handle SSH URLs (git@host:path)
        if let Some(ssh_part) = remote_url.strip_prefix("git@") {
            if let Some(host) = ssh_part.split(':').next() {
                return Ok(format!("https://{}", host));
            }
        }

        // Handle HTTPS URLs (https://host/path or http://host/path)
        if remote_url.starts_with("https://") || remote_url.starts_with("http://") {
            let url = remote_url.trim_end_matches(".git").trim_end_matches('/');

            // Extract protocol and host
            if let Some(proto_end) = url.find("://") {
                let after_proto = &url[proto_end + 3..];
                if let Some(path_start) = after_proto.find('/') {
                    let host = &after_proto[..path_start];
                    let protocol = &url[..proto_end];
                    return Ok(format!("{}://{}", protocol, host));
                }
            }
        }

        Err(Error::config(format!(
            "Unable to extract base URL from remote: {}",
            remote_url
        )))
    }

    /// Extract project path from git remote URL
    ///
    /// Extracts the project path (owner/repo) from a git remote URL.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// extract_project_path("https://gitlab.com/owner/repo.git")
    ///   -> "owner/repo"
    /// extract_project_path("git@gitlab.com:owner/repo.git")
    ///   -> "owner/repo"
    /// extract_project_path("https://gitlab.example.com/group/subgroup/repo.git")
    ///   -> "group/subgroup/repo"
    /// ```
    pub fn extract_project_path(remote_url: &str) -> Result<String> {
        // Handle SSH URLs (git@host:path)
        if let Some(ssh_part) = remote_url.strip_prefix("git@") {
            if let Some(path) = ssh_part.split(':').nth(1) {
                return Ok(path.trim_end_matches(".git").to_string());
            }
        }

        // Handle HTTPS URLs
        if remote_url.starts_with("https://") || remote_url.starts_with("http://") {
            let url = remote_url.trim_end_matches(".git").trim_end_matches('/');

            if let Some(proto_end) = url.find("://") {
                let after_proto = &url[proto_end + 3..];
                if let Some(path_start) = after_proto.find('/') {
                    let path = &after_proto[path_start + 1..];
                    return Ok(path.to_string());
                }
            }
        }

        Err(Error::config(format!(
            "Unable to extract project path from remote: {}",
            remote_url
        )))
    }
}

impl fmt::Display for ProviderType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProviderType::GitLab => write!(f, "GitLab"),
            ProviderType::GitHub => write!(f, "GitHub"),
        }
    }
}

/// Represents a code review (MR, PR, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    /// Provider-specific review ID (e.g., "!123" for GitLab MR, "456" for GitHub PR)
    pub id: String,
    /// Review URL
    pub url: String,
    /// Review title
    pub title: String,
    /// Review description/body (optional)
    pub description: Option<String>,
    /// Source branch name
    pub source_branch: String,
    /// Target branch name
    pub target_branch: String,
    /// Whether the review is a draft
    pub draft: bool,
    /// Review state (open, merged, closed)
    pub state: ReviewState,
}

/// State of a code review
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReviewState {
    /// Review is open for review
    Open,
    /// Review has been merged
    Merged,
    /// Review has been closed without merging
    Closed,
}

impl fmt::Display for ReviewState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReviewState::Open => write!(f, "open"),
            ReviewState::Merged => write!(f, "merged"),
            ReviewState::Closed => write!(f, "closed"),
        }
    }
}

/// Parameters for creating a new review
#[derive(Debug, Clone)]
pub struct CreateReviewParams {
    /// Source branch name
    pub source_branch: String,
    /// Target branch name
    pub target_branch: String,
    /// Review title
    pub title: String,
    /// Review description (optional)
    pub description: Option<String>,
    /// Whether to create as draft
    pub draft: bool,
}

/// Parameters for updating an existing review
#[derive(Debug, Clone)]
pub struct UpdateReviewParams {
    /// Review ID to update
    pub review_id: String,
    /// New title (if changing)
    pub title: Option<String>,
    /// New description (if changing)
    pub description: Option<String>,
    /// New target branch (if changing)
    pub target_branch: Option<String>,
    /// Change draft status (if changing)
    pub draft: Option<bool>,
}

/// Core provider trait that all providers must implement
///
/// This trait abstracts all provider-specific operations so that
/// core stack logic remains provider-agnostic.
///
/// # Example
///
/// ```rust,ignore
/// let mut provider = create_provider(ProviderType::GitLab)?;
///
/// provider.authenticate()?;
/// let review = provider.create_review(params)?;
/// ```
pub trait Provider: Send + Sync {
    /// Get the provider type
    fn provider_type(&self) -> ProviderType;

    /// Check if the user is authenticated with the provider
    fn check_authentication(&self) -> Result<()>;

    /// Authenticate with the provider
    ///
    /// This attempts to find and verify an authentication token.
    fn authenticate(&mut self) -> Result<()>;

    /// Create a new review (MR/PR)
    fn create_review(&mut self, params: CreateReviewParams) -> Result<Review>;

    /// Update an existing review
    fn update_review(&mut self, params: UpdateReviewParams) -> Result<Review>;

    /// Get review details by ID
    fn get_review(&mut self, review_id: &str) -> Result<Review>;

    /// Check if a review exists for the given branch
    fn find_review_for_branch(&mut self, branch: &str) -> Result<Option<Review>>;
}

/// Create a provider instance for the given provider type
///
/// For GitLab, uses the default gitlab.com instance.
/// For self-hosted instances, create the provider directly.
pub fn create_provider(provider_type: ProviderType) -> Result<Box<dyn Provider>> {
    match provider_type {
        ProviderType::GitLab => Ok(Box::new(gitlab::GitLabProvider::new("https://gitlab.com")?)),
        ProviderType::GitHub => Ok(Box::new(github::GitHubProvider::new())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_type_from_str() {
        assert_eq!(
            ProviderType::from_str("gitlab").unwrap(),
            ProviderType::GitLab
        );
        assert_eq!(
            ProviderType::from_str("GitLab").unwrap(),
            ProviderType::GitLab
        );
        assert_eq!(
            ProviderType::from_str("github").unwrap(),
            ProviderType::GitHub
        );
        assert_eq!(
            ProviderType::from_str("GitHub").unwrap(),
            ProviderType::GitHub
        );
        assert!(ProviderType::from_str("unknown").is_err());
    }

    #[test]
    fn test_provider_type_from_remote_url() {
        // GitLab URLs
        assert_eq!(
            ProviderType::from_remote_url("https://gitlab.com/user/repo.git").unwrap(),
            ProviderType::GitLab
        );
        assert_eq!(
            ProviderType::from_remote_url("git@gitlab.com:user/repo.git").unwrap(),
            ProviderType::GitLab
        );
        assert_eq!(
            ProviderType::from_remote_url("https://gitlab.example.com/user/repo.git").unwrap(),
            ProviderType::GitLab
        );

        // GitHub URLs
        assert_eq!(
            ProviderType::from_remote_url("https://github.com/user/repo.git").unwrap(),
            ProviderType::GitHub
        );
        assert_eq!(
            ProviderType::from_remote_url("git@github.com:user/repo.git").unwrap(),
            ProviderType::GitHub
        );

        // Unknown
        assert!(ProviderType::from_remote_url("https://example.com/repo.git").is_err());
    }

    #[test]
    fn test_provider_type_display() {
        assert_eq!(ProviderType::GitLab.to_string(), "GitLab");
        assert_eq!(ProviderType::GitHub.to_string(), "GitHub");
    }

    #[test]
    fn test_review_state_display() {
        assert_eq!(ReviewState::Open.to_string(), "open");
        assert_eq!(ReviewState::Merged.to_string(), "merged");
        assert_eq!(ReviewState::Closed.to_string(), "closed");
    }

    #[test]
    fn test_extract_base_url() {
        // GitLab HTTPS
        assert_eq!(
            ProviderType::extract_base_url("https://gitlab.com/user/repo.git").unwrap(),
            "https://gitlab.com"
        );
        assert_eq!(
            ProviderType::extract_base_url("https://gitlab.example.com/user/repo.git").unwrap(),
            "https://gitlab.example.com"
        );

        // GitLab SSH
        assert_eq!(
            ProviderType::extract_base_url("git@gitlab.com:user/repo.git").unwrap(),
            "https://gitlab.com"
        );
        assert_eq!(
            ProviderType::extract_base_url("git@gitlab.example.com:user/repo.git").unwrap(),
            "https://gitlab.example.com"
        );

        // GitHub
        assert_eq!(
            ProviderType::extract_base_url("https://github.com/user/repo.git").unwrap(),
            "https://github.com"
        );
        assert_eq!(
            ProviderType::extract_base_url("git@github.com:user/repo.git").unwrap(),
            "https://github.com"
        );
    }

    #[test]
    fn test_extract_project_path() {
        // Simple paths
        assert_eq!(
            ProviderType::extract_project_path("https://gitlab.com/owner/repo.git").unwrap(),
            "owner/repo"
        );
        assert_eq!(
            ProviderType::extract_project_path("git@gitlab.com:owner/repo.git").unwrap(),
            "owner/repo"
        );

        // Nested paths (GitLab groups)
        assert_eq!(
            ProviderType::extract_project_path("https://gitlab.com/group/subgroup/repo.git")
                .unwrap(),
            "group/subgroup/repo"
        );

        // GitHub
        assert_eq!(
            ProviderType::extract_project_path("https://github.com/user/repo.git").unwrap(),
            "user/repo"
        );
    }
}
