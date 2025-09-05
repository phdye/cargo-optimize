#!/bin/bash
# Apply permissions fixes and commit them to git
# Run this from the project root

set -e

echo "🚀 Applying cargo-optimize project cleanup..."
echo ""

# 1. Fix file permissions
echo "1️⃣  Fixing file permissions..."
./scripts/fix-permissions.sh

echo ""
echo "2️⃣  Updating git index to track permission changes..."

# Update git index to track permission changes
# Remove execute permissions from files that shouldn't have them
git update-index --chmod=-x *.md *.toml *.rs build.rs LICENSE* .gitignore .gitattributes .cargo-optimize.toml 2>/dev/null || true
git update-index --chmod=-x doc/*.md examples/*.rs src/*.rs src/bin/*.rs tests/*.rs 2>/dev/null || true
git update-index --chmod=-x .github/workflows/*.yml scripts/*.bat scripts/*.py 2>/dev/null || true

# Add execute permissions for shell scripts
git update-index --chmod=+x scripts/*.sh 2>/dev/null || true

echo ""
echo "3️⃣  Staging changes for commit..."

# Stage all changes (moved files + permission changes)
git add -A

echo ""
echo "4️⃣  Creating git commit..."

# Create the commit
git commit -m "fix: organize scripts and fix file permissions

- Move all script files (.sh, .bat, .py) to ./scripts/ directory
- Remove unnecessary execute permissions from:
  - Documentation files (*.md)
  - Source code files (*.rs)
  - Configuration files (*.toml, *.yml)
  - License and other text files
  - Windows batch files (*.bat - don't need +x on Unix)
- Keep execute permissions only for Unix shell scripts (*.sh)
- Improves project organization and fixes Windows->Unix permission issues"

echo ""
echo "✅ Cleanup completed successfully!"
echo ""
echo "📊 Changes made:"
echo "   • Moved script files to ./scripts/ directory"
echo "   • Fixed execute permissions on all files"
echo "   • Committed changes to git"
echo ""
echo "🎯 Script files are now in ./scripts/:"
echo "   • ./scripts/setup.sh - Unix/Linux setup"
echo "   • ./scripts/setup.bat - Windows setup"  
echo "   • ./scripts/fix-permissions.sh - Fix permissions"
echo "   • ./scripts/remove-execute-permissions.sh - Legacy cleanup"
echo ""
echo "💡 Users of the crate will see a clean root directory,"
echo "   while contributors can find development scripts in ./scripts/"
