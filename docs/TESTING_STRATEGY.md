# Testing Strategy

This document explains the comprehensive testing strategy for basalt, covering both native and Docker-based testing approaches.

---

## Overview

basalt uses a **two-tier testing approach** that balances speed and thoroughness:

1. **Native Tests** - Fast iteration during development
2. **Docker Tests** - Comprehensive environment validation

This strategy ensures:
- ✅ Fast feedback during development
- ✅ Thorough validation before shipping
- ✅ Confidence in handling missing dependencies
- ✅ Consistent behavior across platforms

---

## Native Tests (Fast)

### What They Test

Native tests run directly on your development machine using `cargo test`:

- Unit tests for core logic
- Integration tests with temporary git repositories
- Provider trait implementations
- Error handling and edge cases
- Metadata serialization/deserialization

### When to Use

✅ **Use native tests for:**
- Daily development and iteration
- Testing specific functionality
- Debugging logic issues
- Quick validation before commits
- Running in IDE/editor

### How to Run

```bash
# Run all tests
cargo test

# Run with verbose output
cargo test -- --nocapture

# Run specific test
cargo test test_git_available -- --exact

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test '*'

# Run with backtrace
RUST_BACKTRACE=1 cargo test
```

### Performance

- **Typical duration**: 1-5 seconds for all tests
- **Feedback loop**: Nearly instant
- **Best for**: Rapid iteration and development

---

## Docker Tests (Comprehensive)

### What They Test

Docker tests verify basalt behavior in different environment configurations:

| Scenario | Git | glab | gh | Purpose |
|----------|-----|------|----|---------|
| `all` (default) | ✅ | ❌ | ❌ | Standard environment |
| `no-git` | ❌ | ❌ | ❌ | Error handling for missing git |
| `no-providers` | ✅ | ❌ | ❌ | Core functionality without CLIs |
| `with-providers` | ✅ | ✅ | ✅ | Full provider CLI support |

### When to Use

✅ **Use Docker tests for:**
- Testing dependency handling
- Verifying error messages
- Pre-push validation
- Before submitting PRs
- Debugging environment-specific issues
- CI parity verification

❌ **Don't use Docker for:**
- Quick iteration during development (use native tests instead)
- Testing individual functions (use `cargo test test_name`)

### How to Run

#### Using the Convenience Script

```bash
# Run all scenarios (recommended before PR)
./scripts/test-docker.sh matrix

# Run specific scenarios
./scripts/test-docker.sh all              # Full environment
./scripts/test-docker.sh no-git           # Without git
./scripts/test-docker.sh no-providers     # Without glab/gh
./scripts/test-docker.sh with-providers   # With glab + gh

# Development and debugging
./scripts/test-docker.sh verbose          # Detailed output
./scripts/test-docker.sh shell            # Interactive shell
./scripts/test-docker.sh shell-no-git     # Shell without git

# Unit/integration only
./scripts/test-docker.sh unit
./scripts/test-docker.sh integration

# Code quality checks
./scripts/test-docker.sh check            # fmt + clippy only

# Maintenance
./scripts/test-docker.sh build            # Build image
./scripts/test-docker.sh clean            # Clean up
```

#### Using cargo-make (Recommended)

```bash
# Quick commands
cargo make test-docker                    # Full environment
cargo make test-docker-matrix             # All scenarios
cargo make test-docker-no-git             # Missing git scenario
cargo make test-docker-no-providers       # Without CLIs
cargo make test-docker-with-providers     # With CLIs

# Development
cargo make test-docker-verbose            # Verbose output
cargo make test-docker-shell              # Interactive shell
cargo make test-docker-clean              # Cleanup
```

#### Using Docker Compose Directly

```bash
# Run specific services
docker compose -f docker-compose.test.yml run --rm test
docker compose -f docker-compose.test.yml run --rm test-no-git
docker compose -f docker-compose.test.yml run --rm test-no-providers
docker compose -f docker-compose.test.yml run --rm test-with-providers

# Interactive debugging
docker compose -f docker-compose.test.yml run --rm shell
docker compose -f docker-compose.test.yml run --rm shell-no-git

# Cleanup
docker compose -f docker-compose.test.yml down -v
```

### Performance

- **First build**: 2-5 minutes (downloads Rust, dependencies, provider CLIs)
- **Subsequent builds**: 10-30 seconds (uses layer caching)
- **Test execution**: 10-60 seconds depending on scenario
- **Best for**: Comprehensive validation and environment testing

### Docker Architecture

The multi-stage Dockerfile provides different test environments:

```
base (Rust only)
├── with-git (+ git)
│   ├── with-providers (+ glab + gh)
│   ├── test (default: git + rustfmt + clippy)
│   └── no-providers (git only, no CLIs)
└── no-git (for testing error handling)
```

**Benefits:**
- Shared layers minimize build time
- Each scenario reuses common base
- Layer caching speeds up rebuilds
- Volume caching for cargo registry

