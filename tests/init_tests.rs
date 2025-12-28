//! Integration tests for `bt init` command

use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// Helper to create a temporary git repository for testing
fn create_test_git_repo() -> TempDir {
    let temp_dir = TempDir::new().unwrap();

    // Initialize git repository
    Command::new("git")
        .args(["init"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to init git repo");

    // Configure git for commits
    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to configure git email");

    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to configure git name");

    temp_dir
}

/// Helper to add a git remote to a repository
fn add_git_remote(repo_path: &std::path::Path, name: &str, url: &str) {
    Command::new("git")
        .args(["remote", "add", name, url])
        .current_dir(repo_path)
        .output()
        .expect("Failed to add git remote");
}

/// Helper to create an initial commit (needed for some git operations)
fn create_initial_commit(repo_path: &std::path::Path) {
    // Create a file
    let readme_path = repo_path.join("README.md");
    fs::write(&readme_path, "# Test Repository\n").unwrap();

    // Stage and commit
    Command::new("git")
        .args(["add", "README.md"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to stage file");

    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to commit");
}

/// Helper to run bt init command
fn run_bt_init(
    repo_path: &std::path::Path,
    provider: Option<&str>,
    base_branch: Option<&str>,
) -> Result<(), String> {
    let mut args = vec!["init"];

    if let Some(p) = provider {
        args.push("--provider");
        args.push(p);
    }

    if let Some(b) = base_branch {
        args.push("--base-branch");
        args.push(b);
    }

    let output = Command::new(env!("CARGO_BIN_EXE_bt"))
        .args(&args)
        .current_dir(repo_path)
        .output()
        .expect("Failed to execute bt init");

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(stderr.to_string())
    }
}

#[test]
fn test_init_with_gitlab_remote() {
    let repo = create_test_git_repo();
    add_git_remote(repo.path(), "origin", "https://gitlab.com/user/repo.git");

    // Run init
    let result = run_bt_init(repo.path(), None, None);
    assert!(result.is_ok(), "Init should succeed: {:?}", result);

    // Check that basalt directory was created
    let basalt_dir = repo.path().join(".git/basalt");
    assert!(basalt_dir.exists(), "Basalt directory should exist");

    // Check that metadata file was created
    let metadata_file = basalt_dir.join("metadata.yml");
    assert!(metadata_file.exists(), "Metadata file should exist");

    // Check metadata content
    let metadata_content = fs::read_to_string(&metadata_file).unwrap();
    assert!(
        metadata_content.contains("version:"),
        "Should contain version"
    );
    assert!(
        metadata_content.contains("provider: gitlab"),
        "Should detect GitLab"
    );
    assert!(
        metadata_content.contains("base_branch:"),
        "Should contain base_branch"
    );
}

#[test]
fn test_init_with_github_remote() {
    let repo = create_test_git_repo();
    add_git_remote(repo.path(), "origin", "https://github.com/user/repo.git");

    // Run init
    let result = run_bt_init(repo.path(), None, None);
    assert!(result.is_ok(), "Init should succeed: {:?}", result);

    // Check metadata content
    let metadata_file = repo.path().join(".git/basalt/metadata.yml");
    let metadata_content = fs::read_to_string(&metadata_file).unwrap();
    assert!(
        metadata_content.contains("provider: github"),
        "Should detect GitHub"
    );
}

#[test]
fn test_init_with_provider_override() {
    let repo = create_test_git_repo();
    add_git_remote(repo.path(), "origin", "https://github.com/user/repo.git");

    // Run init with GitLab override (even though remote is GitHub)
    let result = run_bt_init(repo.path(), Some("gitlab"), None);
    assert!(result.is_ok(), "Init should succeed: {:?}", result);

    // Check that provider override was used
    let metadata_file = repo.path().join(".git/basalt/metadata.yml");
    let metadata_content = fs::read_to_string(&metadata_file).unwrap();
    assert!(
        metadata_content.contains("provider: gitlab"),
        "Should use GitLab override"
    );
}

#[test]
fn test_init_with_base_branch_override() {
    let repo = create_test_git_repo();
    add_git_remote(repo.path(), "origin", "https://gitlab.com/user/repo.git");

    // Run init with custom base branch
    let result = run_bt_init(repo.path(), None, Some("develop"));
    assert!(result.is_ok(), "Init should succeed: {:?}", result);

    // Check that base branch override was used
    let metadata_file = repo.path().join(".git/basalt/metadata.yml");
    let metadata_content = fs::read_to_string(&metadata_file).unwrap();
    assert!(
        metadata_content.contains("base_branch: develop"),
        "Should use develop as base"
    );
}

#[test]
fn test_init_fails_without_remote() {
    let repo = create_test_git_repo();
    // Don't add any remote

    // Run init without provider override
    let result = run_bt_init(repo.path(), None, None);
    assert!(result.is_err(), "Init should fail without remote");

    let error = result.unwrap_err();
    assert!(
        error.contains("No git remotes found") || error.contains("remote"),
        "Error should mention remotes: {}",
        error
    );
}

#[test]
fn test_init_succeeds_without_remote_with_provider_override() {
    let repo = create_test_git_repo();
    // Don't add any remote

    // Run init with provider override
    let result = run_bt_init(repo.path(), Some("gitlab"), None);
    assert!(
        result.is_ok(),
        "Init should succeed with provider override even without remote: {:?}",
        result
    );

    // Check metadata
    let metadata_file = repo.path().join(".git/basalt/metadata.yml");
    assert!(metadata_file.exists(), "Metadata file should exist");
}

#[test]
fn test_init_fails_if_already_initialized() {
    let repo = create_test_git_repo();
    add_git_remote(repo.path(), "origin", "https://gitlab.com/user/repo.git");

    // Initialize once
    let result = run_bt_init(repo.path(), None, None);
    assert!(result.is_ok(), "First init should succeed: {:?}", result);

    // Try to initialize again
    let result = run_bt_init(repo.path(), None, None);
    assert!(result.is_err(), "Second init should fail");

    let error = result.unwrap_err();
    assert!(
        error.contains("already initialized") || error.contains("Already initialized"),
        "Error should mention already initialized: {}",
        error
    );
}

#[test]
fn test_init_fails_outside_git_repo() {
    let temp_dir = TempDir::new().unwrap();
    // Don't initialize git

    let result = run_bt_init(temp_dir.path(), Some("gitlab"), None);
    assert!(result.is_err(), "Init should fail outside git repo");

    let error = result.unwrap_err();
    assert!(
        error.contains("git repository") || error.contains("Not in a git repository"),
        "Error should mention git repository: {}",
        error
    );
}

#[test]
fn test_init_with_ssh_remote() {
    let repo = create_test_git_repo();
    add_git_remote(repo.path(), "origin", "git@gitlab.com:user/repo.git");

    // Run init
    let result = run_bt_init(repo.path(), None, None);
    assert!(
        result.is_ok(),
        "Init should succeed with SSH remote: {:?}",
        result
    );

    // Check that provider was detected
    let metadata_file = repo.path().join(".git/basalt/metadata.yml");
    let metadata_content = fs::read_to_string(&metadata_file).unwrap();
    assert!(
        metadata_content.contains("provider: gitlab"),
        "Should detect GitLab from SSH URL"
    );
}

#[test]
fn test_init_detects_main_branch() {
    let repo = create_test_git_repo();
    add_git_remote(repo.path(), "origin", "https://gitlab.com/user/repo.git");

    // Create initial commit to establish main branch
    create_initial_commit(repo.path());

    // Rename to main
    Command::new("git")
        .args(["branch", "-M", "main"])
        .current_dir(repo.path())
        .output()
        .expect("Failed to rename branch");

    // Run init
    let result = run_bt_init(repo.path(), None, None);
    assert!(result.is_ok(), "Init should succeed: {:?}", result);

    // Check that main was detected
    let metadata_file = repo.path().join(".git/basalt/metadata.yml");
    let metadata_content = fs::read_to_string(&metadata_file).unwrap();
    assert!(
        metadata_content.contains("base_branch: main"),
        "Should detect main branch"
    );
}

#[test]
fn test_init_with_invalid_provider() {
    let repo = create_test_git_repo();
    add_git_remote(repo.path(), "origin", "https://gitlab.com/user/repo.git");

    // Try to init with invalid provider
    let result = run_bt_init(repo.path(), Some("invalid"), None);
    assert!(result.is_err(), "Init should fail with invalid provider");

    let error = result.unwrap_err();
    assert!(
        error.contains("Unknown provider") || error.contains("unknown"),
        "Error should mention unknown provider: {}",
        error
    );
}
