#!/bin/sh
# Remove incorrect execute permissions from all non-executable files
# This fixes the issue where all files were incorrectly given execute permissions

echo "Removing unnecessary execute permissions from files..."

# Remove execute permissions from ALL files first
find . -type f -exec chmod 644 {} \; 2>/dev/null

# Now add execute permission ONLY to shell scripts that need it
chmod 755 setup.sh 2>/dev/null
chmod 755 fix-permissions.sh 2>/dev/null
chmod 755 remove-execute-permissions.sh 2>/dev/null

# Remove execute from specific file types that definitely shouldn't have it
find . -type f \( \
    -name "*.md" \
    -o -name "*.toml" \
    -o -name "*.rs" \
    -o -name "*.txt" \
    -o -name "*.yml" \
    -o -name "*.yaml" \
    -o -name "*.json" \
    -o -name "*.bat" \
    -o -name "*.gitignore" \
    -o -name "*.gitattributes" \
    -o -name "LICENSE*" \
    -o -name "CHANGELOG*" \
    -o -name "CONTRIBUTING*" \
    -o -name "README*" \
    -o -name "Cargo.lock" \
    -o -name "build.rs" \
\) -exec chmod 644 {} \; 2>/dev/null

echo "✓ Execute permissions removed from non-executable files"
echo ""
echo "Files that SHOULD have execute permission:"
echo "  - setup.sh (755)"
echo "  - fix-permissions.sh (755)"
echo "  - remove-execute-permissions.sh (755)"
echo ""
echo "All other files now have 644 (rw-r--r--) permissions"

# Show the results
echo ""
echo "Current permissions:"
ls -l | head -20
