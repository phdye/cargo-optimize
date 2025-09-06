#!/bin/bash

# cargo-test-summary.sh - Parses cargo test output for a unified summary
# Works with standard cargo test, no additional tools needed

if [ ! -f "Cargo.toml" ] || [ ! -d "src" ] || [ ! -d "issue" ]; then
    echo "ERROR: This script must be run from the project root directory"
    exit 1
fi

echo "Running tests with unified reporting..."
echo "======================================="

# Run cargo test and capture output
TEMP_FILE=$(mktemp)
cargo test --color=always 2>&1 | tee "$TEMP_FILE"

# Parse the output for test statistics
TOTAL_PASSED=0
TOTAL_FAILED=0
TOTAL_IGNORED=0
TEST_BINARIES=0

# Extract test results from each "test result:" line
while IFS= read -r line; do
    if [[ "$line" =~ ([0-9]+)\ passed ]]; then
        PASSED="${BASH_REMATCH[1]}"
        TOTAL_PASSED=$((TOTAL_PASSED + PASSED))
        TEST_BINARIES=$((TEST_BINARIES + 1))
    fi
    if [[ "$line" =~ ([0-9]+)\ failed ]]; then
        FAILED="${BASH_REMATCH[1]}"
        TOTAL_FAILED=$((TOTAL_FAILED + FAILED))
    fi
    if [[ "$line" =~ ([0-9]+)\ ignored ]]; then
        IGNORED="${BASH_REMATCH[1]}"
        TOTAL_IGNORED=$((TOTAL_IGNORED + IGNORED))
    fi
done < <(grep "test result:" "$TEMP_FILE")

# Display unified summary
echo ""
echo "======================================="
echo "UNIFIED TEST SUMMARY"
echo "======================================="
echo "Test binaries executed: $TEST_BINARIES"
echo "Total tests run: $((TOTAL_PASSED + TOTAL_FAILED))"
echo "  ✓ Passed:  $TOTAL_PASSED"
if [ $TOTAL_FAILED -gt 0 ]; then
    echo "  ✗ Failed:  $TOTAL_FAILED ⚠️"
else
    echo "  ✗ Failed:  $TOTAL_FAILED"
fi
echo "  ⊘ Ignored: $TOTAL_IGNORED"
echo "======================================="

# Set exit code based on failures
if [ $TOTAL_FAILED -gt 0 ]; then
    echo "RESULT: TESTS FAILED ❌"
    rm "$TEMP_FILE"
    exit 1
else
    echo "RESULT: ALL TESTS PASSED ✅"
    rm "$TEMP_FILE"
    exit 0
fi
