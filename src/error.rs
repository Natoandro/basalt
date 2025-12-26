//! Error types for basalt
//!
//! This module defines all error types used throughout the application.
//! We use thiserror for ergonomic error handling with proper context.

#![allow(dead_code)] // Allow during early development

use std::path::PathBuf;
use thiserror::Error;

/// Main error type for basalt operations
#[derive(Error, Debug)]
pub enum Error {
    /// Git-related errors
    #[error("Git error: {message}")]
    Git { message: String },

    /// Not in a git repository
    #[error("Not in a git repository. Run this command from inside a git repository.")]
    NotInGitRepository,

    /// Provider CLI not found
    #[error(
        "Provider CLI not found: {cli_name}\n\nThe {provider} provider requires the '{cli_name}' command-line tool.\nInstall it from: {install_url}"
    )]
    ProviderCliNotFound {
        provider: String,
        cli_name: String,
        install_url: String,
    },

    /// Provider authentication failed
    #[error("Not authenticated with {provider}.\n\nRun: {auth_command}")]
    ProviderAuthRequired {
        provider: String,
        auth_command: String,
    },

    /// Provider detection failed
    #[error(
        "Could not detect provider from git remote: {remote_url}\n\nSupported providers: GitLab, GitHub\nYou can manually specify a provider with: bt init --provider <provider>"
    )]
    ProviderDetectionFailed { remote_url: String },

    /// Unknown provider specified
    #[error("Unknown provider: {provider}\n\nSupported providers: gitlab, github")]
    UnknownProvider { provider: String },

    /// Provider operation failed
    #[error("Provider operation failed: {message}")]
    ProviderOperationFailed { message: String },

    /// Stack validation errors
    #[error("Invalid stack: {message}")]
    InvalidStack { message: String },

    /// Merge commits in stack
    #[error(
        "Stack contains merge commits. Stacks must be linear.\n\nBranch '{branch}' has a merge commit.\nUse 'git log --graph --oneline' to visualize the branch history."
    )]
    MergeCommitInStack { branch: String },

    /// No commits in stack
    #[error(
        "No commits in stack between '{current_branch}' and '{base_branch}'.\n\nEnsure you have commits to submit."
    )]
    EmptyStack {
        current_branch: String,
        base_branch: String,
    },

    /// Branch not found
    #[error("Branch not found: {branch}")]
    BranchNotFound { branch: String },

    /// Metadata errors
    #[error("Metadata error: {message}")]
    Metadata { message: String },

    /// Metadata file not found
    #[error("Metadata not found. Have you run 'bt init'?")]
    MetadataNotFound,

    /// Unsupported metadata version
    #[error(
        "Unsupported metadata version: {version}\n\nThis version of basalt supports metadata version {supported_version}.\nPlease upgrade basalt or migrate your metadata."
    )]
    UnsupportedMetadataVersion {
        version: String,
        supported_version: String,
    },

    /// Configuration errors
    #[error("Configuration error: {message}")]
    Config { message: String },

    /// Repository not initialized
    #[error("Repository not initialized. Run 'bt init' first.")]
    NotInitialized,

    /// Already initialized
    #[error("Repository already initialized at {path}")]
    AlreadyInitialized { path: PathBuf },

    /// Uncommitted changes
    #[error("You have uncommitted changes. Commit or stash them before proceeding.")]
    UncommittedChanges,

    /// Rebase in progress
    #[error(
        "A rebase is already in progress. Resolve conflicts and run 'git rebase --continue' or 'bt restack --continue'."
    )]
    RebaseInProgress,

    /// Review not found
    #[error("Review not found for branch: {branch}")]
    ReviewNotFound { branch: String },

    /// JSON parsing error
    #[error("Failed to parse JSON output: {message}")]
    JsonParse { message: String },

    /// YAML parsing error
    #[error("Failed to parse YAML: {message}")]
    YamlParse { message: String },

    /// TOML parsing error
    #[error("Failed to parse TOML: {message}")]
    TomlParse { message: String },

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Command execution failed
    #[error("Command failed: {command}\n\nExit code: {exit_code}\nStderr: {stderr}")]
    CommandFailed {
        command: String,
        exit_code: i32,
        stderr: String,
    },

    /// Generic error for unexpected situations
    #[error("{0}")]
    Other(String),
}

/// Result type alias for basalt operations
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// Create a Git error with a message
    pub fn git<S: Into<String>>(message: S) -> Self {
        Error::Git {
            message: message.into(),
        }
    }

    /// Create a metadata error with a message
    pub fn metadata<S: Into<String>>(message: S) -> Self {
        Error::Metadata {
            message: message.into(),
        }
    }

    /// Create a config error with a message
    pub fn config<S: Into<String>>(message: S) -> Self {
        Error::Config {
            message: message.into(),
        }
    }

    /// Create a provider operation error with a message
    pub fn provider_op<S: Into<String>>(message: S) -> Self {
        Error::ProviderOperationFailed {
            message: message.into(),
        }
    }

    /// Create an invalid stack error with a message
    pub fn invalid_stack<S: Into<String>>(message: S) -> Self {
        Error::InvalidStack {
            message: message.into(),
        }
    }

    /// Create a generic error
    pub fn other<S: Into<String>>(message: S) -> Self {
        Error::Other(message.into())
    }
}

/// Convert serde_json errors to our error type
impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::JsonParse {
            message: err.to_string(),
        }
    }
}

/// Convert serde_yaml errors to our error type
impl From<serde_yaml::Error> for Error {
    fn from(err: serde_yaml::Error) -> Self {
        Error::YamlParse {
            message: err.to_string(),
        }
    }
}

/// Convert toml errors to our error type
impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        Error::TomlParse {
            message: err.to_string(),
        }
    }
}
