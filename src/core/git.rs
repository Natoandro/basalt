//! Git operations wrapper
//!
//! This module provides wrapper functions around git operations using gitoxide (`gix`).
//! All git operations should go through this module to maintain consistency.
//!
//! # Design Principles
//!
//! - Use gitoxide (`gix`) for all basic git operations
//! - Fall back to git CLI only for complex operations (if needed)
//! - Handle errors explicitly with proper context
//! - Never assume git command success
//! - Provide clear error messages with context
//!
//! # Why gitoxide?
//!
//! - **Performance**: No subprocess overhead
//! - **Type safety**: Rust types for git objects, refs, commits
//! - **Single binary**: No external git dependency for basic operations
//! - **Better errors**: Structured error types instead of parsing stderr
//!
//! # Example
//!
//! ```rust,ignore
//! use crate::core::git;
//!
//! // Get current branch
//! let branch = git::get_current_branch()?;
//! println!("Current branch: {}", branch);
//!
//! // Get remote URL
//! let url = git::get_remote_url("origin")?;
//! println!("Origin URL: {}", url);
//! ```

#![allow(dead_code)] // Allow during early development

use crate::error::{Error, Result};
use gix::bstr::ByteSlice;

/// Open a git repository at the current directory or any parent directory
///
/// # Errors
///
/// Returns an error if not in a git repository
fn open_repo() -> Result<gix::Repository> {
    gix::discover(".").map_err(|_| Error::NotInGitRepository)
}

/// Get the current branch name
///
/// # Errors
///
/// Returns an error if:
/// - Not in a git repository
/// - HEAD is detached (not on a branch)
/// - Git operation fails
pub fn get_current_branch() -> Result<String> {
    let repo = open_repo()?;

    let head = repo
        .head()
        .map_err(|e| Error::git(format!("Failed to get HEAD reference: {}", e)))?;

    // Check if HEAD is detached
    if head.is_detached() {
        return Err(Error::git("Detached HEAD state - not on a branch"));
    }

    // Get the branch name
    let reference = head
        .try_into_referent()
        .ok_or_else(|| Error::git("HEAD is not pointing to a branch"))?;

    let branch_name = reference
        .name()
        .shorten()
        .to_str()
        .map_err(|_| Error::git("Branch name is not valid UTF-8"))?
        .to_string();

    if branch_name.is_empty() {
        return Err(Error::git("Empty branch name"));
    }

    Ok(branch_name)
}

/// Get the URL of a git remote
///
/// # Arguments
///
/// * `remote_name` - Name of the remote (e.g., "origin")
///
/// # Errors
///
/// Returns an error if:
/// - Remote doesn't exist
/// - Remote has no URL configured
/// - Git operation fails
pub fn get_remote_url(remote_name: &str) -> Result<String> {
    let repo = open_repo()?;

    let remote = repo.find_remote(remote_name).map_err(|_| {
        Error::git(format!(
            "Remote '{}' not found. Available remotes: {}",
            remote_name,
            list_remotes().unwrap_or_default().join(", ")
        ))
    })?;

    let url = remote.url(gix::remote::Direction::Fetch).ok_or_else(|| {
        Error::git(format!(
            "Remote '{}' has no fetch URL configured",
            remote_name
        ))
    })?;

    let url_str = url
        .to_bstring()
        .to_str()
        .map_err(|_| Error::git(format!("Remote '{}' URL is not valid UTF-8", remote_name)))?
        .to_string();

    Ok(url_str)
}

/// Detect the default branch of the repository
///
/// Tries multiple strategies to find the default branch:
/// 1. Check symbolic-ref of origin/HEAD
/// 2. Look for common default branches (main, master)
/// 3. Fall back to "main" if nothing else works
///
/// # Errors
///
/// Returns an error if git operations fail unexpectedly.
/// Does not error if detection methods don't work - returns a sensible default.
pub fn detect_default_branch() -> Result<String> {
    let repo = open_repo()?;

    // Strategy 1: Check origin/HEAD symbolic ref
    if let Ok(origin_head) = repo.find_reference("refs/remotes/origin/HEAD") {
        let target = origin_head.target();
        if let gix::refs::TargetRef::Symbolic(name) = target {
            // Extract branch name from refs/remotes/origin/main -> main
            let name_str = name.as_bstr().to_str().unwrap_or("");
            if let Some(branch_name) = name_str.strip_prefix("refs/remotes/origin/") {
                return Ok(branch_name.to_string());
            }
        }
    }

    // Strategy 2: Check if common default branches exist locally
    for candidate in &["main", "master"] {
        if branch_exists(&repo, candidate)? {
            return Ok(candidate.to_string());
        }
    }

    // Strategy 3: Check if common default branches exist remotely
    for candidate in &["main", "master"] {
        let remote_ref = format!("refs/remotes/origin/{}", candidate);
        if repo.find_reference(&remote_ref).is_ok() {
            return Ok(candidate.to_string());
        }
    }

    // Strategy 4: Fall back to "main" as the modern default
    Ok("main".to_string())
}

/// Check if a branch exists (local or remote)
///
/// # Arguments
///
/// * `repo` - The git repository
/// * `branch_name` - Branch name (e.g., "main" or "origin/main")
///
/// # Errors
///
/// Returns an error if git operation fails
fn branch_exists(repo: &gix::Repository, branch_name: &str) -> Result<bool> {
    // Try as local branch first
    let local_ref = format!("refs/heads/{}", branch_name);
    if repo.find_reference(&local_ref).is_ok() {
        return Ok(true);
    }

    // Try as remote branch
    if branch_name.contains('/') {
        let remote_ref = format!("refs/remotes/{}", branch_name);
        if repo.find_reference(&remote_ref).is_ok() {
            return Ok(true);
        }
    }

    // Try as-is (might be a full ref path)
    Ok(repo.find_reference(branch_name).is_ok())
}

