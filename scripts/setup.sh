#!/usr/bin/env bash
# cargo-optimize setup script
# This script helps set up cargo-optimize for a Rust project

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Functions
info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    error "Cargo is not installed. Please install Rust from https://rustup.rs/"
fi

# Check if cargo-optimize is installed
if ! command -v cargo-optimize &> /dev/null; then
    info "cargo-optimize is not installed. Installing..."
    cargo install cargo-optimize || error "Failed to install cargo-optimize"
fi

# Parse arguments
PROJECT_DIR="${1:-.}"
OPTIMIZATION_LEVEL="${2:-balanced}"

# Change to project directory
cd "$PROJECT_DIR" || error "Failed to change to project directory: $PROJECT_DIR"

# Check if this is a Rust project
if [ ! -f "Cargo.toml" ]; then
    error "No Cargo.toml found in $PROJECT_DIR. Is this a Rust project?"
fi

info "Setting up cargo-optimize for project in $PROJECT_DIR"

# Run analysis first
info "Analyzing project structure..."
cargo optimize analyze --detailed || warn "Analysis failed, continuing anyway"

# Apply optimizations
info "Applying optimizations (level: $OPTIMIZATION_LEVEL)..."
cargo optimize -O "$OPTIMIZATION_LEVEL" || error "Failed to apply optimizations"

# Install recommended tools if they're missing
if ! command -v sccache &> /dev/null; then
    info "Installing sccache for build caching..."
    cargo install sccache || warn "Failed to install sccache"
fi

# Platform-specific tool installation
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    if ! command -v mold &> /dev/null && ! command -v ld.mold &> /dev/null; then
        warn "mold linker not found. For best performance on Linux, install mold:"
        echo "  Ubuntu/Debian: sudo apt-get install mold"
        echo "  Fedora: sudo dnf install mold"
        echo "  From source: https://github.com/rui314/mold"
    fi
elif [[ "$OSTYPE" == "darwin"* ]] || [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]]; then
    if ! command -v lld &> /dev/null && ! command -v ld.lld &> /dev/null; then
        warn "lld linker not found. For better performance, install lld:"
        if [[ "$OSTYPE" == "darwin"* ]]; then
            echo "  macOS: brew install llvm"
        else
            echo "  Windows: scoop install llvm"
        fi
    fi
fi

# Run a test build to verify everything works
info "Running test build to verify optimizations..."
cargo build || error "Test build failed. Run 'cargo optimize reset' to revert changes."

# Show stats
info "Showing cache statistics..."
cargo optimize stats || true

# Success!
echo -e "\n${GREEN}✓ cargo-optimize successfully configured!${NC}"
echo -e "\nYour builds should now be significantly faster."
echo -e "\nUseful commands:"
echo "  cargo optimize benchmark  # Measure improvement"
echo "  cargo optimize stats      # Show cache statistics"
echo "  cargo optimize reset      # Revert all changes"
echo -e "\nHappy coding! 🚀"
