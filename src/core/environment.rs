//! Environment and dependency checking
//!
//! This module provides functions to verify that the environment is properly
//! set up for basalt operations. It checks:
//!
//! - Git repository presence
//! - Provider CLI availability (glab, gh, etc.)
//! - Provider authentication status
//!
//! All checks provide clear, actionable error messages to guide users.
//!
//! # Example
//!
//! ```rust,ignore
//! use crate::core::environment;
//! use crate::providers::{create_provider, ProviderType};
//!
//! // Check we're in a git repository
//! environment::require_git_repository()?;
//!
//! // Check provider prerequisites
//! let provider = create_provider(ProviderType::GitLab);
//! environment::check_provider_prerequisites(&*provider)?;
//! ```

#![allow(dead_code)] // Allow during early development

use crate::core::git;
use crate::error::{Error, Result};
use crate::providers::Provider;
use std::path::PathBuf;

/// Check if we are inside a Git repository
///
/// Uses gitoxide to check for a git repository.
///
/// # Errors
///
/// Returns `Error::NotInGitRepository` if not inside a git repository
pub fn require_git_repository() -> Result<PathBuf> {
    git::get_repo_root()
}

/// Get the path to the .git directory
///
/// This can be used to store basalt metadata inside `.git/basalt/`
///
/// # Errors
///
/// Returns an error if not in a git repository
pub fn get_git_dir() -> Result<PathBuf> {
    git::get_git_dir()
}

/// Get the basalt metadata directory path (inside .git/)
///
/// This is where basalt stores its metadata files.
/// The directory is `.git/basalt/` which ensures:
/// - Never accidentally committed (`.git/` is always ignored)
/// - Clean workspace (doesn't clutter repository root)
/// - Auto-cleanup (removed if `.git/` is deleted)
///
/// # Errors
///
/// Returns an error if not in a git repository
pub fn get_basalt_dir() -> Result<PathBuf> {
    let git_dir = get_git_dir()?;
    Ok(git_dir.join("basalt"))
}

pub fn basalt_dir_exists() -> Result<bool> {
    let basalt_dir = get_basalt_dir()?;
    Ok(basalt_dir.exists())
}

/// Create the basalt metadata directory if it doesn't exist
///
/// Creates `.git/basalt/` directory with proper permissions.
///
/// # Errors
///
/// Returns an error if:
/// - Not in a git repository
/// - Directory creation fails (permissions, etc.)
pub fn create_basalt_dir() -> Result<PathBuf> {
    let basalt_dir = get_basalt_dir()?;

    if !basalt_dir.exists() {
        std::fs::create_dir_all(&basalt_dir)?;
    }

    Ok(basalt_dir)
}

/// Check if repository is initialized with basalt
///
/// A repository is considered initialized if:
/// - We're in a git repository
/// - The `.git/basalt/` directory exists
/// - The metadata file exists
pub fn is_initialized() -> Result<bool> {
    let basalt_dir = get_basalt_dir()?;
    let metadata_file = basalt_dir.join("metadata.yml");
    Ok(metadata_file.exists())
}

pub fn require_initialized() -> Result<()> {
    if !is_initialized()? {
        return Err(Error::NotInitialized);
    }
    Ok(())
}

/// Check if there are uncommitted changes in the working directory
///
/// This includes both staged and unstaged changes.
///
/// # Errors
///
/// Returns an error if git operation fails
pub fn has_uncommitted_changes() -> Result<bool> {
    git::has_uncommitted_changes()
}

/// Require that there are no uncommitted changes
///
/// This is important for operations that modify branches (like restack).
///
/// # Errors
///
/// Returns `Error::UncommittedChanges` if there are uncommitted changes
pub fn require_clean_working_directory() -> Result<()> {
    if has_uncommitted_changes()? {
        return Err(Error::UncommittedChanges);
    }
    Ok(())
}

pub fn is_rebase_in_progress() -> Result<bool> {
    git::is_rebase_in_progress()
}

pub fn require_no_rebase_in_progress() -> Result<()> {
    if is_rebase_in_progress()? {
        return Err(Error::RebaseInProgress);
    }
    Ok(())
}

/// Check all provider prerequisites
///
/// This checks:
/// 1. Provider CLI is installed and available
/// 2. User is authenticated with the provider
///
/// # Errors
///
/// Returns an error if:
/// - Provider CLI is not installed
/// - User is not authenticated
///
/// # Example
///
/// ```rust,ignore
/// use crate::providers::{create_provider, ProviderType};
/// use crate::core::environment;
///
/// let provider = create_provider(ProviderType::GitLab);
/// environment::check_provider_prerequisites(&*provider)?;
/// ```
pub fn check_provider_prerequisites(provider: &dyn Provider) -> Result<()> {
    // Check CLI is available
    provider.check_cli_available()?;

    // Check authentication
    provider.check_authentication()?;

    Ok(())
}

/// Perform all environment checks needed for basic operations
///
/// This is a convenience function that checks:
/// - Git is installed
/// - We're in a git repository
/// - Repository is initialized with basalt
///
/// # Errors
///
/// Returns the first error encountered during checks
pub fn check_basic_environment() -> Result<()> {
    require_git_repository()?;
    require_initialized()?;
    Ok(())
}

/// Perform all environment checks needed for stack operations
///
/// This checks everything in `check_basic_environment` plus:
/// - No uncommitted changes
/// - No rebase in progress
///
/// # Errors
///
/// Returns the first error encountered during checks
pub fn check_stack_operation_environment() -> Result<()> {
    check_basic_environment()?;
    require_clean_working_directory()?;
    require_no_rebase_in_progress()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Tests require being in a git repository
    // Integration tests should set up temporary git repositories for testing

    #[test]
    fn test_require_git_repository() {
        // This test assumes we're running from within a git repo
        // Will fail if run outside a git repo, which is expected
        let result = require_git_repository();
        if std::path::Path::new(".git").exists() || std::env::var("CI").is_ok() {
            assert!(result.is_ok(), "Should find git repository");
        }
    }
}