/// List all remote names
///
/// # Errors
///
/// Returns an error if git operation fails
pub fn list_remotes() -> Result<Vec<String>> {
    let repo = open_repo()?;

    let remote_names = repo.remote_names();
    let mut remotes = Vec::new();

    for name in remote_names.iter() {
        if let Ok(name_str) = name.to_str() {
            remotes.push(name_str.to_string());
        }
    }

    Ok(remotes)
}

/// Check if a branch has a remote tracking branch
///
/// # Arguments
///
/// * `branch_name` - Local branch name
///
/// # Errors
///
/// Returns an error if git operation fails
pub fn has_upstream(branch_name: &str) -> Result<bool> {
    let repo = open_repo()?;

    let branch_ref = format!("refs/heads/{}", branch_name);
    let _reference = repo
        .find_reference(&branch_ref)
        .map_err(|_| Error::git(format!("Branch '{}' not found", branch_name)))?;

    // Check if branch has an upstream configured
    let config = repo.config_snapshot();
    let upstream_key = format!("branch.{}.remote", branch_name);

    Ok(config.string(upstream_key.as_str()).is_some())
}

/// Get the upstream branch for a local branch
///
/// # Arguments
///
/// * `branch_name` - Local branch name
///
/// # Errors
///
/// Returns an error if:
/// - Branch doesn't exist
/// - Branch has no upstream configured
/// - Git operation fails
pub fn get_upstream(branch_name: &str) -> Result<String> {
    let repo = open_repo()?;

    let branch_ref = format!("refs/heads/{}", branch_name);
    let _reference = repo
        .find_reference(&branch_ref)
        .map_err(|_| Error::git(format!("Branch '{}' not found", branch_name)))?;

    let config = repo.config_snapshot();

    // Get remote and merge ref
    let remote_key = format!("branch.{}.remote", branch_name);
    let merge_key = format!("branch.{}.merge", branch_name);

    let remote = config.string(remote_key.as_str()).ok_or_else(|| {
        Error::git(format!(
            "Branch '{}' has no upstream configured",
            branch_name
        ))
    })?;

    let merge_ref = config.string(merge_key.as_str()).ok_or_else(|| {
        Error::git(format!(
            "Branch '{}' has no upstream configured",
            branch_name
        ))
    })?;

    // Extract branch name from refs/heads/main -> main
    let merge_ref_str = merge_ref.as_ref().to_str().unwrap_or("");
    let branch = merge_ref_str.strip_prefix("refs/heads/").ok_or_else(|| {
        Error::git(format!(
            "Invalid upstream ref format for branch '{}'",
            branch_name
        ))
    })?;

    // Return in format "remote/branch"
    let remote_str = remote.as_ref().to_str().unwrap_or("origin");
    Ok(format!("{}/{}", remote_str, branch))
}

/// Get the path to the repository root directory
///
/// # Errors
///
/// Returns an error if not in a git repository
pub fn get_repo_root() -> Result<std::path::PathBuf> {
    let repo = open_repo()?;
    let work_dir = repo
        .workdir()
        .ok_or_else(|| Error::git("Repository has no working directory (bare repo?)"))?;
    Ok(work_dir.to_path_buf())
}

/// Get the path to the .git directory
///
/// # Errors
///
/// Returns an error if not in a git repository
pub fn get_git_dir() -> Result<std::path::PathBuf> {
    let repo = open_repo()?;
    Ok(repo.git_dir().to_path_buf())
}

/// Check if there are uncommitted changes in the working directory
///
/// This includes both staged and unstaged changes.
///
/// Note: Currently uses git CLI for simplicity. This is one of the few operations
/// where the CLI is actually cleaner than using gitoxide's lower-level status API.
///
/// # Errors
///
/// Returns an error if git operation fails
pub fn has_uncommitted_changes() -> Result<bool> {
    use std::process::Command;

    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .map_err(|e| Error::git(format!("Failed to check git status: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::git(format!("Git status failed: {}", stderr)));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(!stdout.trim().is_empty())
}

/// Check if a rebase is in progress
///
/// # Errors
///
/// Returns an error if git operation fails
pub fn is_rebase_in_progress() -> Result<bool> {
    let git_dir = get_git_dir()?;

    // Check for various rebase state directories
    let rebase_merge = git_dir.join("rebase-merge");
    let rebase_apply = git_dir.join("rebase-apply");

    Ok(rebase_merge.exists() || rebase_apply.exists())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    // Note: These tests require being in a git repository
    // Integration tests should set up temporary git repositories for testing

    #[test]
    fn test_detect_default_branch_returns_something() {
        // Should always return a value, even if it's just the fallback
        let result = detect_default_branch();
        // May fail if not in a git repo, but that's expected
        if let Ok(branch) = result {
            assert!(!branch.is_empty());
        }
    }

    #[test]
    fn test_list_remotes_returns_vec() {
        // Should return a Vec, even if empty
        let result = list_remotes();
        // May fail if not in a git repo, but that's expected
        if let Ok(remotes) = result {
            // Just verify it's a valid Vec
            let _: Vec<String> = remotes;
        }
    }

    #[test]
    fn test_open_repo_in_git_repo() {
        // This test assumes we're running from within the basalt git repo
        let result = open_repo();
        // Should succeed when run from the basalt project
        if std::env::var("CI").is_ok() || Path::new(".git").exists() {
            assert!(
                result.is_ok(),
                "Should be able to open repo when in git directory"
            );
        }
    }
}
