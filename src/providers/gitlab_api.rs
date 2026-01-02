//! GitLab REST API client
//!
//! This module provides a lightweight wrapper around the GitLab REST API,
//! implementing only the endpoints needed for basalt's MR operations.
//!
//! # Authentication
//!
//! The client attempts to find an authentication token in this order:
//! 1. Read from glab CLI config (`~/.config/glab-cli/config.yml`)
//! 2. Query git credential helper for gitlab.com
//! 3. Prompt user for a Personal Access Token (PAT)
//!
//! # API Endpoints
//!
//! - `GET /user` - Verify authentication
//! - `POST /projects/:id/merge_requests` - Create MR
//! - `PUT /projects/:id/merge_requests/:mr_iid` - Update MR
//! - `GET /projects/:id/merge_requests/:mr_iid` - Get MR details
//!
//! # Example
//!
//! ```rust,ignore
//! let client = GitLabClient::new("https://gitlab.com")?;
//! client.authenticate()?;
//!
//! let mr = client.create_merge_request(
//!     "owner/repo",
//!     "feature-branch",
//!     "main",
//!     "My Feature",
//!     "Description",
//!     true, // draft
//! )?;
//! ```

use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::process::Command;
use thiserror::Error;

/// GitLab API errors
#[derive(Debug, Error)]
pub enum GitLabError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("Failed to parse JSON response: {0}")]
    JsonParseFailed(#[from] serde_json::Error),

    #[error("Failed to parse YAML config: {0}")]
    YamlParseFailed(#[from] serde_yaml::Error),

    #[error("Authentication failed: Invalid or expired token")]
    AuthenticationFailed,

    #[error(
        "Token is missing required scope: {required}. Please create a token with '{required}' scope at https://gitlab.com/-/profile/personal_access_tokens"
    )]
    MissingScope { required: String },

    #[error("No authentication token available. Please authenticate.")]
    NoTokenAvailable,

    #[error("Merge request not found: !{0}")]
    MergeRequestNotFound(u64),

    #[error("API error ({status}): {message}")]
    ApiError { status: u16, message: String },

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, GitLabError>;

/// GitLab API client
pub struct GitLabClient {
    /// Base API URL (e.g., "https://gitlab.com/api/v4")
    api_url: String,
    /// HTTP client
    client: reqwest::blocking::Client,
    /// Authentication token (set after successful authentication)
    token: Option<String>,
}

/// GitLab user information (for auth verification)
#[derive(Debug, Deserialize)]
pub struct GitLabUser {
    pub id: u64,
    pub username: String,
    pub name: String,
}

/// GitLab Personal Access Token information
#[derive(Debug, Deserialize)]
pub struct GitLabToken {
    pub scopes: Vec<String>,
    pub active: bool,
}

/// GitLab merge request response
#[derive(Debug, Deserialize, Serialize)]
pub struct MergeRequest {
    pub iid: u64,
    pub id: u64,
    pub title: String,
    pub description: Option<String>,
    pub state: String,
    pub web_url: String,
    pub source_branch: String,
    pub target_branch: String,
    pub draft: bool,
}

/// Parameters for creating a merge request
#[derive(Debug, Serialize)]
pub struct CreateMergeRequestParams {
    pub source_branch: String,
    pub target_branch: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub draft: Option<bool>,
}

/// Parameters for updating a merge request
#[derive(Debug, Serialize)]
pub struct UpdateMergeRequestParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub draft: Option<bool>,
}

impl GitLabClient {
    /// Create a new GitLab API client
    ///
    /// # Arguments
    ///
    /// * `base_url` - Base URL of the GitLab instance (e.g., "https://gitlab.com")
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = GitLabClient::new("https://gitlab.com")?;
    /// ```
    pub fn new(base_url: &str) -> Result<Self> {
        let api_url = format!("{}/api/v4", base_url.trim_end_matches('/'));

        let client = reqwest::blocking::Client::builder()
            .user_agent("basalt-cli")
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        Ok(Self {
            api_url,
            client,
            token: None,
        })
    }

    /// Set the authentication token directly
    ///
    /// This is used when loading a stored token from config.
    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    /// Get the current token
    pub fn get_token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    /// Authenticate with GitLab
    ///
    /// Attempts to find an authentication token in this order:
    /// 1. Use already set token (from stored metadata)
    /// 2. Read from glab CLI config
    /// 3. Query git credential helper
    /// 4. Offer CLI auth (if glab available) or prompt for PAT
    ///
    /// After finding a token, verifies it by calling GET /user
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No token can be found or obtained
    /// - The token is invalid (API returns 401)
    /// - Network error occurs
    pub fn authenticate(&mut self) -> Result<GitLabUser> {
        // If we already have a token, try to verify it first
        if self.token.is_some() {
            match self.verify_token_scopes().and_then(|_| self.verify_auth()) {
                Ok(user) => return Ok(user),
                Err(_) => {
                    // Token is invalid/expired, clear it and get a new one
                    self.token = None;
                }
            }
        }

        // Try to get token from various sources
        let token = self
            .try_glab_token()
            .or_else(|| self.try_git_credential())
            .or_else(|| self.try_cli_auth())
            .or_else(|| self.prompt_for_token())
            .ok_or_else(|| GitLabError::NoTokenAvailable)?;

        // Verify the token works and has required scopes
        self.token = Some(token);
        self.verify_token_scopes()?;
        let user = self.verify_auth()?;

        Ok(user)
    }

