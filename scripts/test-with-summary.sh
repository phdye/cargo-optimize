#!/bin/bash

# test-with-summary.sh - Uses cargo test's JSON output for accurate reporting

if [ ! -f "Cargo.toml" ] || [ ! -d "src" ] || [ ! -d "issue" ]; then
    echo "ERROR: This script must be run from the project root directory"
    exit 1
fi

# This uses cargo test's machine-readable JSON output
# Much more accurate than parsing human-readable output

echo "Running tests with detailed reporting..."
echo "======================================="

# Run tests with JSON output and process in real-time
cargo test --no-fail-fast -- -Z unstable-options --format json 2>/dev/null | {
    TOTAL_PASSED=0
    TOTAL_FAILED=0
    TOTAL_IGNORED=0
    FAILED_TESTS=""
    
    while IFS= read -r line; do
        # Parse JSON lines for test events
        if echo "$line" | grep -q '"type":"test"'; then
            if echo "$line" | grep -q '"event":"ok"'; then
                TOTAL_PASSED=$((TOTAL_PASSED + 1))
                # Extract test name for verbose output if needed
                TEST_NAME=$(echo "$line" | sed -n 's/.*"name":"\([^"]*\)".*/\1/p')
                echo "  ✓ $TEST_NAME"
            elif echo "$line" | grep -q '"event":"failed"'; then
                TOTAL_FAILED=$((TOTAL_FAILED + 1))
                TEST_NAME=$(echo "$line" | sed -n 's/.*"name":"\([^"]*\)".*/\1/p')
                echo "  ✗ $TEST_NAME FAILED"
                FAILED_TESTS="$FAILED_TESTS\n    - $TEST_NAME"
            elif echo "$line" | grep -q '"event":"ignored"'; then
                TOTAL_IGNORED=$((TOTAL_IGNORED + 1))
            fi
        elif echo "$line" | grep -q '"type":"suite"'; then
            # Suite finished, we have final counts
            continue
        fi
    done
    
    # Final summary
    echo ""
    echo "======================================="
    echo "TEST EXECUTION SUMMARY"
    echo "======================================="
    echo "Total: $((TOTAL_PASSED + TOTAL_FAILED + TOTAL_IGNORED)) tests"
    echo "  ✓ Passed:  $TOTAL_PASSED"
    echo "  ✗ Failed:  $TOTAL_FAILED"
    echo "  ⊘ Ignored: $TOTAL_IGNORED"
    
    if [ $TOTAL_FAILED -gt 0 ]; then
        echo ""
        echo "Failed tests:$FAILED_TESTS"
        echo "======================================="
        echo "RESULT: TESTS FAILED ❌"
        exit 1
    else
        echo "======================================="
        echo "RESULT: ALL TESTS PASSED ✅"
        exit 0
    fi
}

# Note: If JSON format doesn't work (requires nightly features), 
# fall back to the simple parser version:
if [ $? -ne 0 ]; then
    echo "JSON output not available, using simple parser..."
    bash scripts/cargo-test-summary.sh
fi
