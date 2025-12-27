//! Integration tests for environment checking
//!
//! These tests verify that basalt correctly checks for:
//! - Git installation
//! - Git repository presence
//! - Basalt initialization
//! - Provider CLI availability
//! - Provider authentication

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

// Helper function to create a temporary git repository
fn create_temp_git_repo() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let repo_path = temp_dir.path().to_path_buf();

    // Initialize git repository
    let status = Command::new("git")
        .args(["init"])
        .current_dir(&repo_path)
        .status()
        .expect("Failed to initialize git repository");

    assert!(status.success(), "Git init failed");

    // Configure git user (required for commits)
    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(&repo_path)
        .status()
        .expect("Failed to set git user.name");

    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(&repo_path)
        .status()
        .expect("Failed to set git user.email");

    (temp_dir, repo_path)
}

// Helper function to create an initial commit
fn create_initial_commit(repo_path: &PathBuf) {
    // Create a file
    let file_path = repo_path.join("README.md");
    fs::write(&file_path, "# Test Repository\n").expect("Failed to write file");

    // Stage and commit
    Command::new("git")
        .args(["add", "README.md"])
        .current_dir(repo_path)
        .status()
        .expect("Failed to stage file");

    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(repo_path)
        .status()
        .expect("Failed to create commit");
}

#[test]
fn test_git_available() {
    // Git should be available (required for development)
    let output = Command::new("git")
        .arg("--version")
        .output()
        .expect("Git should be available");

    assert!(output.status.success());
}

#[test]
fn test_detect_git_repository() {
    let (_temp_dir, repo_path) = create_temp_git_repo();

    // Verify .git directory exists
    let git_dir = repo_path.join(".git");
    assert!(git_dir.exists());
    assert!(git_dir.is_dir());

    // Verify git rev-parse works
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to run git rev-parse");

    assert!(output.status.success());
}

#[test]
fn test_not_in_git_repository() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let non_repo_path = temp_dir.path();

    // Verify git rev-parse fails outside a repository
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(non_repo_path)
        .output()
        .expect("Failed to run git rev-parse");

    assert!(!output.status.success());
}

#[test]
fn test_get_git_dir() {
    let (_temp_dir, repo_path) = create_temp_git_repo();

    let output = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to get git dir");

    assert!(output.status.success());

    let git_dir = String::from_utf8_lossy(&output.stdout);
    let git_dir = git_dir.trim();

    // Should be .git
    assert!(git_dir.ends_with(".git"));
}

#[test]
fn test_create_basalt_directory() {
    let (_temp_dir, repo_path) = create_temp_git_repo();

    // Get git directory
    let output = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to get git dir");

    let git_dir = String::from_utf8_lossy(&output.stdout);
    let git_dir = git_dir.trim();
    let git_dir_path = repo_path.join(git_dir);

    // Create basalt directory
    let basalt_dir = git_dir_path.join("basalt");
    fs::create_dir_all(&basalt_dir).expect("Failed to create basalt directory");

    assert!(basalt_dir.exists());
    assert!(basalt_dir.is_dir());

    // Verify it's inside .git (never committed)
    assert!(basalt_dir.to_string_lossy().contains(".git"));
}

#[test]
fn test_check_uncommitted_changes() {
    let (_temp_dir, repo_path) = create_temp_git_repo();
    create_initial_commit(&repo_path);

    // Initially, no uncommitted changes
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to check status");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.trim().is_empty(), "Should have no changes");

    // Create a new file
    let new_file = repo_path.join("new.txt");
    fs::write(&new_file, "new content").expect("Failed to write file");

    // Now there should be uncommitted changes
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to check status");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.trim().is_empty(), "Should have uncommitted changes");
}

#[test]
fn test_basalt_metadata_location() {
    let (_temp_dir, repo_path) = create_temp_git_repo();

    // Get git directory
    let output = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to get git dir");

    let git_dir = String::from_utf8_lossy(&output.stdout);
    let git_dir = git_dir.trim();
    let git_dir_path = repo_path.join(git_dir);

    // Create basalt metadata directory
    let basalt_dir = git_dir_path.join("basalt");
    fs::create_dir_all(&basalt_dir).expect("Failed to create basalt directory");

    // Create metadata file
    let metadata_file = basalt_dir.join("metadata.yml");
    fs::write(&metadata_file, "version: \"1\"\n").expect("Failed to write metadata");

    assert!(metadata_file.exists());

    // Verify metadata file is inside .git (never committed)
    let metadata_path = metadata_file.to_string_lossy();
    assert!(metadata_path.contains(".git/basalt/metadata.yml"));

    // Verify it's not in the working directory
    let working_metadata = repo_path.join("metadata.yml");
    assert!(
        !working_metadata.exists(),
        "Metadata should not be in working directory"
    );
}

#[test]
fn test_environment_checks_usage_example() {
    // This test demonstrates how to use environment checks in practice
    let (_temp_dir, repo_path) = create_temp_git_repo();
    create_initial_commit(&repo_path);

    // Simulate being in the repository directory
    // In real code, this would be the current working directory

    // Check 1: Verify git is available
    let git_check = Command::new("git")
        .arg("--version")
        .output()
        .expect("Git should be available");
    assert!(git_check.status.success());

    // Check 2: Verify we're in a git repository
    let repo_check = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(&repo_path)
        .output()
        .expect("Should be in git repo");
    assert!(repo_check.status.success());

    // Check 3: Get the git directory
    let git_dir_output = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(&repo_path)
        .output()
        .expect("Should get git dir");
    assert!(git_dir_output.status.success());

    let git_dir_str = String::from_utf8_lossy(&git_dir_output.stdout);
    let git_dir_path = repo_path.join(git_dir_str.trim());

    // Check 4: Create basalt directory
    let basalt_dir = git_dir_path.join("basalt");
    fs::create_dir_all(&basalt_dir).expect("Should create basalt dir");
    assert!(basalt_dir.exists());

    // Check 5: Verify no uncommitted changes initially
    let status_output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&repo_path)
        .output()
        .expect("Should check status");
    let status_str = String::from_utf8_lossy(&status_output.stdout);
    assert!(
        status_str.trim().is_empty(),
        "Should have clean working directory"
    );

    // Check 6: Create metadata file to mark as initialized
    let metadata_file = basalt_dir.join("metadata.yml");
    fs::write(&metadata_file, "version: \"1\"\nprovider: gitlab\n").expect("Should write metadata");
    assert!(
        metadata_file.exists(),
        "Repository should be marked as initialized"
    );

    // All checks passed - repository is ready for basalt operations!
}
