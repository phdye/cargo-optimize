#!/bin/bash
# Fix file permissions for cargo-optimize project
# This script should be run from the project root: ./scripts/fix-permissions.sh

set -e

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$PROJECT_ROOT"

echo "🔧 Fixing file permissions for cargo-optimize project..."
echo "Working directory: $PROJECT_ROOT"
echo ""

# Set base permissions for all files to 644 (rw-r--r--)
# This removes execute permissions from everything first
echo "📂 Removing execute permissions from all files..."
find . -type f -not -path './.git/*' -exec chmod 644 {} \; 2>/dev/null

# Now grant execute permissions ONLY to actual shell scripts
echo "🚀 Setting execute permissions for shell scripts..."
chmod 755 scripts/*.sh 2>/dev/null || echo "Warning: No .sh files found in scripts/"

# Ensure directories have correct permissions (755)
echo "📁 Setting directory permissions..."
find . -type d -not -path './.git/*' -exec chmod 755 {} \; 2>/dev/null

echo ""
echo "✅ Permissions fixed successfully!"
echo ""
echo "📊 Summary of changes:"
echo "   ❌ Removed execute permissions from:"
echo "      • All .md files (documentation)"
echo "      • All .rs files (Rust source code)"  
echo "      • All .toml files (configuration)"
echo "      • All .yml/.yaml files (CI configuration)"
echo "      • All LICENSE files"
echo "      • build.rs and other build files"
echo "      • .gitignore, .gitattributes, .cargo-optimize.toml"
echo "      • .bat files (Windows batch - no execute needed on Unix)"
echo "      • .py files (Python scripts - use 'python script.py')"
echo ""
echo "   ✅ Kept/set execute permissions for:"
echo "      • scripts/*.sh (Unix shell scripts)"
echo ""
echo "🎯 File permission summary:"
echo "   • Regular files:  644 (rw-r--r--)"
echo "   • Shell scripts:  755 (rwxr-xr-x)"  
echo "   • Directories:    755 (rwxr-xr-x)"
echo ""
echo "💡 Note: .bat files should NOT have execute permissions on Unix/Linux systems."
echo "   They are meant to be run on Windows with 'setup.bat', not './setup.bat'"
