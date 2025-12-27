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
//! // Check git is installed
//! environment::check_git_available()?;
//!
//! // Check provider prerequisites
//! let provider = create_provider(ProviderType::GitLab);
//! environment::check_provider_prerequisites(&*provider)?;
//! ```

#![allow(dead_code)] // Allow during early development

use crate::error::{Error, Result};
use crate::providers::Provider;
use std::path::PathBuf;
use std::process::Command;

/// Check if git is installed and available
///
/// This is a fundamental requirement for basalt to function.
///
/// # Errors
///
/// Returns an error if git is not found in PATH
pub fn check_git_available() -> Result<()> {
    Command::new("git")
        .arg("--version")
        .output()
        .map_err(|_| Error::other("Git is not installed or not in PATH.\n\nInstall git from: https://git-scm.com/downloads"))?;

    Ok(())
}

/// Check if we are inside a Git repository
///
/// Searches for a `.git` directory in the current directory or any parent directory.
///
/// # Errors
///
/// Returns `Error::NotInGitRepository` if not inside a git repository
pub fn require_git_repository() -> Result<PathBuf> {
    // Use git to find the repository root
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .map_err(|_| Error::NotInGitRepository)?;

    if !output.status.success() {
        return Err(Error::NotInGitRepository);
    }

    let repo_root = String::from_utf8_lossy(&output.stdout);
    let repo_root = repo_root.trim();

    if repo_root.is_empty() {
        return Err(Error::NotInGitRepository);
    }

    Ok(PathBuf::from(repo_root))
}

/// Get the path to the .git directory
///
/// This can be used to store basalt metadata inside `.git/basalt/`
///
/// # Errors
///
/// Returns an error if not in a git repository
pub fn get_git_dir() -> Result<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .map_err(|_| Error::NotInGitRepository)?;

    if !output.status.success() {
        return Err(Error::NotInGitRepository);
    }

    let git_dir = String::from_utf8_lossy(&output.stdout);
    let git_dir = git_dir.trim();

    if git_dir.is_empty() {
        return Err(Error::NotInGitRepository);
    }

    Ok(PathBuf::from(git_dir))
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
/// Returns an error if git command fails
pub fn has_uncommitted_changes() -> Result<bool> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()?;

    if !output.status.success() {
        return Err(Error::git("Failed to check git status"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(!stdout.trim().is_empty())
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
    let git_dir = get_git_dir()?;

    // Check for various rebase state directories
    let rebase_merge = git_dir.join("rebase-merge");
    let rebase_apply = git_dir.join("rebase-apply");

    Ok(rebase_merge.exists() || rebase_apply.exists())
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
    check_git_available()?;
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

    #[test]
    fn test_check_git_available() {
        // This test assumes git is installed (required for development)
        assert!(check_git_available().is_ok());
    }

    // Note: Other tests require being in a git repository
    // Integration tests should set up temporary git repositories for testing
}
