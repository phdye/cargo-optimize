#!/bin/bash
# Collect metrics for monitoring

set -e

OUTPUT_FILE="issue/mvp/003/phase5/monitoring/metrics_$(date +%Y%m%d).json"

# Collect performance metrics
echo "{" > $OUTPUT_FILE
echo '  "timestamp": "'$(date -u +"%Y-%m-%dT%H:%M:%SZ")'",' >> $OUTPUT_FILE
echo '  "performance": {' >> $OUTPUT_FILE

# Run performance test
DETECTION_TIME=$(cargo run --release --example quick_start 2>&1 | grep "Detection time" | cut -d: -f2 | tr -d ' ms')
echo '    "detection_time_ms": '$DETECTION_TIME',' >> $OUTPUT_FILE

# Memory usage (simplified)
MEMORY=$(ps aux | grep cargo-optimize | awk '{print $6}' | head -1)
echo '    "memory_kb": '$MEMORY >> $OUTPUT_FILE

echo '  },' >> $OUTPUT_FILE

# Collect quality metrics
echo '  "quality": {' >> $OUTPUT_FILE

# Test results
TEST_OUTPUT=$(cargo test --quiet 2>&1 | tail -1)
if [[ $TEST_OUTPUT == *"test result: ok"* ]]; then
    TESTS_PASSED=$(echo $TEST_OUTPUT | grep -oP '\d+(?= passed)')
    TESTS_FAILED=$(echo $TEST_OUTPUT | grep -oP '\d+(?= failed)')
    echo '    "tests_passed": '$TESTS_PASSED',' >> $OUTPUT_FILE
    echo '    "tests_failed": '$TESTS_FAILED',' >> $OUTPUT_FILE
    echo '    "test_pass_rate": '$(( TESTS_PASSED * 100 / (TESTS_PASSED + TESTS_FAILED) )) >> $OUTPUT_FILE
fi

echo '  },' >> $OUTPUT_FILE

# Collect usage metrics (would come from crates.io API in production)
echo '  "usage": {' >> $OUTPUT_FILE
echo '    "mock_installations": 1234,' >> $OUTPUT_FILE
echo '    "mock_daily_active": 567' >> $OUTPUT_FILE
echo '  }' >> $OUTPUT_FILE

echo "}" >> $OUTPUT_FILE

echo "Metrics collected: $OUTPUT_FILE"
