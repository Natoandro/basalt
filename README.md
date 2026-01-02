# basalt

> A multi-provider CLI for managing stacked changes

**basalt** (`bt`) is a Rust-based command-line tool that brings stacked workflow support to multiple Git hosting providers, starting with GitLab.

---

## 🚧 Status

This project is currently in the **planning phase**. The repository contains [Charcoal](./README.charcoal.md), a TypeScript-based stacked PR tool for GitHub, which will serve as the foundation for basalt's design.

basalt will be a ground-up **Rust rewrite** with a **provider-agnostic architecture**.

---

## Why basalt?

**basalt** (`bt`) extends the stacked workflow concept beyond GitHub:

- 🔀 **Multi-provider support** — GitLab, GitHub, and potentially others
- ⚡ **Fast** — Written in Rust for performance
- 📦 **Portable** — Single binary, no runtime dependencies
- 🎯 **Local-first** — Stack state lives in Git and local metadata
- 🔌 **CLI-native** — Delegates to provider CLIs (`glab`, `gh`, etc.)

---

## What are stacked changes?

Stacked changes is a workflow where you:

1. Break large features into small, reviewable chunks
2. Create a **linear stack** of Git branches
3. Each branch maps to one code review (MR/PR)
4. Submit and update the entire stack efficiently
5. Land changes sequentially from bottom to top

This approach leads to:
- Faster code reviews
- Easier debugging and rollbacks
- Better separation of concerns
- Reduced merge conflicts

---

## Planned Features

### MVP (GitLab Provider)
- ✅ Initialize repository (`bt init`)
- ✅ Submit entire stack as draft MRs (`bt submit`)
- ✅ Rebase/restack branches (`bt restack`)
- ✅ Stack validation and detection
- ✅ Local metadata storage

### Post-MVP
- 🎯 **GitHub provider** (leveraging Charcoal knowledge)
- 🎯 **Additional providers** (Gitea, Bitbucket, Azure DevOps)
- 🎯 Enhanced stack operations (split, merge, reorder)
- 🎯 CI/pipeline integration
- 🎯 Smart merge and cleanup workflows
- 🎯 Rich status and visualization

---

## Architecture

### Provider Abstraction

basalt uses a clean provider interface that abstracts:
- **Authentication** — Delegated to provider CLIs
- **Review creation/updates** — MRs, PRs, etc.
- **Metadata storage** — Branch → review ID mapping (stored in `.git/basalt/metadata.yml`)
- **Provider detection** — Auto-detect from Git remotes

### Metadata Location

basalt stores its metadata in **`.git/basalt/metadata.yml`** (inside the `.git/` directory):
- ✅ **Never accidentally committed** — `.git/` is always ignored by git
- ✅ **Clean workspace** — Doesn't clutter your repository root
- ✅ **Auto-cleanup** — Removed if `.git/` is deleted
- ✅ **Following precedent** — Similar to how git-lfs uses `.git/lfs/`
- ✅ **Cached metadata** — Stores provider URL, project path, and auth token for fast access

Team-wide configuration can optionally be stored in `.basalt.toml` in the repository root (this can be committed).

### Dependencies

- **No external git dependency** — Uses gitoxide (pure Rust git implementation)
- **No provider CLI required** — Direct REST API integration
- Network access to provider APIs (GitLab, GitHub, etc.)
- **Supports self-hosted instances** — Extracts and caches instance URL from git remote

---

## Development Roadmap

### MVP Scope (GitLab Provider)

**MVP success criteria:**

> From the top branch of a stack, run one command and have all stacked branches pushed and represented as draft GitLab MRs, using a Rust implementation with a pluggable provider architecture.

Anything not strictly required to reach this outcome is **deferred** for post-MVP.

### MVP Tasks (Rust Rewrite + GitLab Provider)

#### 1. Provider Abstraction Layer

- [x] Define `Provider` trait
- [x] Define review metadata structures
- [x] Define authentication interface
- [x] Create mock provider for testing
- [x] Implement provider detection logic

#### 2. GitLab Provider Implementation

- [x] Implement GitLab REST API client
- [x] Implement authentication (glab token, git credential, or PAT prompt)
- [x] Implement MR creation via POST `/projects/:id/merge_requests`
- [x] Implement MR update via PUT `/projects/:id/merge_requests/:mr_iid`
- [x] Implement MR retrieval via GET `/projects/:id/merge_requests/:mr_iid`
- [x] Parse JSON responses from GitLab API
- [x] Support self-hosted GitLab instances (extracts and caches URL from git remote)
- [x] Cache provider base URL and project path in metadata
- [x] Verify token scopes (requires 'api' scope)

#### 3. Environment & Dependency Checks

- [x] Verify execution inside a Git repository
- [x] Verify provider authentication (API token)
- [x] Provide clear, actionable error messages

#### 4. Repository Initialization (`bt init`)