---

## Recommended Workflow

### During Development

```bash
# Fast iteration
cargo test                    # Run all tests
cargo test test_name          # Run specific test
cargo test -- --nocapture     # See println! output
```

**Frequency**: After every significant code change

### Before Committing

```bash
# Run all checks
cargo make pre-commit
# or manually:
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
```

**Frequency**: Before every commit

### Before Pushing

```bash
# Run CI checks + Docker tests
cargo make ci                 # Format, lint, test
cargo make test-docker        # Docker validation
# or
./scripts/test-docker.sh all
```

**Frequency**: Before every push

### Before Submitting PR

```bash
# Run comprehensive test matrix
cargo make test-docker-matrix
# or
./scripts/test-docker.sh matrix
```

**Frequency**: Before opening or updating a PR

This runs all environment scenarios:
1. Full environment with git
2. Missing git (verifies error handling)
3. Without provider CLIs
4. With provider CLIs (glab + gh)

### Debugging Issues

```bash
# For logic issues - use native tests
cargo test failing_test -- --nocapture
RUST_BACKTRACE=1 cargo test

# For environment issues - use Docker
./scripts/test-docker.sh shell
# In container:
cargo test failing_test -- --nocapture
git --version
cargo --version
```

---

## CI/CD Strategy

### GitHub Actions CI

The CI pipeline runs on every push and pull request:

#### Fast Feedback (Runs on Every Push)

```yaml
test:
  matrix:
    os: [ubuntu-latest, macos-latest, windows-latest]
  steps:
    - Check formatting (rustfmt)
    - Run clippy
    - Run all tests (cargo test)
```

**Duration**: ~5-10 minutes
**Purpose**: Quick feedback on cross-platform compatibility

#### Docker Tests (Runs on PRs and Main)

```yaml
docker-tests:
  runs-on: ubuntu-latest
  steps:
    - Build test image (with caching)
    - Run all tests in Docker
    - Test without git (error handling)
    - Test in non-git directory
```

**Duration**: ~5-15 minutes (first run), ~2-5 minutes (cached)
**Purpose**: Comprehensive environment validation

#### Additional Checks

- **Minimum Rust version**: Verify compatibility with Rust 1.85+
- **Code coverage**: Generate coverage reports (tarpaulin)
- **Release builds**: Build binaries for multiple platforms

### Cache Strategy

CI uses caching to improve performance:

- **Cargo registry**: Cached across runs
- **Cargo build**: Platform-specific caching
- **Docker layers**: GitHub Actions cache for Docker builds

---

## Test Scenarios Explained

### 1. Full Environment (`test`, `all`)

**What it tests:**
- Standard development environment
- Git operations with temporary repositories
- Metadata handling
- Core stack logic
- Provider trait implementations

**Environment:**
- ✅ Git CLI
- ✅ Rust toolchain
- ✅ rustfmt + clippy
- ❌ Provider CLIs (glab/gh not installed)

**Expected result:** All tests pass

### 2. No Git Scenario (`test-no-git`, `no-git`)

**What it tests:**
- Graceful degradation when git is missing
- Error messages guide users to install git
- Functions that check for git fail appropriately

**Environment:**
- ❌ Git CLI (not installed)
- ✅ Rust toolchain

**Expected result:** 
- ⚠️ Some tests fail (intentionally)
- ✅ Error messages are helpful and actionable
- ✅ No panics or obscure errors

**Why this matters:** Users might run `bt` in environments without git, and we need to provide clear guidance.

### 3. No Providers Scenario (`test-no-providers`, `no-providers`)

**What it tests:**
- Core functionality works without provider CLIs
- Provider CLI checks detect missing tools correctly
- Basalt can initialize and manage metadata without providers

**Environment:**
- ✅ Git CLI
- ✅ Rust toolchain
- ❌ glab (GitLab CLI not installed)
- ❌ gh (GitHub CLI not installed)

**Expected result:** All tests pass

**Why this matters:** Users should be able to use core basalt features even if they haven't installed provider CLIs yet.

### 4. With Providers Scenario (`test-with-providers`, `with-providers`)

**What it tests:**
- Provider CLI detection succeeds
- Version checks work correctly
- Future: Actual provider operations (create/update MRs/PRs)

**Environment:**
- ✅ Git CLI
- ✅ Rust toolchain
- ✅ glab v1.42.0
- ✅ gh (latest)

**Expected result:** All tests pass, provider availability checks succeed

**Why this matters:** Verifies the happy path when all dependencies are present.

---

## Writing New Tests

### Unit Tests (in `src/`)

Place tests in the same file as the code:

```rust
//! Module documentation

pub fn some_function() -> Result<String> {
    // Implementation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_some_function() {
        let result = some_function().unwrap();
        assert_eq!(result, "expected");
    }

    #[test]
    fn test_error_case() {
        let result = some_function();
        assert!(result.is_err());
    }
}
```

### Integration Tests (in `tests/`)

