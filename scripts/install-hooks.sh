#!/bin/sh
#
# Install git hooks for basalt development
#
# This script installs pre-commit hooks that run formatting and linting checks.
#
# Usage:
#   ./scripts/install-hooks.sh           # Install full hook (with tests)
#   ./scripts/install-hooks.sh --light   # Install lightweight hook (no tests)
#   ./scripts/install-hooks.sh --help    # Show help

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check if we should use colors (respect NO_COLOR environment variable)
if [ -n "$NO_COLOR" ] || [ ! -t 1 ]; then
    RED=''
    GREEN=''
    YELLOW=''
    BLUE=''
    NC=''
fi

# Parse arguments
HOOK_TYPE="full"

show_help() {
    echo "Usage: ./scripts/install-hooks.sh [OPTIONS]"
    echo ""
    echo "Install git pre-commit hooks for basalt development"
    echo ""
    echo "Options:"
    echo "  --light     Install lightweight hook (fmt + clippy, no tests)"
    echo "  --full      Install full hook (fmt + clippy + tests) [default]"
    echo "  --help      Show this help message"
    echo ""
    echo "Examples:"
    echo "  ./scripts/install-hooks.sh           # Install full hook"
    echo "  ./scripts/install-hooks.sh --light   # Install lightweight hook"
    echo ""
    echo "Hook Types:"
    echo "  Full (default):"
    echo "    - Code formatting check (cargo fmt)"
    echo "    - Linting (cargo clippy)"
    echo "    - Tests (cargo test)"
    echo "    - Slower but more thorough"
    echo ""
    echo "  Lightweight (--light):"
    echo "    - Code formatting check (cargo fmt)"
    echo "    - Linting (cargo clippy)"
    echo "    - No tests (faster for frequent commits)"
    echo "    - Run tests manually before pushing"
    exit 0
}

for arg in "$@"; do
    case "$arg" in
        --light)
            HOOK_TYPE="light"
            ;;
        --full)
            HOOK_TYPE="full"
            ;;
        --help|-h)
            show_help
            ;;
        *)
            echo "${RED}Error: Unknown option: $arg${NC}"
            echo "Run './scripts/install-hooks.sh --help' for usage information"
            exit 1
            ;;
    esac
done

echo "${BLUE}Installing git hooks for basalt...${NC}"
echo ""

# Check if we're in a git repository
if [ ! -d ".git" ]; then
    echo "${RED}Error: Not in a git repository${NC}"
    echo "Please run this script from the root of the basalt repository"
    exit 1
fi

# Check if hooks directory exists
if [ ! -d ".git/hooks" ]; then
    echo "${YELLOW}Warning: .git/hooks directory doesn't exist${NC}"
    echo "Creating .git/hooks directory..."
    mkdir -p .git/hooks
fi

# Determine which hook to install
if [ "$HOOK_TYPE" = "light" ]; then
    HOOK_SOURCE="scripts/hooks/pre-commit-light"
    HOOK_NAME="lightweight"
else
    HOOK_SOURCE="scripts/hooks/pre-commit"
    HOOK_NAME="full"
fi

HOOK_DEST=".git/hooks/pre-commit"

if [ ! -f "$HOOK_SOURCE" ]; then
    echo "${RED}Error: Hook source not found: $HOOK_SOURCE${NC}"
    exit 1
fi

# Check if hook already exists
if [ -f "$HOOK_DEST" ]; then
    # Check if it's our hook (by comparing content or checking for a marker)
    if grep -q "Pre-commit hook for basalt" "$HOOK_DEST" 2>/dev/null; then
        echo "${YELLOW}Pre-commit hook already installed, updating...${NC}"
    else
        echo "${YELLOW}Warning: Existing pre-commit hook found${NC}"
        echo "Backing up to $HOOK_DEST.backup"
        cp "$HOOK_DEST" "$HOOK_DEST.backup"
    fi
fi

# Copy the hook
cp "$HOOK_SOURCE" "$HOOK_DEST"
chmod +x "$HOOK_DEST"

echo "${GREEN}✓ Pre-commit hook installed ($HOOK_NAME)${NC}"
echo ""

if [ "$HOOK_TYPE" = "light" ]; then
    echo "The lightweight pre-commit hook will run:"
    echo "  1. Code formatting check (cargo fmt --check)"
    echo "  2. Linting (cargo clippy)"
    echo ""
    echo "${YELLOW}Note: Tests are NOT run by this hook.${NC}"
    echo "${YELLOW}Remember to run 'cargo test' before pushing!${NC}"
else
    echo "The full pre-commit hook will run:"
    echo "  1. Code formatting check (cargo fmt --check)"
    echo "  2. Linting (cargo clippy)"
    echo "  3. Tests (cargo test)"
    echo ""
    echo "${YELLOW}Note: This may slow down commits. Use --light for faster commits.${NC}"
fi

echo ""
echo "${BLUE}Usage:${NC}"
echo "  - Hooks run automatically on 'git commit'"
echo "  - To bypass: git commit --no-verify"
echo "  - To test manually: .git/hooks/pre-commit"
echo "  - To switch hooks: ./scripts/install-hooks.sh --light (or --full)"
echo ""
echo "${GREEN}✓ Installation complete!${NC}"
