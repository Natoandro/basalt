//! GitLab provider implementation
//!
//! This provider uses the GitLab REST API directly to interact with GitLab.
//! Authentication is handled via Personal Access Tokens (PAT).
//!
//! # Authentication Priority
//!
//! 1. Use stored token from metadata (`.git/basalt/metadata.yml`)
//! 2. If no stored token or authentication fails:
//!    - Try reading from glab CLI config
//!    - Try git credential helper
//!    - Offer CLI auth (if glab is available) or manual PAT entry
//! 3. Store successful token in metadata for future use

use crate::error::{Error, Result};
use crate::providers::gitlab_api::GitLabClient;
use crate::providers::{
    CreateReviewParams, Provider, ProviderType, Review, ReviewState, UpdateReviewParams,
};

/// GitLab provider using REST API
pub struct GitLabProvider {
    /// GitLab API client
    client: GitLabClient,
    /// Project path (e.g., "owner/repo")
    project_path: Option<String>,
    /// Whether we've successfully authenticated
    authenticated: bool,
}

impl GitLabProvider {
    /// Create a new GitLab provider
    ///
    /// # Arguments
    ///
    /// * `base_url` - Base URL of the GitLab instance (e.g., "https://gitlab.com")
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let provider = GitLabProvider::new("https://gitlab.com")?;
    /// ```
    pub fn new(base_url: &str) -> Result<Self> {
        let client = GitLabClient::new(base_url)
            .map_err(|e| Error::provider_op(format!("Failed to create GitLab client: {}", e)))?;

        Ok(Self {
            client,
            project_path: None,
            authenticated: false,
        })
    }

    /// Set the project path for this provider
    ///
    /// The project path is extracted from the git remote URL.
    /// For example, "https://gitlab.com/owner/repo.git" -> "owner/repo"
    pub fn set_project_path(&mut self, project_path: String) {
        self.project_path = Some(project_path);
    }

    /// Get the project path, returning an error if not set
    fn get_project_path(&self) -> Result<&str> {
        self.project_path
            .as_deref()
            .ok_or_else(|| Error::provider_op("Project path not set"))
    }

    /// Convert GitLab MR state to ReviewState
    fn parse_review_state(state: &str) -> ReviewState {
        match state {
            "opened" => ReviewState::Open,
            "closed" => ReviewState::Closed,
            "merged" => ReviewState::Merged,
            _ => ReviewState::Open, // Default to open for unknown states
        }
    }

    /// Convert GitLab MR to Review
    fn mr_to_review(mr: crate::providers::gitlab_api::MergeRequest) -> Review {
        Review {
            id: mr.iid.to_string(),
            title: mr.title,
            description: mr.description,
            state: Self::parse_review_state(&mr.state),
            url: mr.web_url,
            source_branch: mr.source_branch,
            target_branch: mr.target_branch,
            draft: mr.draft,
        }
    }
}

impl Default for GitLabProvider {
    fn default() -> Self {
        // Default to gitlab.com
        Self::new("https://gitlab.com").expect("Failed to create default GitLab provider")
    }
}

impl GitLabProvider {
    /// Get the authentication token for storage in metadata
    ///
    /// This should be called after successful authentication to persist the token.
    pub fn get_auth_token(&self) -> Option<String> {
        self.client.get_token().map(String::from)
    }

    /// Set the authentication token from metadata
    ///
    /// This should be called when loading stored credentials.
    /// The token will be verified on first use.
    pub fn set_auth_token(&mut self, token: String) {
        self.client.set_token(token);
    }
}

impl Provider for GitLabProvider {
    fn provider_type(&self) -> ProviderType {
        ProviderType::GitLab
    }

    fn check_authentication(&self) -> Result<()> {
        if self.authenticated {
            Ok(())
        } else {
            Err(Error::ProviderAuthRequired {
                provider: self.provider_type().to_string(),
                auth_command: "Run 'bt init' or authenticate manually".to_string(),
            })
        }
    }