    /// Verify authentication by calling GET /user
    fn verify_auth(&self) -> Result<GitLabUser> {
        let token = self
            .token
            .as_ref()
            .ok_or_else(|| GitLabError::NoTokenAvailable)?;

        let response = self
            .client
            .get(format!("{}/user", self.api_url))
            .header("PRIVATE-TOKEN", token)
            .send()?;

        if response.status() == 401 {
            return Err(GitLabError::AuthenticationFailed);
        }

        let user = response.error_for_status()?.json::<GitLabUser>()?;
        Ok(user)
    }

    /// Verify token has required scopes
    ///
    /// Checks that the token has the 'api' scope which is required for
    /// creating and managing merge requests.
    fn verify_token_scopes(&self) -> Result<()> {
        let token = self
            .token
            .as_ref()
            .ok_or_else(|| GitLabError::NoTokenAvailable)?;

        // Check token info via the personal access tokens API
        let response = self
            .client
            .get(format!("{}/personal_access_tokens/self", self.api_url))
            .header("PRIVATE-TOKEN", token)
            .send()?;

        if response.status() == 401 {
            return Err(GitLabError::AuthenticationFailed);
        }

        let token_info: GitLabToken = response.error_for_status()?.json()?;

        // Check if token has 'api' scope
        if !token_info.scopes.contains(&"api".to_string()) {
            return Err(GitLabError::MissingScope {
                required: "api".to_string(),
            });
        }

        if !token_info.active {
            return Err(GitLabError::AuthenticationFailed);
        }

        Ok(())
    }

    /// Try to read token from glab CLI config
    ///
    /// Reads from `~/.config/glab-cli/config.yml`
    fn try_glab_token(&self) -> Option<String> {
        let config_path = dirs::home_dir()?.join(".config/glab-cli/config.yml");
        let contents = std::fs::read_to_string(config_path).ok()?;

        // Parse YAML to extract token
        let config: serde_yaml::Value = serde_yaml::from_str(&contents).ok()?;

        // glab config structure: hosts -> gitlab.com -> token
        let host = self.extract_host_from_api_url();
        let token = config
            .get("hosts")?
            .get(host)?
            .get("token")?
            .as_str()?
            .to_string();

        Some(token)
    }

    /// Try to get token from git credential helper
    ///
    /// Runs `git credential fill` with the GitLab host
    fn try_git_credential(&self) -> Option<String> {
        let host = self.extract_host_from_api_url();

        let input = format!("protocol=https\nhost={}\n\n", host);

        let output = Command::new("git")
            .args(["credential", "fill"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
            .ok()?
            .stdin
            .as_mut()?
            .write_all(input.as_bytes())
            .ok()
            .and_then(|_| {
                Command::new("git")
                    .args(["credential", "fill"])
                    .output()
                    .ok()
            })?;

        if !output.status.success() {
            return None;
        }

        // Parse output for password field
        let stdout = String::from_utf8(output.stdout).ok()?;
        for line in stdout.lines() {
            if let Some(password) = line.strip_prefix("password=") {
                return Some(password.to_string());
            }
        }

        None
    }

    /// Try to authenticate using CLI tool (glab)
    ///
    /// Checks if glab is available and offers to run `glab auth login`
    fn try_cli_auth(&self) -> Option<String> {
        // Check if glab is available
        if Command::new("glab").arg("--version").output().is_err() {
            return None;
        }

        eprintln!();
        eprintln!("GitLab authentication required.");
        eprintln!();
        eprintln!("Choose authentication method:");
        eprintln!("  1) Use glab CLI (will run 'glab auth login')");
        eprintln!("  2) Enter Personal Access Token manually");
        eprintln!();
        eprint!("Enter choice (1 or 2): ");
        io::stderr().flush().ok()?;

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).ok()?;

        match choice.trim() {
            "1" => {
                eprintln!();
                eprintln!("Running 'glab auth login'...");
                eprintln!();

                let status = Command::new("glab").args(["auth", "login"]).status().ok()?;

                if status.success() {
                    // Try to read the token that was just created
                    self.try_glab_token()
                } else {
                    eprintln!("glab authentication failed");
                    None
                }
            }
            "2" => self.prompt_for_token(),
            _ => {
                eprintln!("Invalid choice, falling back to manual token entry");
                self.prompt_for_token()
            }
        }
    }

    /// Prompt user for a Personal Access Token
    fn prompt_for_token(&self) -> Option<String> {
        eprintln!();
        eprintln!("GitLab authentication required.");
        eprintln!();
        eprintln!("Please create a Personal Access Token (PAT) with 'api' scope:");
        eprintln!("  https://gitlab.com/-/profile/personal_access_tokens");
        eprintln!();
        eprint!("Enter your GitLab Personal Access Token: ");
        io::stderr().flush().ok()?;

        let mut token = String::new();
        io::stdin().read_line(&mut token).ok()?;

        let token = token.trim().to_string();
        if token.is_empty() { None } else { Some(token) }
    }

    /// Extract host from API URL
    ///
    /// Example: "https://gitlab.com/api/v4" -> "gitlab.com"
    fn extract_host_from_api_url(&self) -> &str {
        self.api_url
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .split('/')
            .next()
            .unwrap_or("gitlab.com")
    }

    /// Create a merge request
    ///
    /// # Arguments
    ///
    /// * `project_path` - Project path (e.g., "owner/repo")
    /// * `source_branch` - Source branch name
    /// * `target_branch` - Target branch name
    /// * `title` - MR title
    /// * `description` - MR description (optional)
    /// * `draft` - Whether to create as draft
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Not authenticated
    /// - Project doesn't exist
    /// - Branch doesn't exist
    /// - Network error occurs
    pub fn create_merge_request(
        &self,
        project_path: &str,
        source_branch: &str,
        target_branch: &str,
        title: &str,
        description: Option<&str>,
        draft: bool,
    ) -> Result<MergeRequest> {
        let token = self
            .token
            .as_ref()
            .ok_or_else(|| GitLabError::NoTokenAvailable)?;

        let project_id = urlencoding::encode(project_path);
        let url = format!("{}/projects/{}/merge_requests", self.api_url, project_id);

        let params = CreateMergeRequestParams {
            source_branch: source_branch.to_string(),
            target_branch: target_branch.to_string(),
            title: title.to_string(),
            description: description.map(String::from),
            draft: Some(draft),
        };

        let response = self
            .client
            .post(&url)
            .header("PRIVATE-TOKEN", token)
            .json(&params)
            .send()?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().unwrap_or_default();
            return Err(GitLabError::ApiError {
                status,
                message: body,
            });
        }

        let mr = response.json::<MergeRequest>()?;
        Ok(mr)
    }

