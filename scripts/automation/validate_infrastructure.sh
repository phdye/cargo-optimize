#!/bin/bash
# Validate entire automation infrastructure

set -e

echo "Validating automation infrastructure..."

ERRORS=0

# Check GitHub Actions workflows
echo -n "Checking CI workflow... "
if [ -f .github/workflows/ci.yml ]; then
    echo "✓"
else
    echo "✗"
    ((ERRORS++))
fi

echo -n "Checking release workflow... "
if [ -f .github/workflows/release.yml ]; then
    echo "✓"
else
    echo "✗"
    ((ERRORS++))
fi

echo -n "Checking test automation... "
if [ -f .github/workflows/test-comprehensive.yml ]; then
    echo "✓"
else
    echo "✗"
    ((ERRORS++))
fi

# Check scripts
echo -n "Checking automation scripts... "
if [ -d scripts/automation ] && [ $(ls scripts/automation/*.sh | wc -l) -gt 0 ]; then
    echo "✓"
else
    echo "✗"
    ((ERRORS++))
fi

# Check monitoring
echo -n "Checking monitoring config... "
if [ -f issue/mvp/003/phase5/monitoring/metrics_config.yaml ]; then
    echo "✓"
else
    echo "✗"
    ((ERRORS++))
fi

# Check documentation
echo -n "Checking documentation setup... "
if [ -f .github/workflows/docs.yml ]; then
    echo "✓"
else
    echo "✗"
    ((ERRORS++))
fi

# Summary
echo ""
if [ $ERRORS -eq 0 ]; then
    echo "✅ All infrastructure components validated successfully!"
    exit 0
else
    echo "❌ Found $ERRORS issues with infrastructure"
    exit 1
fi