    fn authenticate(&mut self) -> Result<()> {
        // Try to authenticate with the client
        // The client will try stored token first (if set), then external sources
        self.client
            .authenticate()
            .map_err(|e| Error::ProviderAuthRequired {
                provider: self.provider_type().to_string(),
                auth_command: format!(
                    "Authentication failed: {}\nCreate a Personal Access Token at https://gitlab.com/-/profile/personal_access_tokens",
                    e
                ),
            })?;

        self.authenticated = true;
        Ok(())
    }

    fn create_review(&mut self, params: CreateReviewParams) -> Result<Review> {
        let project_path = self.get_project_path()?;

        let mr = self
            .client
            .create_merge_request(
                project_path,
                &params.source_branch,
                &params.target_branch,
                &params.title,
                params.description.as_deref(),
                params.draft,
            )
            .map_err(|e| Error::provider_op(format!("Failed to create merge request: {}", e)))?;

        Ok(Self::mr_to_review(mr))
    }

    fn update_review(&mut self, params: UpdateReviewParams) -> Result<Review> {
        let project_path = self.get_project_path()?;

        // Parse the review ID as u64 (GitLab MR IID)
        let mr_iid: u64 = params
            .review_id
            .parse()
            .map_err(|_| Error::provider_op(format!("Invalid MR ID: {}", params.review_id)))?;

        let update_params = crate::providers::gitlab_api::UpdateMergeRequestParams {
            title: params.title,
            description: params.description,
            target_branch: params.target_branch,
            draft: None, // We don't update draft status during regular updates
        };

        let mr = self
            .client
            .update_merge_request(project_path, mr_iid, update_params)
            .map_err(|e| Error::provider_op(format!("Failed to update merge request: {}", e)))?;

        Ok(Self::mr_to_review(mr))
    }

    fn get_review(&mut self, review_id: &str) -> Result<Review> {
        let project_path = self.get_project_path()?;

        // Parse the review ID as u64 (GitLab MR IID)
        let mr_iid: u64 = review_id
            .parse()
            .map_err(|_| Error::provider_op(format!("Invalid MR ID: {}", review_id)))?;

        let mr = self
            .client
            .get_merge_request(project_path, mr_iid)
            .map_err(|e| Error::provider_op(format!("Failed to get merge request: {}", e)))?;

        Ok(Self::mr_to_review(mr))
    }

    fn find_review_for_branch(&mut self, _branch: &str) -> Result<Option<Review>> {
        // TODO: Implement finding MR by branch
        // This requires listing MRs with filters, which we haven't implemented yet
        // For MVP, we can use metadata to track branch -> MR mapping instead
        Err(Error::provider_op(
            "GitLab MR lookup by branch not yet implemented",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_type() {
        let provider = GitLabProvider::new("https://gitlab.com").unwrap();
        assert_eq!(provider.provider_type(), ProviderType::GitLab);
    }

    #[test]
    fn test_set_project_path() {
        let mut provider = GitLabProvider::new("https://gitlab.com").unwrap();
        provider.set_project_path("owner/repo".to_string());
        assert_eq!(provider.get_project_path().unwrap(), "owner/repo");
    }

    #[test]
    fn test_parse_review_state() {
        assert_eq!(
            GitLabProvider::parse_review_state("opened"),
            ReviewState::Open
        );
        assert_eq!(
            GitLabProvider::parse_review_state("closed"),
            ReviewState::Closed
        );
        assert_eq!(
            GitLabProvider::parse_review_state("merged"),
            ReviewState::Merged
        );
        assert_eq!(
            GitLabProvider::parse_review_state("unknown"),
            ReviewState::Open
        );
    }

    #[test]
    fn test_project_path_not_set() {
        let provider = GitLabProvider::new("https://gitlab.com").unwrap();
        assert!(provider.get_project_path().is_err());
    }
}
