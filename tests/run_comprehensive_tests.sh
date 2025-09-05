#!/bin/bash

# Comprehensive test runner script for cargo-optimize

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default values
CATEGORIES="all"
VERBOSE=""
FAIL_FAST=""
REPORT="text"
NO_VALIDATE=""

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --categories)
            CATEGORIES="$2"
            shift 2
            ;;
        --verbose|-v)
            VERBOSE="--verbose"
            shift
            ;;
        --fail-fast)
            FAIL_FAST="--fail-fast"
            shift
            ;;
        --report)
            REPORT="$2"
            shift 2
            ;;
        --no-validate)
            NO_VALIDATE="--no-validate"
            shift
            ;;
        --quick)
            CATEGORIES="unit"
            FAIL_FAST="--fail-fast"
            echo -e "${YELLOW}Running quick tests (unit tests only)${NC}"
            shift
            ;;
        --ci)
            FAIL_FAST="--fail-fast"
            REPORT="junit"
            VERBOSE="--verbose"
            echo -e "${YELLOW}Running in CI mode${NC}"
            shift
            ;;
        --help|-h)
            echo "cargo-optimize comprehensive test runner"
            echo ""
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --categories <LIST>    Comma-separated list of test categories"
            echo "  --verbose, -v          Enable verbose output"
            echo "  --fail-fast            Stop on first test failure"
            echo "  --report <FORMAT>      Report format (text|json|junit|html)"
            echo "  --no-validate          Skip environment validation"
            echo "  --quick                Run quick tests only (unit tests)"
            echo "  --ci                   Run in CI mode"
            echo "  --help, -h             Show this help message"
            echo ""
            echo "Categories:"
            echo "  unit          Unit tests"
            echo "  integration   Integration tests"
            echo "  property      Property-based tests"
            echo "  fuzz          Fuzz tests"
            echo "  performance   Performance benchmarks"
            echo "  golden        Golden master tests"
            echo "  stress        Stress tests"
            echo "  boundary      Boundary value tests"
            echo "  regression    Regression tests"
            echo "  all           All tests (default)"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Build the test command
CMD="cargo test --bin test_main"

if [ "$CATEGORIES" != "all" ]; then
    CMD="$CMD --categories $CATEGORIES"
fi

if [ -n "$VERBOSE" ]; then
    CMD="$CMD $VERBOSE"
fi

if [ -n "$FAIL_FAST" ]; then
    CMD="$CMD $FAIL_FAST"
fi

if [ "$REPORT" != "text" ]; then
    CMD="$CMD --report $REPORT"
fi

if [ -n "$NO_VALIDATE" ]; then
    CMD="$CMD $NO_VALIDATE"
fi

# Print what we're going to run
echo -e "${GREEN}Running comprehensive tests...${NC}"
echo "Command: $CMD"
echo ""

# Run the tests
$CMD

# Check the exit code
if [ $? -eq 0 ]; then
    echo ""
    echo -e "${GREEN}✅ All tests passed!${NC}"
else
    echo ""
    echo -e "${RED}❌ Some tests failed${NC}"
    exit 1
fi
