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

### 2. Delegate to Provider CLIs

basalt does **NOT** directly call provider APIs. Instead:

- GitLab → Use `glab` CLI
- GitHub → Use `gh` CLI
- Parse JSON output from these tools
- Handle authentication through their auth systems

**Why:**
- Leverages existing, well-tested tools
- Automatic updates and bug fixes
- No API key management in basalt
- Respects user's existing auth configuration

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
- Use `anyhow` for application errors, `thiserror` for library errors

### Error Handling

```rust
// Good: Specific, actionable error messages
return Err(Error::ProviderCliNotFound {
    provider: "GitLab",
    cli_name: "glab",
    install_url: "https://gitlab.com/gitlab-org/cli",
});

// Bad: Generic error
return Err(Error::Generic("Something went wrong".to_string()));
```

### Naming Conventions

- **Reviews not PRs/MRs**: Use "review" in core code, provider-specific in providers
- **Branches**: Always `Branch` type, never raw strings in core logic
- **Stack**: Use `Stack` struct, never ad-hoc Vec<Branch>

### Git Operations

- Always use `git` CLI, never libgit2 (for now)
- Wrap all git calls in the `core::git` module
- Parse output carefully, handle errors explicitly
- Never assume git command success

### Provider CLI Calls

```rust
// Good: Parse JSON, handle errors
let output = Command::new("glab")
    .args(["mr", "create", "--json"])
    .output()?;
let mr: MergeRequest = serde_json::from_slice(&output.stdout)?;

// Bad: String parsing, fragile
let output = Command::new("glab").args(["mr", "create"]).output()?;
let url = String::from_utf8(output.stdout)?.trim().to_string();
```

---

## Testing Requirements

### Unit Tests

- Test all core stack logic with mock providers
- Test git operations with temporary repositories
- Test metadata serialization/deserialization
- Test error conditions explicitly

### Integration Tests

- Test full workflows with real git repositories
- Mock provider CLI calls (don't require actual glab/gh)
- Test migration scenarios (Charcoal → basalt)
- Test cross-platform behavior

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

- Use `glab` CLI exclusively
- Commands to implement:
  - `glab auth status` — Check authentication
  - `glab mr create --fill --draft --json` — Create MR
  - `glab mr update <id> --json` — Update MR
  - `glab mr view <id> --json` — Get MR details
- Parse JSON output from provider CLIs using `serde_json`
- Parse YAML metadata files using `serde_yaml`
- Store MR ID (with `!` prefix) in metadata

### GitHub Provider (Post-MVP)

- Use `gh` CLI exclusively
- Leverage Charcoal knowledge for workflow patterns
- Commands to implement:
  - `gh auth status` — Check authentication
  - `gh pr create --draft --json` — Create PR
  - `gh pr edit <number> --json` — Update PR
  - `gh pr view <number> --json` — Get PR details
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

1. **Don't use libgit2 directly** — Use git CLI for consistency
2. **Don't hardcode GitHub/GitLab in core** — Use provider abstraction
3. **Don't assume linear history** — Validate and error clearly
4. **Don't force-push without confirmation** — Ask or require --force flag
5. **Don't make network requests in core** — Delegate to providers
6. **Don't use print!() directly** — Use proper logging/output framework
7. **Don't panic!()** — Return Result with clear error

### ✅ Do This

1. **Use provider trait** — All provider operations go through abstraction
2. **Validate git state** — Check for uncommitted changes, conflicts, etc.
3. **Parse structured output** — JSON from provider CLIs
4. **Provide clear errors** — Include context and next steps
5. **Test with temp repos** — All git operations in tests
6. **Document assumptions** — Comment non-obvious design decisions
7. **Check dependencies** — Verify git, glab, gh are available

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
- Single binary distribution
- Strong type safety for complex stack logic
- Great CLI ecosystem (clap, etc.)

### Why CLI delegation vs direct API?

- Simpler authentication
- Automatic updates to provider APIs
- Less code to maintain
- Leverages community tools

### Why local-first?

- Works offline for read operations
- Fast, no network latency
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
- ✅ Dependencies added: clap, anyhow, thiserror, serde, serde_json, toml
- ✅ GitHub Actions CI workflow configured for:
  - Multi-platform testing (Linux, macOS, Windows)
  - Code formatting checks (rustfmt)
  - Linting (clippy)
  - Code coverage (tarpaulin)
  - Release binary builds
- ✅ .gitignore updated for Rust artifacts
- ✅ Basic error handling structure in place
- ✅ Metadata location decided: `.git/basalt/` (never committed, clean workspace)

**Next steps**: Begin implementing the Provider Abstraction Layer (Section 1 of MVP tasks in README.md)

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

*Last updated: Planning phase*