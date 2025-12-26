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
//! - **CLI delegation**: Each provider delegates to its respective CLI tool (glab, gh, etc.)
//! - **No direct API calls**: Always use provider CLIs, never call APIs directly
//!
//! ## CLI Delegation Pattern
//!
//! All providers delegate to their respective CLI tools instead of calling provider APIs directly.
//! This provides:
//! - Simpler authentication (use existing CLI auth flows)
//! - Automatic updates (CLI tools handle API changes)
//! - Less maintenance (no API client code)
//! - User familiarity (respects existing CLI configuration)
//!
//! All CLI commands use JSON output for structured parsing:
//! ```rust,ignore
//! let output = Command::new("glab")
//!     .args(&["mr", "create", "--json"])
//!     .output()?;
//! let mr: MergeRequest = serde_json::from_slice(&output.stdout)?;
//! ```
//!
//! # Usage Example
//!
//! ```rust,ignore
//! use crate::providers::{create_provider, ProviderType, CreateReviewParams};
//!
//! // Create a provider instance
//! let provider = create_provider(ProviderType::GitLab);
//!
//! // Check prerequisites
//! provider.check_cli_available()?;
//! provider.check_authentication()?;
//!
//! // Create a review
//! let params = CreateReviewParams {
//!     source_branch: "feature-branch".to_string(),
//!     target_branch: "main".to_string(),
//!     title: "Add new feature".to_string(),
//!     description: "This PR adds...".to_string(),
//!     draft: true,
//! };
//!
//! let review = provider.create_review(params)?;
//! println!("Created review: {}", review.url);
//! ```
//!
//! # Provider Implementations
//!
//! - [`gitlab::GitLabProvider`] - GitLab provider using `glab` CLI (in progress)
//! - [`github::GitHubProvider`] - GitHub provider using `gh` CLI (in progress)
//! - [`mock::MockProvider`] - Mock provider for testing (complete)

#![allow(dead_code)] // Allow during early development

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::fmt;

pub mod github;
pub mod gitlab;
pub mod mock;

/// Supported provider types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderType {
    /// GitLab provider (uses glab CLI)
    GitLab,
    /// GitHub provider (uses gh CLI)
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
    /// Review description/body
    pub description: String,
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
    /// Review description
    pub description: String,
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
/// let provider = create_provider(ProviderType::GitLab);
///
/// // Use provider methods directly instead of provider_type().method()
/// println!("Using CLI: {}", provider.cli_name());  // "glab"
/// println!("Install from: {}", provider.install_url());
///
/// provider.check_cli_available()?;
/// provider.check_authentication()?;
/// ```
pub trait Provider: Send + Sync {
    /// Get the provider type
    fn provider_type(&self) -> ProviderType;

    /// Get the CLI command name for this provider
    fn cli_name(&self) -> &'static str {
        match self.provider_type() {
            ProviderType::GitLab => "glab",
            ProviderType::GitHub => "gh",
        }
    }

    /// Get the installation URL for the provider CLI
    fn install_url(&self) -> &'static str {
        match self.provider_type() {
            ProviderType::GitLab => "https://gitlab.com/gitlab-org/cli",
            ProviderType::GitHub => "https://cli.github.com/",
        }
    }

    /// Get the authentication command for this provider
    fn auth_command(&self) -> &'static str {
        match self.provider_type() {
            ProviderType::GitLab => "glab auth login",
            ProviderType::GitHub => "gh auth login",
        }
    }

    /// Check if the provider CLI is installed
    fn check_cli_available(&self) -> Result<()>;

    /// Check if the user is authenticated with the provider
    fn check_authentication(&self) -> Result<()>;

    /// Create a new review (MR/PR)
    fn create_review(&self, params: CreateReviewParams) -> Result<Review>;

    /// Update an existing review
    fn update_review(&self, params: UpdateReviewParams) -> Result<Review>;

    /// Get review details by ID
    fn get_review(&self, review_id: &str) -> Result<Review>;

    /// Check if a review exists for the given branch
    fn find_review_for_branch(&self, branch: &str) -> Result<Option<Review>>;
}

/// Create a provider instance for the given provider type
pub fn create_provider(provider_type: ProviderType) -> Box<dyn Provider> {
    match provider_type {
        ProviderType::GitLab => Box::new(gitlab::GitLabProvider::new()),
        ProviderType::GitHub => Box::new(github::GitHubProvider::new()),
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
    fn test_provider_cli_name() {
        let gitlab = mock::MockProvider::new_gitlab();
        let github = mock::MockProvider::new_github();
        assert_eq!(gitlab.cli_name(), "glab");
        assert_eq!(github.cli_name(), "gh");
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
}
