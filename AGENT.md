# AGENT.md — AI Assistant Guidelines for basalt

This document provides context and guidelines for AI coding assistants working on the **basalt** project.

---

## Project Overview

**basalt** is a Rust-based CLI tool (`bt`) for managing stacked Git workflows across multiple providers (GitLab, GitHub, etc.).

### Current State

- **Phase**: Early Development
- **Codebase**: Rust project initialized; contains legacy Charcoal (TypeScript/Node.js) for GitHub
- **Target**: Multi-provider support starting with GitLab
- **MVP**: GitLab provider with core stacking features
- **Project setup**: ✅ Complete (Cargo initialized, CLI framework in place, CI configured)

### Key Facts

- **Command name**: `bt` (basalt)
- **Primary language**: Rust (future), TypeScript (current/legacy)
- **First provider target**: GitLab (using `glab` CLI)
- **Second provider target**: GitHub (using `gh` CLI)
- **Philosophy**: Local-first, provider-agnostic, CLI-native
- **Metadata format**: YAML (human-readable, easy to edit)

---

## Architectural Principles

### 1. Provider Abstraction is Sacred

The entire architecture revolves around a clean provider abstraction:

```rust
// Example trait structure (conceptual)
trait Provider {
    fn authenticate(&self) -> Result<()>;
    fn create_review(&self, branch: &Branch) -> Result<Review>;
    fn update_review(&self, review: &Review) -> Result<()>;
    fn get_review(&self, id: &str) -> Result<Review>;
}
```

**Rules:**
- ✅ All provider-specific logic goes in provider implementations
- ✅ Core stack logic must be provider-agnostic
- ❌ Never hardcode provider-specific assumptions in core code
- ❌ Never bypass the provider abstraction

### 2. Direct API Integration with Smart Authentication

basalt calls provider REST APIs directly using HTTP clients:

- GitLab → Direct REST API calls via `reqwest`
- GitHub → Direct REST API calls via `reqwest` (future)
- Parse JSON responses using `serde`
- Smart authentication: read existing tokens or prompt for PAT

