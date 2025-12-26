//! Mock provider for testing
//!
//! This provider is used in tests to simulate provider operations
//! without requiring actual provider CLIs or network access.

#![allow(dead_code)] // Allow during early development

use crate::error::{Error, Result};
use crate::providers::{
    CreateReviewParams, Provider, ProviderType, Review, ReviewState, UpdateReviewParams,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Mock provider for testing
///
/// This provider stores reviews in memory and simulates provider operations
/// without making actual CLI calls or network requests.
#[derive(Clone)]
pub struct MockProvider {
    /// Internal state shared across clones
    state: Arc<Mutex<MockProviderState>>,
    /// Provider type to simulate
    provider_type: ProviderType,
}

#[derive(Default)]
struct MockProviderState {
    /// Reviews by ID
    reviews: HashMap<String, Review>,
    /// Reviews by branch name
    branch_to_review: HashMap<String, String>,
    /// Next review ID counter
    next_id: u32,
    /// Whether CLI is available
    cli_available: bool,
    /// Whether authentication is valid
    authenticated: bool,
    /// Simulate errors for testing
    should_fail_create: bool,
    should_fail_update: bool,
    should_fail_get: bool,
}

impl MockProvider {
    /// Create a new mock provider
    pub fn new(provider_type: ProviderType) -> Self {
        Self {
            state: Arc::new(Mutex::new(MockProviderState {
                cli_available: true,
                authenticated: true,
                next_id: 1,
                ..Default::default()
            })),
            provider_type,
        }
    }

    /// Create a new mock GitLab provider
    pub fn new_gitlab() -> Self {
        Self::new(ProviderType::GitLab)
    }

    /// Create a new mock GitHub provider
    pub fn new_github() -> Self {
        Self::new(ProviderType::GitHub)
    }

    /// Set whether the CLI should be reported as available
    pub fn set_cli_available(&self, available: bool) {
        self.state.lock().unwrap().cli_available = available;
    }

    /// Set whether authentication should be reported as valid
    pub fn set_authenticated(&self, authenticated: bool) {
        self.state.lock().unwrap().authenticated = authenticated;
    }

    /// Make create_review fail on next call
    pub fn fail_next_create(&self) {
        self.state.lock().unwrap().should_fail_create = true;
    }

    /// Make update_review fail on next call
    pub fn fail_next_update(&self) {
        self.state.lock().unwrap().should_fail_update = true;
    }

    /// Make get_review fail on next call
    pub fn fail_next_get(&self) {
        self.state.lock().unwrap().should_fail_get = true;
    }

    /// Get all reviews
    pub fn get_all_reviews(&self) -> Vec<Review> {
        self.state
            .lock()
            .unwrap()
            .reviews
            .values()
            .cloned()
            .collect()
    }

    /// Clear all reviews
    pub fn clear_reviews(&self) {
        let mut state = self.state.lock().unwrap();
        state.reviews.clear();
        state.branch_to_review.clear();
        state.next_id = 1;
    }

    /// Get review count
    pub fn review_count(&self) -> usize {
        self.state.lock().unwrap().reviews.len()
    }

    /// Generate the next review ID based on provider type
    fn next_review_id(&self, state: &mut MockProviderState) -> String {
        let id = state.next_id;
        state.next_id += 1;

        match self.provider_type {
            ProviderType::GitLab => format!("!{}", id),
            ProviderType::GitHub => format!("{}", id),
        }
    }
}

impl Provider for MockProvider {
    fn provider_type(&self) -> ProviderType {
        self.provider_type
    }

    fn check_cli_available(&self) -> Result<()> {
        let state = self.state.lock().unwrap();
        if state.cli_available {
            Ok(())
        } else {
            Err(Error::ProviderCliNotFound {
                provider: self.provider_type().to_string(),
                cli_name: self.cli_name().to_string(),
                install_url: self.install_url().to_string(),
            })
        }
    }

    fn check_authentication(&self) -> Result<()> {
        let state = self.state.lock().unwrap();
        if state.authenticated {
            Ok(())
        } else {
            Err(Error::ProviderAuthRequired {
                provider: self.provider_type().to_string(),
                auth_command: self.auth_command().to_string(),
            })
        }
    }

    fn create_review(&self, params: CreateReviewParams) -> Result<Review> {
        let mut state = self.state.lock().unwrap();

        if state.should_fail_create {
            state.should_fail_create = false;
            return Err(Error::provider_op("Simulated create failure"));
        }

        let review_id = self.next_review_id(&mut state);
        let url = match self.provider_type {
            ProviderType::GitLab => {
                format!(
                    "https://gitlab.com/mock/repo/-/merge_requests/{}",
                    review_id.trim_start_matches('!')
                )
            }
            ProviderType::GitHub => {
                format!("https://github.com/mock/repo/pull/{}", review_id)
            }
        };

        let review = Review {
            id: review_id.clone(),
            url,
            title: params.title,
            description: params.description,
            source_branch: params.source_branch.clone(),
            target_branch: params.target_branch,
            draft: params.draft,
            state: ReviewState::Open,
        };

        state.reviews.insert(review_id.clone(), review.clone());
        state
            .branch_to_review
            .insert(params.source_branch, review_id);

        Ok(review)
    }

    fn update_review(&self, params: UpdateReviewParams) -> Result<Review> {
        let mut state = self.state.lock().unwrap();

        if state.should_fail_update {
            state.should_fail_update = false;
            return Err(Error::provider_op("Simulated update failure"));
        }

        let review =
            state
                .reviews
                .get_mut(&params.review_id)
                .ok_or_else(|| Error::ReviewNotFound {
                    branch: params.review_id.clone(),
                })?;

        if let Some(title) = params.title {
            review.title = title;
        }
        if let Some(description) = params.description {
            review.description = description;
        }
        if let Some(target_branch) = params.target_branch {
            review.target_branch = target_branch;
        }
        if let Some(draft) = params.draft {
            review.draft = draft;
        }

        Ok(review.clone())
    }

    fn get_review(&self, review_id: &str) -> Result<Review> {
        let state = self.state.lock().unwrap();

        if state.should_fail_get {
            // Note: We don't reset the flag here because we can't mutate through &self
            return Err(Error::provider_op("Simulated get failure"));
        }

        state
            .reviews
            .get(review_id)
            .cloned()
            .ok_or_else(|| Error::ReviewNotFound {
                branch: review_id.to_string(),
            })
    }

    fn find_review_for_branch(&self, branch: &str) -> Result<Option<Review>> {
        let state = self.state.lock().unwrap();

        if let Some(review_id) = state.branch_to_review.get(branch) {
            Ok(state.reviews.get(review_id).cloned())
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_provider_creation() {
        let provider = MockProvider::new_gitlab();
        assert_eq!(provider.provider_type(), ProviderType::GitLab);

        let provider = MockProvider::new_github();
        assert_eq!(provider.provider_type(), ProviderType::GitHub);
    }

    #[test]
    fn test_cli_available() {
        let provider = MockProvider::new_gitlab();
        assert!(provider.check_cli_available().is_ok());

        provider.set_cli_available(false);
        assert!(provider.check_cli_available().is_err());
    }

    #[test]
    fn test_authentication() {
        let provider = MockProvider::new_gitlab();
        assert!(provider.check_authentication().is_ok());

        provider.set_authenticated(false);
        assert!(provider.check_authentication().is_err());
    }

    #[test]
    fn test_create_review_gitlab() {
        let provider = MockProvider::new_gitlab();

        let params = CreateReviewParams {
            source_branch: "feature".to_string(),
            target_branch: "main".to_string(),
            title: "Test MR".to_string(),
            description: "Test description".to_string(),
            draft: true,
        };

        let review = provider.create_review(params).unwrap();
        assert_eq!(review.id, "!1");
        assert_eq!(review.title, "Test MR");
        assert_eq!(review.source_branch, "feature");
        assert_eq!(review.target_branch, "main");
        assert!(review.draft);
        assert_eq!(review.state, ReviewState::Open);
        assert!(review.url.contains("gitlab.com"));
    }

    #[test]
    fn test_create_review_github() {
        let provider = MockProvider::new_github();

        let params = CreateReviewParams {
            source_branch: "feature".to_string(),
            target_branch: "main".to_string(),
            title: "Test PR".to_string(),
            description: "Test description".to_string(),
            draft: false,
        };

        let review = provider.create_review(params).unwrap();
        assert_eq!(review.id, "1");
        assert!(review.url.contains("github.com"));
    }

    #[test]
    fn test_update_review() {
        let provider = MockProvider::new_gitlab();

        let params = CreateReviewParams {
            source_branch: "feature".to_string(),
            target_branch: "main".to_string(),
            title: "Original Title".to_string(),
            description: "Original description".to_string(),
            draft: true,
        };

        let review = provider.create_review(params).unwrap();
        let review_id = review.id.clone();

        let update_params = UpdateReviewParams {
            review_id: review_id.clone(),
            title: Some("Updated Title".to_string()),
            description: None,
            target_branch: None,
            draft: Some(false),
        };

        let updated = provider.update_review(update_params).unwrap();
        assert_eq!(updated.title, "Updated Title");
        assert_eq!(updated.description, "Original description");
        assert!(!updated.draft);
    }

    #[test]
    fn test_get_review() {
        let provider = MockProvider::new_gitlab();

        let params = CreateReviewParams {
            source_branch: "feature".to_string(),
            target_branch: "main".to_string(),
            title: "Test MR".to_string(),
            description: "Test description".to_string(),
            draft: true,
        };

        let created = provider.create_review(params).unwrap();
        let fetched = provider.get_review(&created.id).unwrap();

        assert_eq!(created.id, fetched.id);
        assert_eq!(created.title, fetched.title);
    }

    #[test]
    fn test_find_review_for_branch() {
        let provider = MockProvider::new_gitlab();

        let params = CreateReviewParams {
            source_branch: "feature".to_string(),
            target_branch: "main".to_string(),
            title: "Test MR".to_string(),
            description: "Test description".to_string(),
            draft: true,
        };

        let created = provider.create_review(params).unwrap();

        let found = provider.find_review_for_branch("feature").unwrap().unwrap();
        assert_eq!(found.id, created.id);

        let not_found = provider.find_review_for_branch("nonexistent").unwrap();
        assert!(not_found.is_none());
    }

    #[test]
    fn test_simulated_failures() {
        let provider = MockProvider::new_gitlab();

        provider.fail_next_create();
        let params = CreateReviewParams {
            source_branch: "feature".to_string(),
            target_branch: "main".to_string(),
            title: "Test MR".to_string(),
            description: "Test description".to_string(),
            draft: true,
        };
        assert!(provider.create_review(params).is_err());
    }

    #[test]
    fn test_clear_reviews() {
        let provider = MockProvider::new_gitlab();

        let params = CreateReviewParams {
            source_branch: "feature".to_string(),
            target_branch: "main".to_string(),
            title: "Test MR".to_string(),
            description: "Test description".to_string(),
            draft: true,
        };

        provider.create_review(params).unwrap();
        assert_eq!(provider.review_count(), 1);

        provider.clear_reviews();
        assert_eq!(provider.review_count(), 0);
    }

    #[test]
    fn test_multiple_reviews_increment_id() {
        let provider = MockProvider::new_gitlab();

        let params1 = CreateReviewParams {
            source_branch: "feature-1".to_string(),
            target_branch: "main".to_string(),
            title: "First MR".to_string(),
            description: "First description".to_string(),
            draft: true,
        };

        let params2 = CreateReviewParams {
            source_branch: "feature-2".to_string(),
            target_branch: "main".to_string(),
            title: "Second MR".to_string(),
            description: "Second description".to_string(),
            draft: true,
        };

        let review1 = provider.create_review(params1).unwrap();
        let review2 = provider.create_review(params2).unwrap();

        assert_eq!(review1.id, "!1");
        assert_eq!(review2.id, "!2");
    }
}
