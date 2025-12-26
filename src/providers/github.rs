//! GitHub provider implementation
//!
//! This provider uses the `gh` CLI to interact with GitHub.
//! All operations are delegated to the gh command-line tool.

#![allow(dead_code)] // Allow during early development

use crate::error::{Error, Result};
use crate::providers::{CreateReviewParams, Provider, ProviderType, Review, UpdateReviewParams};
use std::process::Command;

/// GitHub provider using gh CLI
pub struct GitHubProvider {}

impl GitHubProvider {
    /// Create a new GitHub provider
    pub fn new() -> Self {
        Self {}
    }

    /// Execute a gh command and return the output
    fn run_gh(&self, args: &[&str]) -> Result<std::process::Output> {
        let output = Command::new(self.cli_name())
            .args(args)
            .output()
            .map_err(|_e| Error::ProviderCliNotFound {
                provider: self.provider_type().to_string(),
                cli_name: self.cli_name().to_string(),
                install_url: self.install_url().to_string(),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::CommandFailed {
                command: format!("gh {}", args.join(" ")),
                exit_code: output.status.code().unwrap_or(-1),
                stderr: stderr.to_string(),
            });
        }

        Ok(output)
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

    fn check_cli_available(&self) -> Result<()> {
        Command::new(self.cli_name())
            .arg("--version")
            .output()
            .map_err(|_| Error::ProviderCliNotFound {
                provider: self.provider_type().to_string(),
                cli_name: self.cli_name().to_string(),
                install_url: self.install_url().to_string(),
            })?;

        Ok(())
    }

    fn check_authentication(&self) -> Result<()> {
        let output = self.run_gh(&["auth", "status"])?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.contains("Logged in") && !stdout.contains("✓") {
            return Err(Error::ProviderAuthRequired {
                provider: self.provider_type().to_string(),
                auth_command: self.auth_command().to_string(),
            });
        }

        Ok(())
    }

    fn create_review(&self, _params: CreateReviewParams) -> Result<Review> {
        // TODO: Implement PR creation via gh
        // Use: gh pr create --base <branch> --head <branch> --title <title> --body <desc> --draft --json
        Err(Error::provider_op("GitHub PR creation not yet implemented"))
    }

    fn update_review(&self, _params: UpdateReviewParams) -> Result<Review> {
        // TODO: Implement PR update via gh
        // Use: gh pr edit <number> --title <title> --body <desc> --ready/--draft --json
        Err(Error::provider_op("GitHub PR update not yet implemented"))
    }

    fn get_review(&self, _review_id: &str) -> Result<Review> {
        // TODO: Implement PR retrieval via gh
        // Use: gh pr view <number> --json
        Err(Error::provider_op(
            "GitHub PR retrieval not yet implemented",
        ))
    }

    fn find_review_for_branch(&self, _branch: &str) -> Result<Option<Review>> {
        // TODO: Implement finding PR by branch
        // Use: gh pr list --head <branch> --json
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

    // Note: Other tests require gh CLI to be installed and authenticated
    // Integration tests should be run in CI with proper setup
}
