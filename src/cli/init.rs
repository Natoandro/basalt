//! Implementation of the `bt init` command
//!
//! This module handles repository initialization, which includes:
//! - Creating the `.git/basalt/` directory
//! - Auto-detecting the Git provider from remote URL
//! - Detecting the default base branch
//! - Creating initial metadata file
//!
//! # Example
//!
//! ```rust,ignore
//! use crate::cli::init::run_init;
//!
//! // Initialize with auto-detection
//! run_init(None, None)?;
//!
//! // Initialize with explicit provider
//! run_init(Some("gitlab".to_string()), None)?;
//!
//! // Initialize with custom base branch
//! run_init(None, Some("develop".to_string()))?;
//! ```

use crate::core::{environment, git, metadata};
use crate::error::{Error, Result};
use crate::providers::ProviderType;

/// Run the init command
///
/// Initializes a basalt repository by:
/// 1. Checking we're in a git repository
/// 2. Detecting or validating the provider
/// 3. Detecting or using the specified base branch
/// 4. Creating the metadata directory
/// 5. Saving initial metadata
///
/// # Arguments
///
/// * `provider_override` - Optional provider to use (overrides auto-detection)
/// * `base_branch_override` - Optional base branch (overrides auto-detection)
///
/// # Errors
///
/// Returns an error if:
/// - Not in a git repository
/// - Git is not installed
/// - Provider detection fails (and no override provided)
/// - Failed to create basalt directory
/// - Failed to save metadata
/// - Repository is already initialized
pub fn run_init(
    provider_override: Option<String>,
    base_branch_override: Option<String>,
) -> Result<()> {
    // Check prerequisites (gitoxide will fail if git repo doesn't exist)
    let repo_root = environment::require_git_repository()?;

    // Check if already initialized
    if environment::is_initialized()? {
        let basalt_dir = environment::get_basalt_dir()?;
        return Err(Error::AlreadyInitialized { path: basalt_dir });
    }

    println!("🚀 Initializing basalt repository...\n");

    // Detect or validate provider
    let provider = detect_provider(provider_override)?;
    println!("✓ Provider: {}", provider);

    // Detect or use base branch
    let base_branch = detect_base_branch(base_branch_override)?;
    println!("✓ Base branch: {}", base_branch);

    // Create basalt directory
    let basalt_dir = environment::create_basalt_dir()?;
    println!("✓ Created metadata directory: {}", basalt_dir.display());

    // Create and save metadata
    let metadata = metadata::Metadata::new(provider, base_branch.clone());
    metadata::save_metadata(&metadata)?;
    println!(
        "✓ Saved metadata: {}",
        basalt_dir.join("metadata.yml").display()
    );

    println!("\n✨ Successfully initialized basalt!");
    println!("\nRepository root: {}", repo_root.display());
    println!("Next steps:");
    println!("  1. Create a branch: git checkout -b feature-part-1");
    println!("  2. Make changes and commit");
    println!("  3. Submit your stack: bt submit");

    Ok(())
}

/// Detect the provider type
///
/// If a provider override is provided, validates and uses it.
/// Otherwise, attempts to auto-detect from git remote URL.
///
/// # Arguments
///
/// * `provider_override` - Optional provider string (e.g., "gitlab", "github")
///
/// # Errors
///
/// Returns an error if:
/// - Provider override is invalid
/// - Auto-detection fails (no remotes or unrecognized URL)
fn detect_provider(provider_override: Option<String>) -> Result<ProviderType> {
    if let Some(provider_str) = provider_override {
        // Use explicit provider
        let provider = ProviderType::from_str(&provider_str)?;
        println!("  Using explicitly specified provider");
        return Ok(provider);
    }

    // Auto-detect from git remote
    println!("  Auto-detecting provider from git remote...");

    // Get list of remotes
    let remotes = git::list_remotes()?;

    if remotes.is_empty() {
        return Err(Error::config(
            "No git remotes found. Add a remote first:\n  git remote add origin <url>\n\nOr specify a provider explicitly:\n  bt init --provider <gitlab|github>",
        ));
    }

    // Try 'origin' first, then fall back to first remote
    let remote_name = if remotes.contains(&"origin".to_string()) {
        "origin"
    } else {
        &remotes[0]
    };

    let remote_url = git::get_remote_url(remote_name)?;
    println!("  Checking remote '{}': {}", remote_name, remote_url);

    ProviderType::from_remote_url(&remote_url)
}

/// Detect or use the base branch
///
/// If a base branch override is provided, uses it.
/// Otherwise, attempts to auto-detect the default branch.
///
/// # Arguments
///
/// * `base_branch_override` - Optional base branch name
///
/// # Errors
///
/// Returns an error if git commands fail
fn detect_base_branch(base_branch_override: Option<String>) -> Result<String> {
    if let Some(branch) = base_branch_override {
        println!("  Using explicitly specified base branch");
        return Ok(branch);
    }

    println!("  Auto-detecting base branch...");
    git::detect_default_branch()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_provider_with_override() {
        let result = detect_provider(Some("gitlab".to_string()));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ProviderType::GitLab);

        let result = detect_provider(Some("github".to_string()));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ProviderType::GitHub);

        let result = detect_provider(Some("unknown".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_base_branch_with_override() {
        let result = detect_base_branch(Some("develop".to_string()));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "develop");
    }
}
