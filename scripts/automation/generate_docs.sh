#!/bin/bash
# Generate all documentation

set -e

echo "Generating documentation..."

# Generate API docs
cargo doc --no-deps --all-features

# Generate test report
echo "# Test Report" > docs/test_report.md
echo "Generated: $(date)" >> docs/test_report.md
echo "" >> docs/test_report.md
cargo test -- --format json --report-time | jq -r '.type' | sort | uniq -c >> docs/test_report.md

# Generate coverage report
if command -v cargo-tarpaulin &> /dev/null; then
    cargo tarpaulin --out Html --output-dir docs/coverage
fi

# Generate benchmark report
if [ -f benchmark.json ]; then
    echo "# Benchmark Report" > docs/benchmark_report.md
    echo "Generated: $(date)" >> docs/benchmark_report.md
    jq '.results[0]' benchmark.json >> docs/benchmark_report.md
fi

echo "Documentation generated in docs/"