- [x] Create config directory (`.git/basalt/`)
- [x] Auto-detect Git provider from remote
- [x] Detect and store default base branch
- [x] Persist minimal config (YAML)
- [x] Support provider override flag

#### 5. Stack Detection & Validation

- [ ] Detect current branch
- [ ] Walk linear ancestry up to base branch
- [ ] Validate stack is linear (no merges)
- [ ] Build in-memory stack representation
- [ ] Abort on ambiguous or unsupported graphs

#### 6. Stack Submission (`bt submit`)

- [ ] Enumerate stack bottom → top
- [ ] Checkout each branch
- [ ] Push branch (create upstream if needed)
- [ ] Create review via provider
- [ ] Set appropriate metadata (draft status, etc.)
- [ ] Update review if it already exists
- [ ] Print submission summary

#### 7. Restacking (`bt restack`)

- [ ] Rebase first stack branch onto base
- [ ] Rebase each subsequent branch onto its parent
- [ ] Surface conflicts directly from git
- [ ] Force-push rebased branches
- [ ] Handle rebase failures gracefully

#### 8. Metadata Storage

- [ ] Design metadata format (per-provider)
- [ ] Store branch → review ID mapping
- [ ] Persist provider information
- [ ] Load metadata on subsequent runs
- [ ] Handle metadata migration/versioning

#### 9. Output & UX

- [ ] Deterministic, readable CLI output
- [ ] Success and failure indicators
- [ ] Display review URLs after submission
- [ ] Progress indicators for multi-step operations
- [ ] Colored output (optional, respects NO_COLOR)

#### 10. Documentation

- [ ] Write comprehensive README
- [ ] Installation instructions (cargo install, binaries)
- [ ] Provider-specific setup guides
- [ ] End-to-end workflow examples
- [ ] Migration guide from Charcoal

#### 11. Testing

- [ ] Unit tests for stack logic
- [ ] Provider trait tests with mock provider
- [ ] Integration tests with real git repos
- [ ] CI pipeline setup

### Post-MVP Features

#### A. GitHub Provider

- [ ] Implement GitHub provider using `gh` CLI
- [ ] Feature parity with current Charcoal
- [ ] Migration tooling from Charcoal metadata
- [ ] GitHub-specific optimizations

#### B. Stack Intelligence

- [ ] Automatic stack inference
- [ ] Support non-linear branch graphs
- [ ] Stack reordering
- [ ] Partial stack submission
- [ ] Stack visualization

#### C. Review Enhancements

- [ ] Encode stack relationships in review descriptions
- [ ] Provider-native dependency features (GitLab MR dependencies)
- [ ] Automatic title templating
- [ ] Parent / child review navigation links
- [ ] Custom description templates

#### D. CI & Merge Awareness

- [ ] Inspect CI/pipeline status via provider CLI
- [ ] Merge readiness checks
- [ ] Merge train awareness (GitLab)
- [ ] Merge queue integration (GitHub)
- [ ] Optional CI gating on submission

#### E. Merge & Cleanup Workflow

- [ ] Sequential merge command
- [ ] Auto-rebase remaining stack after merge
- [ ] Automatic remote branch deletion
- [ ] Local stack cleanup
- [ ] Stack archival

#### F. UX Improvements

- [ ] `bt status` command with rich output
- [ ] Interactive prompts for ambiguous operations
- [ ] Dry-run mode (`--dry-run` flag)
- [ ] Shell completion scripts (bash, zsh, fish)
- [ ] Man pages

#### G. Configuration & Policy

- [ ] Configurable base branch per stack
- [ ] Draft vs ready review policy
- [ ] Force-push safety controls
- [ ] Team-wide configuration support (`.basalt.toml` in repo root)
- [ ] Per-provider configuration overrides

#### H. Additional Providers

- [ ] Gitea/Forgejo provider
- [ ] Bitbucket provider
- [ ] Azure DevOps provider
- [ ] Generic git provider (local-only mode)

#### I. Advanced Stack Operations

- [ ] Branch insertion into existing stacks
- [ ] Branch removal from stacks
- [ ] Stack splitting
- [ ] Stack merging
- [ ] Cherry-pick across stacks

#### J. Performance & Optimization

- [ ] Parallel review operations where possible
- [ ] Caching of provider CLI outputs
- [ ] Incremental stack validation
- [ ] Optimistic UI updates

#### K. Distribution & Installation

- [ ] Prebuilt binaries for major platforms
- [ ] Homebrew formula
- [ ] APT/RPM packages
- [ ] cargo-binstall support
- [ ] Windows installer
- [ ] Docker image

#### L. Observability & Debugging

- [ ] Verbose logging mode (`-v`, `-vv`, `-vvv`)
- [ ] Debug output for provider interactions
- [ ] Git operation tracing
- [ ] Performance profiling hooks

---

## Current State: Charcoal

