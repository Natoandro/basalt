# Contributing to basalt

Thank you for your interest in contributing to basalt! This guide will help you get started.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Running Tests](#running-tests)
- [Code Quality](#code-quality)
- [Submitting Changes](#submitting-changes)
- [Project Structure](#project-structure)
- [Development Guidelines](#development-guidelines)

## Code of Conduct

Please be respectful and constructive in all interactions. We're building this together!

## Getting Started

Before contributing:

1. **Open an issue first** - Discuss your proposed changes before opening a PR
2. **Check existing issues** - Someone might already be working on it
3. **Read [AGENT.md](./AGENT.md)** - Understand the architecture and design principles
4. **Review the MVP roadmap** - See what's currently being worked on in [README.md](./README.md)

## Development Setup

### Prerequisites

- **Rust 1.85+** - Install from [rustup.rs](https://rustup.rs/)
- **Git** - Required for testing and development
- **cargo-make** (recommended) - Install with `cargo install cargo-make`
- **Docker** (optional) - For Docker-based testing

### Verify Your Environment

```bash
# Check Rust installation
rustc --version
cargo --version

# Verify git is installed
git --version

# Verify environment (if using cargo-make)
cargo make verify-env
```

### Clone and Build

```bash
# Clone the repository
git clone https://github.com/natoandro/basalt.git
cd basalt

# Build the project
cargo build

# Run tests to verify setup
cargo test
```

### Install Development Tools (Optional)

```bash
# Using cargo-make
cargo make install-dev-tools

# Or manually
cargo install cargo-watch    # Watch for changes
cargo install cargo-tarpaulin # Code coverage
cargo install cargo-audit     # Security auditing
```

## Running Tests

### Local Testing

```bash
# Run all tests
cargo test

# Run with verbose output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run only unit tests
cargo test --bin bt

# Run only integration tests
cargo test --test '*'

# Watch mode (with cargo-watch)
cargo watch -x test
```

### Using cargo-make

```bash
# Show all available tasks
cargo make

# Run all tests
cargo make test

# Run CI checks (format, lint, test)
cargo make ci

# Run before committing
cargo make pre-commit
```

### Docker Testing

For reproducible testing in a clean environment:

```bash
# Using the script
./scripts/test-docker.sh

# Using cargo-make
cargo make test-docker

# Verbose output
cargo make test-docker-verbose

# Interactive shell for debugging
cargo make test-docker-shell
```

See [docs/DOCKER_TESTING.md](./docs/DOCKER_TESTING.md) for detailed Docker testing documentation.

## Code Quality

### Formatting

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check

# Using cargo-make
cargo make fmt
cargo make fmt-check
```

### Linting

```bash
# Run clippy
cargo clippy --all-targets -- -D warnings

# Using cargo-make
cargo make clippy
```

### Before Committing

```bash
# Run all checks
cargo make ci

# Or manually
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

## Submitting Changes

### Pull Request Process

1. **Fork the repository** and create a feature branch
2. **Make your changes** following the coding guidelines below
3. **Write tests** for new functionality
4. **Run all checks** with `cargo make ci`
5. **Update documentation** if needed
6. **Write a clear commit message** following conventional commits format
7. **Open a PR** with a description of your changes

### Commit Message Format

Use conventional commits:

```
feat(gitlab): implement MR creation via glab CLI
fix(environment): handle missing .git directory
docs(readme): update installation instructions
test(stack): add test for merge commit detection
```

Types: `feat`, `fix`, `docs`, `test`, `refactor`, `chore`, `ci`

### Pull Request Guidelines

- Reference the issue number in your PR description
- Keep PRs focused on a single feature or fix
- Include tests for new functionality
- Update relevant documentation
- Ensure all CI checks pass
- Respond to review feedback promptly

## Project Structure

```
basalt/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── cli/                 # Command definitions
│   ├── core/                # Provider-agnostic logic
│   │   ├── environment.rs   # Environment checks ✅
│   │   ├── git.rs           # Git operations (TODO)
│   │   ├── stack.rs         # Stack detection (TODO)
│   │   └── metadata.rs      # Metadata storage (TODO)
│   ├── providers/           # Provider implementations
│   │   ├── mod.rs           # Provider trait ✅
│   │   ├── gitlab.rs        # GitLab provider (in progress)
│   │   ├── github.rs        # GitHub provider (TODO)
│   │   └── mock.rs          # Mock provider ✅
│   └── error.rs             # Error types ✅
├── tests/                   # Integration tests
├── docs/                    # Documentation
├── scripts/                 # Development scripts
└── Makefile.toml           # cargo-make tasks
```

## Development Guidelines

### Architectural Principles

1. **Provider abstraction is sacred** - All provider-specific logic goes in provider implementations
2. **Core must be provider-agnostic** - Never hardcode provider assumptions in core code
3. **Delegate to provider CLIs** - Use `glab`, `gh`, etc. instead of direct API calls
4. **Local-first** - Never require network for read-only operations
5. **Explicit over implicit** - Predictable, transparent behavior

See [AGENT.md](./AGENT.md) for detailed architectural guidelines.

### Coding Style

- Follow standard Rust conventions (`rustfmt`, `clippy`)
- Use `Result<T, Error>` for all fallible operations
- Prefer `&str` for function parameters, `String` for owned data
- Document public APIs with `///` doc comments
- Keep module-level documentation comprehensive
- Avoid obvious comments - document *why*, not *what*

### Error Handling

```rust
// Good: Specific, actionable error messages
return Err(Error::ProviderCliNotFound {
    provider: "GitLab",
    cli_name: "glab",
    install_url: "https://gitlab.com/gitlab-org/cli",
});

// Bad: Generic error
return Err(Error::Other("Something went wrong".to_string()));
```

### Testing

- Write tests for all new functionality
- Use temporary git repositories for integration tests
- Test error conditions explicitly
- Provide clear test names that describe what's being tested
- Use the mock provider for unit testing provider-agnostic code

### Documentation

- Document all public APIs
- Include usage examples in doc comments
- Update README.md for user-facing changes
- Update AGENT.md for architectural changes
- Add inline comments for non-obvious logic

## Common Pitfalls

❌ **Don't:**
- Use provider-specific logic in core code
- Make network requests in core modules
- Bypass the provider abstraction
- Use `panic!()` - return `Result` instead
- Force-push without confirmation flags
- Assume linear git history

✅ **Do:**
- Use the provider trait for all provider operations
- Validate git state before operations
- Parse structured output (JSON) from provider CLIs
- Provide clear, actionable error messages
- Write tests with temporary git repositories
- Check dependencies are available

## Getting Help

- **Questions?** Open a discussion or issue
- **Stuck?** Check [AGENT.md](./AGENT.md) for architectural guidance
- **Docker issues?** See [docs/DOCKER_TESTING.md](./docs/DOCKER_TESTING.md)
- **Not sure where to start?** Look for issues labeled `good first issue`

## Resources

- [Rust Book](https://doc.rust-lang.org/book/) - Learn Rust
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) - Best practices
- [glab CLI](https://gitlab.com/gitlab-org/cli) - GitLab CLI reference
- [gh CLI](https://cli.github.com/) - GitHub CLI reference
- [cargo-make](https://github.com/sagiegurari/cargo-make) - Task runner

## License

By contributing to basalt, you agree that your contributions will be licensed under the same license as the project.

---

Thank you for contributing to basalt! 🪨