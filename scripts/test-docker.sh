#!/usr/bin/env bash
# Docker test runner for basalt
#
# This script provides convenient access to Docker-based testing with different
# environment configurations to test dependency handling.
#
# Usage:
#   ./scripts/test-docker.sh [scenario] [options]
#
# Scenarios:
#   all (default)    - Run all tests with full environment
#   no-git           - Test without git (verifies error handling)
#   no-providers     - Test with git but without glab/gh
#   with-providers   - Test with git, glab, and gh
#   verbose          - Run tests with verbose output
#   unit             - Run only unit tests
#   integration      - Run only integration tests
#   check            - Run formatting and clippy checks only
#   shell            - Open interactive shell in container
#   shell-no-git     - Open shell in container without git
#   build            - Build Docker image only
#   clean            - Remove Docker images and volumes
#
# Options:
#   --rebuild        - Force rebuild of Docker image
#   --no-cache       - Build without using cache
#   --help           - Show this help message
#
# Examples:
#   ./scripts/test-docker.sh                    # Run all tests
#   ./scripts/test-docker.sh verbose            # Run with verbose output
#   ./scripts/test-docker.sh no-git             # Test missing git scenario
#   ./scripts/test-docker.sh shell              # Interactive debugging
#   ./scripts/test-docker.sh all --rebuild      # Rebuild and test

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
COMPOSE_FILE="${PROJECT_ROOT}/docker-compose.test.yml"

# Default values
SCENARIO="${1:-all}"
BUILD_ARGS=()

# Parse additional arguments
shift || true
while [[ $# -gt 0 ]]; do
    case $1 in
        --rebuild)
            BUILD_ARGS+=("--build")
            shift
            ;;
        --no-cache)
            BUILD_ARGS+=("--no-cache" "--build")
            shift
            ;;
        --help|-h)
            grep '^#' "$0" | grep -v '#!/' | sed 's/^# \?//'
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            echo "Run with --help for usage information"
            exit 1
            ;;
    esac
done

# Change to project root
cd "${PROJECT_ROOT}"

# Helper functions
info() {
    echo -e "${BLUE}==>${NC} $*"
}

success() {
    echo -e "${GREEN}✓${NC} $*"
}

error() {
    echo -e "${RED}✗${NC} $*"
}

warning() {
    echo -e "${YELLOW}!${NC} $*"
}

header() {
    echo ""
    echo -e "${CYAN}========================================${NC}"
    echo -e "${CYAN}  $*${NC}"
    echo -e "${CYAN}========================================${NC}"
    echo ""
}

# Run docker compose command
run_compose() {
    local service=$1
    shift
    docker compose -f "${COMPOSE_FILE}" run --rm "${BUILD_ARGS[@]}" "$service" "$@"
}

# Main test scenarios
case "${SCENARIO}" in
    all|test)
        header "Running all tests with full environment"
        info "This includes: git, cargo, rustfmt, clippy"
        run_compose test
        success "All tests passed!"
        ;;

    no-git)
        header "Testing without git (error handling scenario)"
        info "This verifies basalt properly detects missing git"
        warning "Some tests are expected to fail - that's the point!"
        run_compose test-no-git || {
            info "Tests completed (some failures expected for missing git)"
        }
        ;;

    no-providers)
        header "Testing without provider CLIs (glab/gh)"
        info "This verifies basalt works without glab or gh installed"
        run_compose test-no-providers
        success "Tests passed without provider CLIs!"
        ;;

    with-providers)
        header "Testing with provider CLIs (glab + gh)"
        info "This tests with glab and gh installed"
        run_compose test-with-providers
        success "Tests passed with provider CLIs!"
        ;;

    verbose|v)
        header "Running tests with verbose output"
        run_compose test-verbose
        success "Verbose tests passed!"
        ;;

    unit)
        header "Running unit tests only"
        run_compose test-unit
        success "Unit tests passed!"
        ;;

    integration|int)
        header "Running integration tests only"
        run_compose test-integration
        success "Integration tests passed!"
        ;;

    check|lint)
        header "Running code quality checks (fmt + clippy)"
        run_compose check
        success "Code quality checks passed!"
        ;;

    shell|sh)
        header "Opening interactive shell in test container"
        info "Type 'exit' to leave the container"
        info "Git is available in this environment"
        run_compose shell
        ;;

    shell-no-git)
        header "Opening interactive shell without git"
        info "Type 'exit' to leave the container"
        warning "Git is NOT available in this environment"
        run_compose shell-no-git
        ;;

    build)
        header "Building Docker test image"
        info "Building all test stages..."
        docker compose -f "${COMPOSE_FILE}" build "${BUILD_ARGS[@]}"
        success "Docker image built successfully!"
        ;;

    clean)
        header "Cleaning Docker artifacts"
        info "Removing containers..."
        docker compose -f "${COMPOSE_FILE}" down -v 2>/dev/null || true
        info "Removing test images..."
        docker rmi basalt-test:latest 2>/dev/null || true
        docker images -f "dangling=true" -q | xargs -r docker rmi 2>/dev/null || true
        success "Cleanup complete!"
        ;;

    matrix)
        header "Running test matrix (all scenarios)"
        info "This runs tests in all environment configurations"

        echo ""
        info "1/4 - Full environment (with git)"
        run_compose test || error "Full environment tests failed"

        echo ""
        info "2/4 - Without git"
        run_compose test-no-git || warning "No-git tests completed (some failures expected)"

        echo ""
        info "3/4 - Without provider CLIs"
        run_compose test-no-providers || error "No-providers tests failed"

        echo ""
        info "4/4 - With provider CLIs"
        run_compose test-with-providers || error "Provider CLI tests failed"

        success "Test matrix complete!"
        ;;

    help|-h|--help)
        grep '^#' "$0" | grep -v '#!/' | sed 's/^# \?//'
        exit 0
        ;;

    *)
        error "Unknown scenario: ${SCENARIO}"
        echo ""
        echo "Available scenarios:"
        echo "  all, no-git, no-providers, with-providers, verbose,"
        echo "  unit, integration, check, shell, shell-no-git,"
        echo "  build, clean, matrix, help"
        echo ""
        echo "Run with --help for detailed usage information"
        exit 1
        ;;
esac