Create separate files in `tests/` directory:

```rust
//! Integration test for feature X

use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_full_workflow() {
    // Create temp git repo
    let temp_dir = TempDir::new().unwrap();
    
    // Set up test scenario
    Command::new("git")
        .args(["init"])
        .current_dir(&temp_dir)
        .status()
        .unwrap();
    
    // Test basalt behavior
    // ...
    
    // Assertions
    assert!(/* condition */);
}
```

### Tests for Environment Checks

When testing environment dependencies:

```rust
#[test]
fn test_git_available() {
    let output = Command::new("git")
        .arg("--version")
        .output()
        .expect("Git should be available");
    
    assert!(output.status.success());
}

#[test]
fn test_not_in_git_repository() {
    let temp_dir = TempDir::new().unwrap();
    
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(&temp_dir)
        .output()
        .unwrap();
    
    assert!(!output.status.success(), 
        "Should fail outside git repo");
}
```

### Docker-Specific Test Considerations

Some tests behave differently in Docker:

```rust
#[test]
#[cfg_attr(
    all(target_os = "linux", not(target_env = "musl")),
    ignore
)]
fn test_that_needs_special_environment() {
    // This test only runs in specific scenarios
}
```

Or check environment at runtime:

```rust
#[test]
fn test_git_detection() {
    let git_available = Command::new("git")
        .arg("--version")
        .output()
        .is_ok();
    
    if !git_available {
        // Test error handling path
        assert!(/* basalt detects missing git */);
    } else {
        // Test normal path
        assert!(/* basalt uses git */);
    }
}
```

---

## Troubleshooting

### Tests Pass Locally but Fail in CI

**Possible causes:**
1. Platform-specific behavior (Windows vs Linux vs macOS)
2. Different environment variables
3. Different file system behavior
4. Missing dependencies in CI

**Solutions:**
- Check CI logs for specific errors
- Run Docker tests locally: `./scripts/test-docker.sh all`
- Compare environment: `cargo --version`, `git --version`
- Check for hardcoded paths or assumptions

### Docker Tests Are Slow

**First run is always slow** (downloads everything):
- Rust toolchain installation
- Dependency compilation
- Provider CLI downloads

**Subsequent runs should be fast** (uses caching):
- Docker layer caching
- Cargo registry volumes
- Build artifact caching

**If subsequent runs are still slow:**
```bash
# Check cache volumes exist
docker volume ls | grep cargo

# Clean and rebuild if needed
./scripts/test-docker.sh clean
./scripts/test-docker.sh build
```

### Docker Build Fails

**Common issues:**
1. Docker daemon not running
2. Insufficient disk space
3. Network issues downloading dependencies

**Solutions:**
```bash
# Check Docker is running
docker info

# Clean up space
docker system prune -a

# Rebuild without cache
./scripts/test-docker.sh build --no-cache
```

### Tests Fail in `no-git` Scenario

**This is often intentional!** The `no-git` scenario is designed to verify error handling.

**Expected failures:**
- Tests that require git should fail gracefully
- Error messages should be clear and actionable
- No panics or obscure errors

**Unexpected failures:**
- Panics or crashes
- Unclear error messages
- Tests that shouldn't need git failing

---

## Best Practices

### DO ✅

- **Run native tests frequently** during development
- **Run Docker tests before pushing** for confidence
- **Run test matrix before PRs** for comprehensive validation
- **Write tests for error conditions** explicitly
- **Use temporary git repos** for integration tests
- **Test with and without dependencies** present
- **Provide clear error messages** that guide users
- **Use `cargo make ci`** for local CI parity

### DON'T ❌

- **Don't skip Docker tests** before submitting PRs
- **Don't assume dependencies are present** without checking
- **Don't panic on missing dependencies** - return helpful errors
- **Don't hardcode paths** - use temp directories
- **Don't require network** for core tests
- **Don't make tests platform-specific** unless necessary
- **Don't ignore failing tests** - fix or document why they fail

---

## Summary

| Test Type | Speed | Coverage | When to Use |
|-----------|-------|----------|-------------|
| **Native** | ⚡ Fast (1-5s) | Good | Development, iteration |
| **Docker (all)** | 🐢 Slower (10-60s) | Better | Pre-push, validation |
| **Docker (matrix)** | 🐌 Slowest (1-3min) | Best | Pre-PR, comprehensive |

**Golden Rule:** Use the fastest test that gives you confidence, but don't skip comprehensive validation before shipping.

---

## Resources

- [DOCKER_TESTING.md](./DOCKER_TESTING.md) - Detailed Docker testing guide
- [QUICK_REFERENCE.md](./QUICK_REFERENCE.md) - Quick command reference
- [CONTRIBUTING.md](../CONTRIBUTING.md) - Contribution guidelines
- [GitHub Actions CI](../.github/workflows/ci.yml) - CI configuration

---

**Questions?** Open an issue or check the documentation in `docs/`.