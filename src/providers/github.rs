//! GitHub provider implementation
//!
//! This provider will use the GitHub REST API directly to interact with GitHub.
//! Authentication is handled via Personal Access Tokens (PAT), with automatic
//! detection from gh CLI config or git credential helper.
//!
//! NOTE: This is a stub implementation. GitHub support is planned for post-MVP.

#![allow(dead_code)] // Allow during early development

use crate::error::{Error, Result};
use crate::providers::{CreateReviewParams, Provider, ProviderType, Review, UpdateReviewParams};

/// GitHub provider using REST API (stub)
pub struct GitHubProvider {}

impl GitHubProvider {
    /// Create a new GitHub provider
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for GitHubProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for GitHubProvider {
    fn provider_type(&self) -> ProviderType {
        ProviderType::GitHub
    }

    fn check_authentication(&self) -> Result<()> {
        // TODO: Implement authentication check
        Ok(())
    }

    fn authenticate(&mut self) -> Result<()> {
        // TODO: Implement authentication
        // Similar to GitLab: read gh token, git credential, or prompt for PAT
        Err(Error::provider_op(
            "GitHub authentication not yet implemented",
        ))
    }

    fn create_review(&mut self, _params: CreateReviewParams) -> Result<Review> {
        // TODO: Implement PR creation via GitHub REST API
        Err(Error::provider_op("GitHub PR creation not yet implemented"))
    }

    fn update_review(&mut self, _params: UpdateReviewParams) -> Result<Review> {
        // TODO: Implement PR update via GitHub REST API
        Err(Error::provider_op("GitHub PR update not yet implemented"))
    }

    fn get_review(&mut self, _review_id: &str) -> Result<Review> {
        // TODO: Implement PR retrieval via GitHub REST API
        Err(Error::provider_op(
            "GitHub PR retrieval not yet implemented",
        ))
    }

    fn find_review_for_branch(&mut self, _branch: &str) -> Result<Option<Review>> {
        // TODO: Implement finding PR by branch via GitHub REST API
        Err(Error::provider_op(
            "GitHub PR lookup by branch not yet implemented",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_type() {
        let provider = GitHubProvider::new();
        assert_eq!(provider.provider_type(), ProviderType::GitHub);
    }

    // Note: GitHub provider is a stub for now
    // Real implementation will be done post-MVP
}
