//! GitLab provider implementation
//!
//! This provider uses the `glab` CLI to interact with GitLab.
//! All operations are delegated to the glab command-line tool.

#![allow(dead_code)] // Allow during early development

use crate::error::{Error, Result};
use crate::providers::{CreateReviewParams, Provider, ProviderType, Review, UpdateReviewParams};
use std::process::Command;

/// GitLab provider using glab CLI
pub struct GitLabProvider {}

impl GitLabProvider {
    /// Create a new GitLab provider
    pub fn new() -> Self {
        Self {}
    }

    /// Execute a glab command and return the output
    fn run_glab(&self, args: &[&str]) -> Result<std::process::Output> {
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
                command: format!("glab {}", args.join(" ")),
                exit_code: output.status.code().unwrap_or(-1),
                stderr: stderr.to_string(),
            });
        }

        Ok(output)
    }
}

impl Default for GitLabProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for GitLabProvider {
    fn provider_type(&self) -> ProviderType {
        ProviderType::GitLab
    }

    fn check_cli_available(&self) -> Result<()> {
        Command::new("glab")
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
        let output = self.run_glab(&["auth", "status"])?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.contains("Logged in") && !stdout.contains("Active account") {
            return Err(Error::ProviderAuthRequired {
                provider: self.provider_type().to_string(),
                auth_command: self.auth_command().to_string(),
            });
        }

        Ok(())
    }

    fn create_review(&self, _params: CreateReviewParams) -> Result<Review> {
        // TODO: Implement MR creation via glab
        // Use: glab mr create --source-branch <branch> --target-branch <branch> --title <title> --description <desc> --draft --json
        Err(Error::provider_op("GitLab MR creation not yet implemented"))
    }

    fn update_review(&self, _params: UpdateReviewParams) -> Result<Review> {
        // TODO: Implement MR update via glab
        // Use: glab mr update <id> --title <title> --description <desc> --ready/--draft --json
        Err(Error::provider_op("GitLab MR update not yet implemented"))
    }

    fn get_review(&self, _review_id: &str) -> Result<Review> {
        // TODO: Implement MR retrieval via glab
        // Use: glab mr view <id> --json
        Err(Error::provider_op(
            "GitLab MR retrieval not yet implemented",
        ))
    }

    fn find_review_for_branch(&self, _branch: &str) -> Result<Option<Review>> {
        // TODO: Implement finding MR by branch
        // Use: glab mr list --source-branch <branch> --json
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
        let provider = GitLabProvider::new();
        assert_eq!(provider.provider_type(), ProviderType::GitLab);
    }

    // Note: Other tests require glab CLI to be installed and authenticated
    // Integration tests should be run in CI with proper setup
}