**Authentication Priority:**
1. Read provider CLI token if available (e.g., glab's token for GitLab)
2. Fallback to git credential helper
3. Prompt user for Personal Access Token (PAT)

**Why:**
- Full control over API interactions
- Minimal dependencies (no external CLI required)
- Better error handling and debugging
- Faster (no subprocess overhead)
- Transparent API usage

### 3. Local-First Philosophy

- Stack metadata lives in `.git/basalt/` directory (inside `.git/` so it's never committed)
- Never require network for read-only operations
- Git is the source of truth for branch structure
- Metadata augments Git, doesn't replace it

### 4. Explicit Over Implicit

- Operations should be predictable and transparent
- Fail loudly with clear error messages
- Don't make assumptions about user intent
- Require explicit flags for destructive operations

---

## Code Organization

### Expected Rust Project Structure

```
basalt/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── cli/                 # Command definitions (clap)
│   ├── core/                # Provider-agnostic stack logic
│   │   ├── stack.rs         # Stack detection and manipulation
│   │   ├── git.rs           # Git operations wrapper
│   │   └── metadata.rs      # Metadata storage
│   ├── providers/           # Provider implementations
│   │   ├── mod.rs           # Provider trait definition
│   │   ├── gitlab.rs        # GitLab provider
│   │   ├── github.rs        # GitHub provider
│   │   └── mock.rs          # Mock for testing
│   ├── config.rs            # Configuration management
│   └── error.rs             # Error types
├── tests/                   # Integration tests
└── Cargo.toml
```

### Module Guidelines

- **`core/`**: Must be 100% provider-agnostic
- **`providers/`**: Each provider is self-contained
- **`cli/`**: Command definitions, delegates to core logic
- **`error.rs`**: Use `thiserror` or similar for ergonomic errors

---

## Documentation Guidelines

### Code Documentation

All documentation should live in the code itself, not in separate README files:

- ✅ **DO** use module-level doc comments (`//!`) to document modules comprehensively
- ✅ **DO** use doc comments (`///`) for public APIs (functions, structs, traits, etc.)
- ✅ **DO** include usage examples in doc comments using `rust,ignore` code blocks
- ✅ **DO** document design decisions and non-obvious behavior in comments
- ❌ **DON'T** create separate README.md files in subdirectories
- ❌ **DON'T** duplicate information between code docs and external files

### Documentation Priority

Information should be documented in this order of preference:

1. **Module-level docs** (`//!` at top of `.rs` files) - Architecture, design principles, usage examples
2. **Item-level docs** (`///` on functions, structs, etc.) - API documentation, parameters, return values
3. **Inline comments** (`//`) - Implementation details, why not what
4. **Top-level README.md** - Project overview, installation, quick start
5. **AGENT.md** - AI assistant guidelines, architectural decisions, development practices

**Exception**: Top-level `README.md` and `AGENT.md` are allowed for project-wide documentation.

### Example

```rust
//! Stack detection and manipulation
//!
//! This module provides the core stack logic for detecting and manipulating
//! linear branch stacks. All operations are provider-agnostic.
//!
//! # Stack Structure
//!
//! A stack is a linear sequence of branches where each branch has exactly
//! one parent branch, forming a chain back to a base branch (usually `main`).
//!
//! # Example
//!
//! ```rust,ignore
//! let stack = Stack::detect_from_branch("feature-part-3")?;
//! for branch in stack.branches() {
//!     println!("Branch: {}", branch.name);
//! }
//! ```

/// Detect a stack starting from the given branch
///
/// Walks the Git history from `branch` back to the base branch,
/// validating that the history is linear (no merge commits).
///
/// # Errors
///
/// Returns an error if:
/// - The branch doesn't exist
/// - The history contains merge commits
/// - The stack is empty (no commits between branch and base)
pub fn detect_stack(branch: &str) -> Result<Stack> {
    // Implementation here
}
```

---

## Coding Conventions

### Rust Style

- Follow standard Rust conventions (`rustfmt`, `clippy`)
- Use `Result<T, Error>` for all fallible operations
- Prefer `&str` for function parameters, `String` for owned data
- Use `thiserror` for all error types (consistent, structured errors)
- Never use `anyhow` - always define proper error types

### Error Handling

```rust
// Good: Specific, actionable error messages using thiserror
#[derive(Debug, Error)]
pub enum GitLabError {
    #[error("Authentication failed: Invalid or expired token")]
    AuthenticationFailed,
    
    #[error("Token is missing required scope: {required}")]
    MissingScope { required: String },
}

return Err(GitLabError::AuthenticationFailed);

// Bad: Generic error strings
return Err(Error::Generic("Something went wrong".to_string()));
```

### Naming Conventions

- **Reviews not PRs/MRs**: Use "review" in core code, provider-specific in providers
- **Branches**: Always `Branch` type, never raw strings in core logic
- **Stack**: Use `Stack` struct, never ad-hoc Vec<Branch>

### Git Operations

**Architecture**: basalt uses **gitoxide** (`gix` crate) for all git operations.

**Why gitoxide?**
- ✅ Pure Rust - better performance, type safety, single binary
- ✅ No subprocess overhead
- ✅ Better error handling with Rust Result types
- ✅ More control over git operations
- ✅ Actively maintained and production-ready (used by cargo, GitLab)

**Exception**: A few operations use git CLI when it's genuinely simpler:
- `git status --porcelain` for uncommitted changes check (cleaner than low-level API)
- Complex interactive operations if/when needed in the future

**Guidelines**:
- Wrap all git operations in the `core::git` module
- Use gitoxide (`gix`) for: reading refs, examining history, reading config, branch operations
- Handle errors explicitly with proper context
- Provide clear error messages that help users fix issues

### Provider API Calls

```rust
// Good: Direct API call with proper error handling using thiserror
let response = client
    .post(&format!("{}/projects/{}/merge_requests", api_url, project_id))
    .header("PRIVATE-TOKEN", &token)
    .json(&create_params)
    .send()?;

if !response.status().is_success() {
    return Err(GitLabError::ApiError {
        status: response.status().as_u16(),
        message: response.text().unwrap_or_default(),
    });
}

let mr: MergeRequest = response.json()?;

// Bad: Using anyhow instead of thiserror
let response = client.post(&url).send().context("Failed to create MR")?;
```

---

## Testing Requirements

### Testing Strategy

basalt uses a **two-tier testing approach**:

1. **Native Tests** (Fast) - Run with `cargo test` for quick iteration during development
2. **Docker Tests** (Comprehensive) - Run with `./scripts/test-docker.sh` to test environment scenarios

**When to use each:**

| Scenario | Use Native Tests | Use Docker Tests |
|----------|-----------------|------------------|
| Daily development | ✅ Yes | ❌ No |
| Quick iteration | ✅ Yes | ❌ No |
| Before commit | ✅ Yes | ⚠️ Optional |
| Before push | ✅ Yes | ✅ Yes |
| Before PR | ✅ Yes | ✅ Yes (matrix) |
| Debugging env issues | ❌ No | ✅ Yes |
| Testing missing deps | ❌ No | ✅ Yes |

### Unit Tests

- Test all core stack logic with mock providers
- Test git operations with temporary repositories
- Test metadata serialization/deserialization
- Test error conditions explicitly
- Run with `cargo test --lib`

### Integration Tests

- Test full workflows with real git repositories
- Use temporary git repos (via `tempfile` crate)
- Mock HTTP responses for provider APIs (using `wiremock` or similar)
- Test migration scenarios (Charcoal → basalt)
- Test cross-platform behavior
- Test authentication fallback chains
- Run with `cargo test --test '*'`

### Docker Environment Tests

Docker tests verify basalt behavior in different environment configurations:

**Scenarios:**
1. **Full environment** (`test`) - All dependencies installed (default)
2. **No git** (`test-no-git`) - Verifies error handling when git is missing
3. **No network** (`test-no-network`) - Tests offline behavior
4. **No auth** (`test-no-auth`) - Tests authentication fallback mechanisms

**Run Docker tests:**

```bash
# Quick commands
./scripts/test-docker.sh all              # Full environment (default)
./scripts/test-docker.sh no-git           # Test missing git scenario
./scripts/test-docker.sh no-network       # Test offline behavior
./scripts/test-docker.sh matrix           # Run all scenarios

# Using cargo-make
cargo make test-docker                    # Full environment
cargo make test-docker-matrix             # All scenarios
cargo make test-docker-shell              # Interactive debugging
```

**Why Docker tests matter:**
- Verify graceful degradation when git is missing
- Test authentication fallback mechanisms
- Ensure error messages are helpful and actionable
- Test in clean, reproducible environments
- Catch environment-specific issues before CI

### Test Organization

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_stack_detection_linear() {
        // Create temp repo
        // Set up linear branch history
        // Assert stack is detected correctly
    }
    
    #[test]
    fn test_stack_detection_rejects_merge_commits() {
        // Create temp repo with merge commit
        // Assert stack detection fails with clear error
    }
}
```

### CI Testing

GitHub Actions CI runs:
- **Native tests** on every push (Linux, macOS, Windows) - Fast feedback
- **Docker tests** on PRs and main branch - Comprehensive validation
- **Clippy** and **rustfmt** checks on all platforms
- **Code coverage** reporting (tarpaulin)

See `.github/workflows/ci.yml` for the full configuration.

---

## Command Design

### Command Naming

- Keep commands simple and Unix-like
- Prefer full words over abbreviations in flags
- Use consistent flag names across commands

### Examples

```bash
# Good
bt init
bt submit
bt restack
bt status --verbose

# Bad (avoid)
bt initialize-repository
bt s  # Too terse for command names
bt update-stack  # 'restack' is clearer in this domain
```

### Output Design

- Default: Concise, human-readable
- `--json`: Machine-parseable output
- `--verbose`: Detailed operation logs
- Colors: Use, but respect `NO_COLOR` environment variable
- Progress: Show for multi-step operations

---

## Configuration

### Config File Format

Use TOML (`.basalt.toml` or `.basalt/config.toml`):

```toml
[basalt]
version = "1"

[repository]
base_branch = "main"
provider = "gitlab"  # auto-detected, can override

[providers.gitlab]
draft_by_default = true
mr_title_prefix = "[Stack]"

[providers.github]
draft_by_default = true
pr_title_prefix = "[Stack]"
```

### Config Priority

1. Command-line flags
2. Repository config (`.basalt.toml`)
3. User config (`~/.config/basalt/config.toml`)
4. Defaults

---

## Metadata Storage

### Location

Metadata is stored in `.git/basalt/` directory:
- **Never committed** (`.git/` is always ignored by git)
- **Co-located with git metadata** (clean repository root)
- **Automatically cleaned** if `.git/` is removed
- **Precedent**: Similar to how git-lfs uses `.git/lfs/`

### Format

Store in `.git/basalt/metadata.yml`:

```yaml
version: "1"
provider: gitlab
base_branch: main

branches:
  feature-part-1:
    review_id: "!123"
    review_url: "https://gitlab.com/..."
    parent: main
    created_at: "2024-01-01T00:00:00Z"
  
  feature-part-2:
    review_id: "!124"
    review_url: "https://gitlab.com/..."
    parent: feature-part-1
    created_at: "2024-01-01T00:00:00Z"
```

### Metadata Rules

- Always validate version on load
- Provide migration path for version changes
- Never trust metadata without validating against git
- Handle missing/corrupted metadata gracefully
- Create `.git/basalt/` directory if it doesn't exist
- Fail gracefully if `.git/` directory doesn't exist (not a git repo)

---

## Provider Implementation Guidelines

### GitLab Provider (MVP)

- Use GitLab REST API directly via `reqwest`
- **Supports both gitlab.com and self-hosted instances**
- **Caches instance URL and project path in metadata**
  - Only extracts from git remote when missing or invalid
  - Stored in `.git/basalt/metadata.yml` for fast access
- API endpoints to implement:
  - `GET /user` — Verify authentication
  - `GET /personal_access_tokens/self` — Verify token scopes
  - `POST /projects/:id/merge_requests` — Create MR
  - `PUT /projects/:id/merge_requests/:mr_iid` — Update MR
  - `GET /projects/:id/merge_requests/:mr_iid` — Get MR details
- **Authentication Priority**:
  1. Use stored token from metadata (`.git/basalt/metadata.yml`)
  2. If no stored token or authentication fails (expired/revoked):
     - Try reading from glab CLI config (`~/.config/glab-cli/config.yml`)
     - Try git credential helper (`git credential fill`)
     - Offer CLI auth: if glab available, ask user to choose:
       - Option 1: Run `glab auth login` (interactive)
       - Option 2: Enter PAT manually
     - If no glab, prompt for PAT directly
  3. Verify token has required scopes (must have 'api' scope)
  4. Store successful token in metadata for future use
- Parse JSON responses using `serde_json`
- Parse YAML metadata files using `serde_yaml`
- Store MR IID (internal ID) and auth token in metadata
- Use `thiserror` for all error types (not `anyhow`)

### GitHub Provider (Post-MVP)

- Use GitHub REST API directly via `reqwest`
- Leverage Charcoal knowledge for workflow patterns
- API endpoints to implement:
  - `GET /user` — Verify authentication
  - `POST /repos/:owner/:repo/pulls` — Create PR
  - `PATCH /repos/:owner/:repo/pulls/:number` — Update PR
  - `GET /repos/:owner/:repo/pulls/:number` — Get PR details
- Authentication:
  - Read gh's token from `~/.config/gh/hosts.yml` if available
  - Fallback to git credential helper
  - Fallback to prompting user for PAT
- Store PR number in metadata

### Provider Detection

Auto-detect from git remote:

```rust
// Pseudo-code
fn detect_provider(repo: &Repository) -> Result<ProviderType> {
    let remote_url = repo.get_remote_url("origin")?;
    
    if remote_url.contains("gitlab.com") || remote_url.contains("gitlab") {
        Ok(ProviderType::GitLab)
    } else if remote_url.contains("github.com") {
        Ok(ProviderType::GitHub)
    } else {
        Err(Error::UnknownProvider(remote_url))
    }
}
```

---

## Migration from Charcoal

### Considerations

- Charcoal stores metadata differently (location TBD after code review)
- Charcoal is GitHub-only
- Users may have active stacks

### Migration Strategy

1. Detect existing Charcoal metadata
2. Offer migration command: `bt migrate-from-charcoal`
3. Convert metadata format
4. Preserve review IDs and URLs
5. Verify migration success

### Don't Break Workflows

- Allow running both tools side-by-side during transition
- Provide clear migration documentation
- Test migration thoroughly with real Charcoal repos

---

## Common Pitfalls to Avoid

### ❌ Don't Do This

1. **Don't spawn git CLI unnecessarily** — Use gitoxide (`gix`) for git operations
2. **Don't hardcode GitHub/GitLab in core** — Use provider abstraction
3. **Don't assume linear history** — Validate and error clearly
4. **Don't force-push without confirmation** — Ask or require --force flag
5. **Don't make network requests in core** — Delegate to providers
6. **Don't use print!() directly** — Use proper logging/output framework
7. **Don't panic!()** — Return Result with clear error
8. **Don't hardcode API tokens** — Store in metadata, fetch only when needed
9. **Don't spawn provider CLIs for API operations** — Use direct REST API calls
10. **Don't use anyhow** — Always use thiserror for proper error types
11. **Don't fetch auth tokens on every operation** — Use stored token, only refresh if invalid

### ✅ Do This

1. **Use provider trait** — All provider operations go through abstraction
2. **Validate git state** — Check for uncommitted changes, conflicts, etc.
3. **Parse structured responses** — JSON from provider APIs
4. **Provide clear errors** — Include context and next steps using thiserror
5. **Test with temp repos** — All git operations in tests
6. **Document assumptions** — Comment non-obvious design decisions
7. **Mock HTTP responses** — Use wiremock or similar for testing
8. **Handle auth gracefully** — Store token, try multiple sources only when needed
9. **Verify token scopes** — Check token has required permissions before use
10. **Offer CLI auth option** — If available, let user choose between CLI login or PAT
11. **Cache provider metadata** — Store base_url and project_path, only extract from remote when needed
12. **Use conditional compilation** — Disable test-only features in release builds

---

## Documentation Standards

### Code Comments

- Document **why**, not **what**
- Explain non-obvious design decisions
- Link to relevant issues or discussions
- Use `///` for public API docs
- Use `//` for implementation notes

### Commit Messages

- Use conventional commits format
- Reference issues when applicable
- Keep first line under 72 characters

Example:
```
feat(gitlab): implement MR creation via glab CLI

- Use glab mr create --json for structured output
- Parse MR ID and URL from response
- Store metadata in .basalt/metadata.json

Closes #123
```

---

## Key Design Decisions

### Why Rust?

- Performance for large repos
- Single binary distribution (including git operations via gitoxide)
- Strong type safety for complex stack logic
- Great CLI ecosystem (clap, etc.)
- Native git integration with gitoxide (no subprocess overhead)

### Why direct API vs CLI delegation?

- **Full control**: Direct control over API interactions and error handling
- **Performance**: No subprocess overhead
- **Transparency**: Easier to debug and understand API calls
- **Minimal dependencies**: No external CLI tools required
- **Better errors**: Structured error responses from APIs
- **Smart auth**: Reuse existing tokens or prompt for PAT when needed

### Why gitoxide?

- **Performance**: No subprocess overhead
- **Type safety**: Rust types for git objects, refs, commits, etc.
- **Single binary**: No external git dependency
- **Better errors**: Structured error types instead of parsing stderr
- **Production ready**: Used by major projects (cargo, GitLab)

### Why local-first?

- Works offline for read operations
- Fast, no network latency (even faster with gitoxide)
- Git is already local
- Users own their data

### Why provider abstraction?

- Multi-provider from day one
- Easier to add new providers
- Testable with mock providers
- Clear separation of concerns

---

## When in Doubt

1. **Check existing code** — Maintain consistency
2. **Prefer explicit** — Over clever or implicit
3. **Test it** — Write tests for new behavior
4. **Ask questions** — In comments or issues
5. **Keep it simple** — Defer complexity until needed

---

## Useful Commands During Development

```bash
# Format code
cargo fmt

# Lint
cargo clippy

# Run tests
cargo test

# Run specific test
cargo test test_name

# Build release binary
cargo build --release

# Check without building
cargo check
```

---

## Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [GitLab CLI (glab)](https://gitlab.com/gitlab-org/cli)
- [GitHub CLI (gh)](https://cli.github.com/)
- [Charcoal codebase](./README.charcoal.md) — Reference for workflow patterns

---

**Remember**: basalt is local-first, provider-agnostic, and explicit. When adding features, always ask: "Does this work for all providers?" and "Is this behavior predictable and transparent?"

---

## Project Setup (Completed)

The following foundational work has been completed:

- ✅ Rust project initialized with `cargo init --name bt`
- ✅ CLI framework configured using `clap` with derive macros
- ✅ Basic command structure implemented (init, submit, restack, status)
- ✅ Dependencies added: clap, anyhow, thiserror, serde, serde_json, toml, serde_yaml
- ✅ GitHub Actions CI workflow configured for:
  - Multi-platform testing (Linux, macOS, Windows)
  - Code formatting checks (rustfmt)
  - Linting (clippy)
  - Code coverage (tarpaulin)
  - Release binary builds
- ✅ .gitignore updated for Rust artifacts
- ✅ Basic error handling structure in place (`src/error.rs`)
- ✅ Metadata location decided: `.git/basalt/` (never committed, clean workspace)
- ✅ **Provider Abstraction Layer complete** (`src/providers/`):
  - Provider trait for API-based operations
  - Review metadata structures (Review, ReviewState, CreateReviewParams, UpdateReviewParams)
  - Authentication interface
  - Mock provider for testing (fully functional)
  - Provider detection logic (from remote URLs and string parsing)
  - GitLab and GitHub provider stubs with authentication checks
- ✅ **Environment & Dependency Checks complete** (`src/core/environment.rs`):
  - Git repository detection and validation
  - Basalt directory management (`.git/basalt/`)
  - Initialization status checks
  - Working directory state checks (uncommitted changes, rebase in progress)
  - Provider authentication checks (delegates to provider trait)
  - Comprehensive integration tests with temporary git repositories
  - Clear, actionable error messages for all failure cases

**Next steps**: Complete Stack Detection & Validation (Section 5 of MVP tasks in README.md)

---

## Recently Completed Work (Continued)

### GitLab Provider Implementation (Section 2) - Enhanced Authentication & Self-Hosted Support

The GitLab provider now uses direct REST API access with intelligent authentication management and full support for self-hosted GitLab instances:

- ✅ **GitLab REST API Client** (`src/providers/gitlab_api.rs`):
  - Lightweight wrapper around GitLab REST API v4
  - **Smart authentication priority**:
    1. Use stored token (from metadata) if available
    2. Only query external sources if token missing/invalid:
       - Read from glab CLI config (`~/.config/glab-cli/config.yml`)
       - Query git credential helper
       - Offer CLI auth (run `glab auth login`) if glab is available
       - Prompt for PAT as fallback
  - **Token scope verification** via `GET /personal_access_tokens/self`
    - Validates token has required 'api' scope
    - Checks token is active (not expired/revoked)
  - MR creation via `POST /projects/:id/merge_requests`
  - MR update via `PUT /projects/:id/merge_requests/:mr_iid`
  - MR retrieval via `GET /projects/:id/merge_requests/:mr_iid`
  - Authentication verification via `GET /user`
  - **Uses thiserror for all errors** (not anyhow)
  - Clear, actionable error messages with structured error types
  
- ✅ **GitLab Provider** (`src/providers/gitlab.rs`):
  - Direct REST API integration (no CLI dependency)
  - Stores authentication state
  - Methods to get/set auth token for metadata persistence
  - Project path configuration support
  - Full implementation of Provider trait
  - Converts GitLab MR responses to Review structs
  - Comprehensive unit tests
  
- ✅ **Metadata with Auth Token Storage** (`src/core/metadata.rs`):
  - Added `auth_token: Option<String>` field
  - Tokens stored in `.git/basalt/metadata.yml` (never committed)
  - Only fetched from external sources when missing or invalid
  - Reduces authentication prompts and improves UX
  
- ✅ **Init Command with Authentication** (`src/cli/init.rs`):
  - Authenticates during `bt init`
  - Stores token in metadata for future use
  - Added `--skip-auth` flag for testing (conditionally compiled - debug builds only)
  - Clear authentication flow with user choice (CLI vs PAT)
  - **Caches base_url and project_path in metadata**
  - Only extracts from git remote when missing or on first init
  - Works with both HTTPS and SSH remote URLs
  - Allows init without remote when provider is explicitly specified
  
- ✅ **Provider Trait Updates** (`src/providers/mod.rs`):
  - Removed CLI-related methods (`check_cli_available`, `cli_name`, `install_url`, `auth_command`)
  - Changed all methods to take `&mut self` for authentication state
  - Added `authenticate()` method to Provider trait
  - Updated Review struct to use `Option<String>` for description
  - Updated documentation to reflect REST API approach
  
- ✅ **Mock Provider Updates** (`src/providers/mock.rs`):
  - Updated to match new Provider trait signature
  - Removed CLI availability simulation
  - All tests updated and passing
  
- ✅ **Environment Checks Updates** (`src/core/environment.rs`):
  - Removed CLI availability checks
  - Updated to focus on authentication only
  
- ✅ **Provider URL Extraction** (`src/providers/mod.rs`):
  - `extract_base_url()` - Extracts base URL from git remote (e.g., "https://gitlab.com")
  - `extract_project_path()` - Extracts project path (e.g., "owner/repo")
  - Supports both HTTPS and SSH remote URL formats
  - Supports self-hosted GitLab instances (e.g., "https://gitlab.example.com")
  - Supports nested project paths (GitLab groups/subgroups)
  
- ✅ **Metadata Caching** (`src/core/metadata.rs`):
  - Added `base_url: Option<String>` field - cached provider instance URL
  - Added `project_path: Option<String>` field - cached project path
  - `get_base_url()` helper - returns cached value or extracts from remote
  - `get_project_path()` helper - returns cached value or extracts from remote
  - Reduces git remote parsing - only done on first init or when cache is invalid
  
- ✅ **Documentation Updates**:
  - AGENT.md updated throughout to reflect REST API approach
  - README.md updated to remove CLI dependencies
  - All examples updated to show direct API usage

**Dependencies added**:
- `reqwest` (v0.12) with `json` and `blocking` features - HTTP client
- `tokio` (v1.0) with runtime features - Async runtime (for future async support)
- `urlencoding` (v2.1) - URL encoding for API paths
- `dirs` (v5.0) - Cross-platform home directory detection

**Key architectural decisions**:
- **thiserror over anyhow**: Consistent structured errors throughout codebase
- **Store tokens in metadata**: Reduces authentication friction, only fetch when needed
- **Verify token scopes**: Catch permission issues early with clear error messages
- **Offer CLI auth option**: If glab available, let user choose interactive login or PAT
- **Token-first approach**: Try stored token first, only fall back to external sources if invalid
- **Self-hosted support**: Extract instance URL from git remote, cache in metadata
- **Metadata caching**: Store base_url and project_path, only extract when missing
- **Skip-auth for testing only**: Conditionally compiled - completely absent in release builds
- **Deferred extraction**: Allow init without remote, extract URL/path later when needed

**All 55 tests passing** ✅ (36 unit + 8 environment + 11 init)

---

## Meta-Guidelines for AI Assistants

### Maintaining This Document

**You MUST update this AGENT.md file when:**

- New architectural decisions are made
- New tools or dependencies are introduced
- Coding conventions change or are clarified
- New provider support is added
- Testing strategies evolve
- Common pitfalls are discovered
- Design decisions are revised

**How to update:**
1. Add new sections for new topics
2. Update existing sections when rules change
3. Remove obsolete information
4. Keep examples current with the codebase
5. Update "Last updated" timestamp

### Tracking Progress in README.md

**You MUST check off TODO items in README.md when:**

- A task is fully completed and tested
- The implementation meets the requirements
- Tests pass for the feature
- Documentation is updated accordingly

**How to check off items:**
```markdown
- [x] Completed task description
```

**When NOT to check off:**
- Partial implementations
- Untested code
- Features missing documentation
- Work-in-progress changes

**Be conservative** — It's better to leave something unchecked than to prematurely mark it complete.

---

*Last updated: GitLab Provider Implementation with Enhanced Authentication complete (Section 2 of MVP)*

---

## Recently Completed Work

### Repository Initialization (`bt init`) - Section 4

The `bt init` command is now fully implemented and tested:

- ✅ **Git operations module** (`src/core/git.rs`):
  - Current branch detection
  - Remote URL retrieval
  - Default branch detection (multiple strategies)
  - Branch existence checks
  - Upstream tracking queries
  
- ✅ **Metadata management** (`src/core/metadata.rs`):
  - YAML-based metadata format (`.git/basalt/metadata.yml`)
  - Version validation and migration support
  - Branch metadata tracking (review IDs, parent branches, timestamps)
  - Full serialization/deserialization support
  
- ✅ **Init command** (`src/cli/init.rs`):
  - Auto-detection of provider from git remote URL
  - Support for SSH and HTTPS remote formats
  - Auto-detection of default base branch
  - Provider override via `--provider` flag
  - Base branch override via `--base-branch` flag
  - Checks for already-initialized repositories
  - Clear error messages for all failure cases
  
- ✅ **Comprehensive testing**:
  - Unit tests for all new modules
  - 11 integration tests covering all init scenarios
  - Tests for error conditions (no remote, invalid provider, etc.)
  - All 47 tests passing

**Dependencies added**: 
- `chrono` for timestamp handling in metadata
- `gix` (gitoxide) for native Rust git operations

**Architectural decision**: Using gitoxide (`gix`) for all git operations provides better performance, type safety, and eliminates subprocess overhead while maintaining a single binary distribution.

---