This repository currently contains **Charcoal**, a fully functional TypeScript-based stacked PR tool for GitHub. See [README.charcoal.md](./README.charcoal.md) for details.

Charcoal users will be able to migrate to basalt's GitHub provider when available.

---

## Design Principles

- **Explicit over implicit** — Predictable, transparent behavior
- **Fail fast and loudly** — Clear errors, no silent failures
- **Leverage existing tools** — Let Git do what it does well, use APIs for the rest
- **Local-first** — Never require network for read-only operations
- **Provider parity** — Core operations work across all providers
- **Direct API access** — No external CLI dependencies, full control over interactions
- **Self-hosted support** — Works with gitlab.com and self-hosted GitLab instances
- **Metadata caching** — Stores provider URL, project path, and credentials for fast access

---

## Non-goals

- Web UI or desktop application
- Replacing provider-native features (CI, merge queues)
- Background services or daemons
- Complex OAuth flows (use simpler PAT authentication)

---

## Installation

**Not yet available** — basalt is currently in planning.

Once released:
```bash
# Via cargo
cargo install basalt

# Via Homebrew (planned)
brew install basalt

# Via prebuilt binaries
# Download from GitHub releases
```

---

## Quick Start (Planned)

```bash
# Initialize in your Git repository
bt init

# Create a stack of branches
git checkout -b feature-part-1
# ... make changes ...
git commit -m "Part 1"

git checkout -b feature-part-2
# ... make changes ...
git commit -m "Part 2"

# Submit entire stack
bt submit

# Restack after changes
bt restack

# Check stack status
bt status
```

---

## Development

### Setup

#### Initial Setup

**Install cargo-make** (recommended for development):

```bash
cargo install cargo-make
```

This provides convenient tasks for testing, formatting, linting, and more:

```bash
cargo make          # Show all available tasks
cargo make ci       # Run all CI checks
cargo make test     # Run all tests
```

#### Git Hooks

Install pre-commit hooks to catch formatting and linting issues before committing:

```bash
# Lightweight (recommended) - runs fmt + clippy
cargo make install-hooks-light

# Full version - also runs tests (slower)
cargo make install-hooks
```

Hooks run automatically on `git commit`. Use `git commit --no-verify` to bypass if needed.

### Running Tests

#### Local Testing

```bash
# Run all tests
cargo test

# Run with verbose output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test '*'

# Run formatting and linting
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

#### Docker Testing (Environment Scenarios)

basalt uses Docker to test different environment configurations, especially for verifying error handling when dependencies are missing:

```bash
# Test all scenarios
./scripts/test-docker.sh matrix          # Run full test matrix
cargo make test-docker-matrix

# Specific scenarios
./scripts/test-docker.sh all             # Full environment (default)
./scripts/test-docker.sh no-git          # Without git (error handling)
./scripts/test-docker.sh no-network      # Without network access
./scripts/test-docker.sh no-auth         # Without authentication

# Development
./scripts/test-docker.sh shell           # Interactive debugging
./scripts/test-docker.sh verbose         # Detailed output
./scripts/test-docker.sh clean           # Cleanup
```

**Using cargo-make** (recommended):

```bash
cargo make test-docker                   # Full environment
cargo make test-docker-no-git            # Test missing git
cargo make test-docker-no-providers      # Test missing CLIs
cargo make test-docker-matrix            # All scenarios
```

**Why Docker?**
- ✅ **Test dependency handling** — Verify behavior when git is missing
- ✅ **Test authentication** — Verify auth fallback mechanisms
- ✅ **Environment isolation** — Clean, reproducible test environments
- ✅ **CI parity** — Match what runs in GitHub Actions
- ✅ **Cross-platform** — Test Linux behavior regardless of dev OS

**When to use:**
- Before submitting PRs (run `cargo make test-docker-matrix`)
- When debugging environment-specific issues
- To verify error messages for missing git or authentication issues

**For daily development**, native `cargo test` is faster. Docker tests are for comprehensive validation.

See [docs/DOCKER_TESTING.md](./docs/DOCKER_TESTING.md) for detailed documentation.

---

## Contributing

Interested in contributing? Here's how you can help:

1. **Design feedback** — Review and comment on architectural decisions
2. **Provider expertise** — Share knowledge about GitLab/GitHub/other platforms
3. **Rust development** — Help implement the MVP once we begin
4. **Testing** — Try basalt with your workflows once available

See [CONTRIBUTING.md](./CONTRIBUTING.md) for more details.

---

## License

See [LICENSE](./LICENSE)

---

## Acknowledgments

basalt builds on ideas from:
- [Graphite](https://graphite.dev) — The original stacked workflow tool
- [Charcoal](./README.charcoal.md) — Open-source continuation of Graphite CLI
- Tools like Phabricator and Critique that pioneered these workflows

---

**Project**: basalt | **Command**: `bt` | **Status**: Planning 🪨