    /// Update a merge request
    ///
    /// # Arguments
    ///
    /// * `project_path` - Project path (e.g., "owner/repo")
    /// * `mr_iid` - Merge request IID (internal ID, not global ID)
    /// * `params` - Update parameters
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Not authenticated
    /// - MR doesn't exist
    /// - Network error occurs
    pub fn update_merge_request(
        &self,
        project_path: &str,
        mr_iid: u64,
        params: UpdateMergeRequestParams,
    ) -> Result<MergeRequest> {
        let token = self
            .token
            .as_ref()
            .ok_or_else(|| GitLabError::NoTokenAvailable)?;

        let project_id = urlencoding::encode(project_path);
        let url = format!(
            "{}/projects/{}/merge_requests/{}",
            self.api_url, project_id, mr_iid
        );

        let response = self
            .client
            .put(&url)
            .header("PRIVATE-TOKEN", token)
            .json(&params)
            .send()?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().unwrap_or_default();
            return Err(GitLabError::ApiError {
                status,
                message: body,
            });
        }

        let mr = response.json::<MergeRequest>()?;
        Ok(mr)
    }

    /// Get a merge request
    ///
    /// # Arguments
    ///
    /// * `project_path` - Project path (e.g., "owner/repo")
    /// * `mr_iid` - Merge request IID (internal ID, not global ID)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Not authenticated
    /// - MR doesn't exist
    /// - Network error occurs
    pub fn get_merge_request(&self, project_path: &str, mr_iid: u64) -> Result<MergeRequest> {
        let token = self
            .token
            .as_ref()
            .ok_or_else(|| GitLabError::NoTokenAvailable)?;

        let project_id = urlencoding::encode(project_path);
        let url = format!(
            "{}/projects/{}/merge_requests/{}",
            self.api_url, project_id, mr_iid
        );

        let response = self
            .client
            .get(&url)
            .header("PRIVATE-TOKEN", token)
            .send()?;

        if response.status() == 404 {
            return Err(GitLabError::MergeRequestNotFound(mr_iid));
        }

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().unwrap_or_default();
            return Err(GitLabError::ApiError {
                status,
                message: body,
            });
        }

        let mr = response.json::<MergeRequest>()?;
        Ok(mr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = GitLabClient::new("https://gitlab.com").unwrap();
        assert_eq!(client.api_url, "https://gitlab.com/api/v4");
        assert!(client.token.is_none());
    }

    #[test]
    fn test_extract_host() {
        let client = GitLabClient::new("https://gitlab.com").unwrap();
        assert_eq!(client.extract_host_from_api_url(), "gitlab.com");

        let client = GitLabClient::new("https://gitlab.example.com").unwrap();
        assert_eq!(client.extract_host_from_api_url(), "gitlab.example.com");
    }

    #[test]
    fn test_api_url_normalization() {
        let client = GitLabClient::new("https://gitlab.com/").unwrap();
        assert_eq!(client.api_url, "https://gitlab.com/api/v4");

        let client = GitLabClient::new("https://gitlab.com").unwrap();
        assert_eq!(client.api_url, "https://gitlab.com/api/v4");
    }
}
