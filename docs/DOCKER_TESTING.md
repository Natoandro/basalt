# Docker Testing Guide

This guide explains how to use Docker for testing basalt in various environment configurations.

## Overview

basalt uses Docker testing to verify behavior in different environments:

- ✅ **Full environment** - with git, cargo, rustfmt, clippy (default)
- 🚫 **Missing git** - to verify error handling when git is not installed
- ⚠️ **Missing provider CLIs** - to test without glab/gh installed
- 🔧 **With provider CLIs** - to test with glab and gh available

This approach ensures basalt gracefully handles missing dependencies with clear error messages.

## Quick Start

```bash
# Run all tests in Docker (default scenario)
./scripts/test-docker.sh

# Or using cargo-make
cargo make test-docker
```

## Available Test Scenarios

### 1. Full Environment (Default)

Runs all tests with git, cargo, rustfmt, and clippy installed:

```bash
./scripts/test-docker.sh all
cargo make test-docker
```

**Includes:**
- Git CLI
- Rust toolchain (cargo, rustc, rustfmt, clippy)
- All integration tests with temporary git repositories

### 2. No Git Scenario

Tests behavior when git is not installed (verifies error handling):

```bash
./scripts/test-docker.sh no-git
cargo make test-docker-no-git
```

**Purpose:** Verify that basalt detects missing git and provides helpful error messages.

**Expected:** Some tests will fail - this is intentional to verify error handling works correctly.

### 3. No Provider CLIs Scenario

Tests with git but without glab or gh installed:

```bash
./scripts/test-docker.sh no-providers
cargo make test-docker-no-providers
```

**Purpose:** Ensure basalt core functionality works without provider CLIs.

**Expected:** All tests should pass. Provider-specific features will detect missing CLIs at runtime.

### 4. With Provider CLIs Scenario

Tests with git, glab, and gh all installed:

```bash
./scripts/test-docker.sh with-providers
cargo make test-docker-with-providers
```

**Purpose:** Test full functionality with provider CLIs available.

**Expected:** All tests pass, provider availability checks succeed.

### 5. Test Matrix

Run tests in all environment configurations:

```bash
./scripts/test-docker.sh matrix
cargo make test-docker-matrix
```

Runs:
1. Full environment tests
2. No-git tests (expects some failures)
3. No-providers tests
4. With-providers tests

**Use this before submitting PRs to ensure compatibility across environments.**

## Specialized Test Runs

### Verbose Output

Run tests with detailed output:

```bash
./scripts/test-docker.sh verbose
cargo make test-docker-verbose
```

### Unit Tests Only

Run only unit tests (faster):

```bash
./scripts/test-docker.sh unit
cargo make test-docker-unit
```

### Integration Tests Only

Run only integration tests:

```bash
./scripts/test-docker.sh integration
cargo make test-docker-integration
```

### Code Quality Checks

Run formatting and clippy checks only:

```bash
./scripts/test-docker.sh check
```

## Interactive Debugging

### Shell with Full Environment

Open an interactive bash shell in the Docker container with all dependencies:

```bash
./scripts/test-docker.sh shell
cargo make test-docker-shell
```

Inside the container:
```bash
# Run specific tests
cargo test test_git_available -- --exact --nocapture

# Check git version
git --version

# Run clippy
cargo clippy

# Exit when done
exit
```

### Shell without Git

Open a shell in an environment without git:

```bash
./scripts/test-docker.sh shell-no-git
```

Useful for testing error handling manually.

## Docker Compose Services

You can also use docker-compose directly for more control:

```bash
# Run default tests
docker compose -f docker-compose.test.yml run --rm test

# Run without git
docker compose -f docker-compose.test.yml run --rm test-no-git

# Run without provider CLIs
docker compose -f docker-compose.test.yml run --rm test-no-providers

# Run with provider CLIs
docker compose -f docker-compose.test.yml run --rm test-with-providers

# Interactive shell
docker compose -f docker-compose.test.yml run --rm shell
```

## Building and Cleaning

### Build Docker Image

Build the test image without running tests:

```bash
./scripts/test-docker.sh build
cargo make test-docker-build
```

### Rebuild from Scratch

Force rebuild without using cache:

```bash
./scripts/test-docker.sh all --no-cache
```

### Clean Up

Remove Docker images and volumes:

```bash
./scripts/test-docker.sh clean
cargo make test-docker-clean
```

## CI Integration

The Docker tests are integrated into GitHub Actions CI:

- **Fast native tests** run on every push (Linux/macOS/Windows)
- **Docker environment tests** run to verify dependency handling
- **Caching** is used to speed up Docker builds in CI

See `.github/workflows/ci.yml` for the full CI configuration.

## Architecture

### Multi-Stage Dockerfile

The `Dockerfile.test` uses multi-stage builds:

```
base (Rust only)
├── with-git (+ git)
│   ├── with-providers (+ glab + gh)
│   ├── test (default, with rustfmt/clippy)
│   └── no-providers (for testing without CLIs)
└── no-git (for testing error handling)
```

**Benefits:**
- Shared layers reduce build time
- Different scenarios reuse common base
- Layer caching speeds up rebuilds

### Volume Caching

Docker Compose uses volumes to cache cargo registry and git data:

- `cargo-registry` - Caches downloaded crates
- `cargo-git` - Caches git dependencies

This significantly speeds up subsequent test runs.

## Troubleshooting

### Docker Build Fails

1. Check Docker is running:
   ```bash
   docker info
   ```

2. Clean and rebuild:
   ```bash
   ./scripts/test-docker.sh clean
   ./scripts/test-docker.sh build --no-cache
   ```

### Tests Pass Locally but Fail in Docker

This usually indicates an environment-specific issue. Use the interactive shell to debug:

```bash
./scripts/test-docker.sh shell
# Inside container:
cargo test --verbose
cargo test failing_test_name -- --nocapture
```

### Permission Issues

If you get permission errors, ensure the script is executable:

```bash
chmod +x scripts/test-docker.sh
```

### Slow Builds

- First build will be slow (downloads Rust, dependencies, etc.)
- Subsequent builds use layer caching and are much faster
- Use `./scripts/test-docker.sh clean` if caches become stale

## Best Practices

### When to Use Docker Tests

✅ **Use Docker tests when:**
- Testing dependency handling (missing git, glab, gh)
- Verifying cross-platform compatibility
- Debugging environment-specific issues
- Running tests in a clean, reproducible environment
- Before submitting PRs (run matrix tests)

❌ **Don't need Docker for:**
- Quick iteration during development (use `cargo test`)
- Running individual tests (use `cargo test test_name`)
- Code formatting checks (use `cargo fmt`)

### Development Workflow

Recommended workflow:

1. **During development:** Use `cargo test` for fast iteration
2. **Before commit:** Use `cargo make pre-commit` (format, lint, test)
3. **Before push:** Use `cargo make test-docker` or `cargo make ci`
4. **Before PR:** Use `cargo make test-docker-matrix` to test all scenarios

### CI Strategy

- **Native tests** (fast): Run on every push for quick feedback
- **Docker tests** (thorough): Run on main branch and PRs for comprehensive validation
- **Matrix tests** (complete): Run nightly or before releases

## Advanced Usage

### Custom Test Command

Run a specific test in Docker:

```bash
docker compose -f docker-compose.test.yml run --rm test \
  cargo test test_name -- --exact --nocapture
```

### Mount Source Code

For development, mount source code (faster iteration):

```bash
docker compose -f docker-compose.test.yml run --rm \
  -v $(pwd):/app \
  test cargo test
```

### Different Rust Version

Build with a specific Rust version:

```bash
docker build -f Dockerfile.test \
  --build-arg RUST_VERSION=1.85.0 \
  -t basalt-test:custom .
```

## Examples

### Test Missing Git Handling

```bash
# Run the no-git scenario
./scripts/test-docker.sh no-git

# Or interactively:
./scripts/test-docker.sh shell-no-git
# In container:
cargo test test_git_available
# Should fail with helpful error message
```

### Test Provider CLI Detection

```bash
# Test without provider CLIs
./scripts/test-docker.sh no-providers
# All tests should pass

# Test with provider CLIs
./scripts/test-docker.sh with-providers
# Provider availability checks should succeed
```

### Debug Failing Test

```bash
# Open shell
./scripts/test-docker.sh shell

# Run failing test with output
cargo test failing_test_name -- --nocapture

# Check environment
git --version
cargo --version
```

## Resources

- [Docker Documentation](https://docs.docker.com/)
- [Docker Compose Documentation](https://docs.docker.com/compose/)
- [Rust Docker Images](https://hub.docker.com/_/rust)
- [basalt CI Configuration](../.github/workflows/ci.yml)

---

**Need help?** Open an issue or check the [Quick Reference](./QUICK_REFERENCE.md) for common commands.