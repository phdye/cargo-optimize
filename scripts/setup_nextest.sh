#!/bin/bash

# Install cargo-nextest for better test reporting
# This gives pytest-style unified test output

echo "Setting up cargo-nextest for better test reporting..."

# Check if we're in project root
if [ ! -f "Cargo.toml" ] || [ ! -d "src" ] || [ ! -d "issue" ]; then
    echo "ERROR: This script must be run from the project root directory"
    exit 1
fi

# Install cargo-nextest
echo "Installing cargo-nextest..."
cargo install cargo-nextest --locked

echo ""
echo "Installation complete! Usage:"
echo "  cargo nextest run              # Run all tests with unified output"
echo "  cargo nextest run --no-fail-fast  # Continue on failures"  
echo "  cargo nextest run --retries 2  # Retry flaky tests"
echo ""
echo "Example output:"
echo "--------------------------------"
echo "    Finished test [unoptimized + debuginfo] target(s) in 0.04s"
echo "    Starting 9 tests across 3 binaries"
echo "        PASS [   0.005s] cargo-optimize::tests::test_detect_linker"
echo "        PASS [   0.003s] cargo-optimize::tests::test_optimize_config"
echo "        ... (all tests shown with timing)"
echo "------------"
echo "     Summary [   0.123s] 9 tests run: 9 passed, 0 skipped"
echo "--------------------------------"
