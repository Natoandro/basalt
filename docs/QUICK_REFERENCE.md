# basalt Quick Reference

A quick reference guide for basalt development tasks.

## Installation

### Prerequisites
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install cargo-make
cargo install cargo-make

# Verify installation
cargo make verify-env
```

## Common Commands

### Development

```bash
# Show all available tasks
cargo make

# Run all tests
cargo make test

# Run tests with verbose output
cargo make test-verbose

# Run only unit tests
cargo make test-unit

# Run only integration tests
cargo make test-integration

# Watch for changes and run tests
cargo make watch
```

### Code Quality

```bash
# Format code
cargo make fmt

# Check formatting without changing files
cargo make fmt-check

# Run linter
cargo make clippy

# Check compilation
cargo make check

# Run all CI checks (format, lint, test)
cargo make ci

# Run before committing
cargo make pre-commit
```

### Building

```bash
# Build debug binary
cargo make build

# Build release binary
cargo make build-release

# Clean build artifacts
cargo make clean

# Install binary locally
cargo make install
```

### Docker Testing

```bash
# Run all tests in Docker
cargo make test-docker

# Run with verbose output
cargo make test-docker-verbose

# Rebuild Docker image and run tests
cargo make test-docker-build

# Open interactive shell
cargo make test-docker-shell

# Clean up Docker resources
cargo make test-docker-clean

# Build Docker image manually
cargo make docker-build

# Run tests using Docker directly
cargo make docker-run
```

### Using Scripts Directly

```bash
# Docker testing script
./scripts/test-docker.sh                 # Run all tests
./scripts/test-docker.sh --verbose       # Verbose output
./scripts/test-docker.sh --build         # Rebuild and test
./scripts/test-docker.sh --shell         # Interactive shell
./scripts/test-docker.sh --clean         # Clean up
./scripts/test-docker.sh --help          # Show help
```

### Using Docker Compose

```bash
# Run all tests
docker compose -f docker-compose.test.yml run --rm test

# Verbose output
docker compose -f docker-compose.test.yml run --rm test-verbose

# Unit tests only
docker compose -f docker-compose.test.yml run --rm test-unit

# Integration tests only
docker compose -f docker-compose.test.yml run --rm test-integration

# Interactive shell
docker compose -f docker-compose.test.yml run --rm shell

# Clean up
docker compose -f docker-compose.test.yml down -v
```

### Using Cargo Directly

```bash
# Run all tests
cargo test --all-targets

# Run specific test
cargo test test_name

# Run with verbose output
cargo test -- --nocapture

# Format code
cargo fmt

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy --all-targets -- -D warnings

# Build
cargo build

# Build release
cargo build --release
```

## Development Workflow

### Before Starting Work

```bash
# Pull latest changes
git pull origin main

# Create feature branch
git checkout -b feature/my-feature

# Verify environment
cargo make verify-env
```

### During Development

```bash
# Watch mode (runs tests on save)
cargo make watch

# Or watch compilation only
cargo make watch-check

# Format on save (configure in your editor)
# Or run manually
cargo make fmt
```

### Before Committing

```bash
# Run all checks
cargo make pre-commit

# Stage changes
git add .

# Commit with conventional commit message
git commit -m "feat(module): add new feature"
```

### Before Pushing

```bash
# Run CI checks
cargo make ci

# Run Docker tests
cargo make test-docker

# Push changes
git push origin feature/my-feature
```

## Troubleshooting

### Tests Failing

```bash
# Run with verbose output
cargo make test-verbose

# Run specific test to isolate issue
cargo test test_name -- --nocapture

# Check for uncommitted changes
git status
```

### Build Failing

```bash
# Check for errors
cargo make check

# Clean and rebuild
cargo make clean
cargo make build
```

### Docker Issues

```bash
# Rebuild Docker image
cargo make test-docker-build

# Open shell to debug
cargo make test-docker-shell

# Check Docker is running
docker info

# Clean up and retry
cargo make test-docker-clean
cargo make test-docker
```

### Formatting/Linting Issues

```bash
# Auto-fix formatting
cargo make fmt

# Check what clippy wants
cargo make clippy

# See detailed clippy output
cargo clippy --all-targets
```

## Project Structure

```
basalt/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── cli/                 # Command definitions
│   ├── core/                # Provider-agnostic logic
│   │   ├── environment.rs   # Environment checks ✅
│   │   └── mod.rs           # Core module exports
│   ├── providers/           # Provider implementations
│   │   ├── mod.rs           # Provider trait ✅
│   │   ├── gitlab.rs        # GitLab (in progress)
│   │   ├── github.rs        # GitHub (TODO)
│   │   └── mock.rs          # Testing ✅
│   └── error.rs             # Error types ✅
├── tests/                   # Integration tests
│   └── environment_tests.rs # Environment tests ✅
├── docs/                    # Documentation
├── scripts/                 # Development scripts
├── Cargo.toml              # Dependencies
└── Makefile.toml           # cargo-make tasks
```

## Useful Environment Variables

```bash
# Enable Rust backtrace
RUST_BACKTRACE=1 cargo test

# Enable debug logging
RUST_LOG=debug cargo run

# Disable colored output
NO_COLOR=1 cargo test
```

## Git Workflow

### Conventional Commits

```bash
feat(scope): add new feature
fix(scope): fix a bug
docs(scope): update documentation
test(scope): add or update tests
refactor(scope): refactor code
chore(scope): maintenance tasks
ci(scope): CI/CD changes
```

### Examples

```bash
git commit -m "feat(gitlab): implement MR creation"
git commit -m "fix(environment): handle missing .git directory"
git commit -m "docs(readme): add Docker testing section"
git commit -m "test(stack): add merge commit detection test"
```

## Resources

- **Main README**: [README.md](../README.md)
- **Contributing Guide**: [CONTRIBUTING.md](../CONTRIBUTING.md)
- **Architecture Guide**: [AGENT.md](../AGENT.md)
- **Docker Testing**: [docs/DOCKER_TESTING.md](./DOCKER_TESTING.md)
- **cargo-make**: [Makefile.toml](../Makefile.toml)

## Quick Links

- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [cargo-make Documentation](https://github.com/sagiegurari/cargo-make)
- [GitLab CLI (glab)](https://gitlab.com/gitlab-org/cli)
- [GitHub CLI (gh)](https://cli.github.com/)

---

**Tip**: Run `cargo make` to see all available tasks at any time!