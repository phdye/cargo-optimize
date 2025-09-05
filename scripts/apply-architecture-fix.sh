#!/bin/bash
# Remove deprecated setup scripts and apply architectural improvements
# Run this from project root: ./scripts/apply-architecture-fix.sh

set -e

echo "🏗️  Applying cargo-optimize architectural improvements..."
echo ""

# Check we're in the right directory
if [[ ! -f "Cargo.toml" ]] || [[ ! -d "src" ]]; then
    echo "❌ Error: Please run this script from the cargo-optimize project root"
    exit 1
fi

echo "1️⃣  Removing end-user scripts from contributor directory..."

# Remove setup scripts that don't belong in ./scripts/
if [[ -f "scripts/setup.sh" ]]; then
    echo "   🗑️  Removing scripts/setup.sh (functionality moved to CLI)"
    rm "scripts/setup.sh"
fi

if [[ -f "scripts/setup.bat" ]]; then
    echo "   🗑️  Removing scripts/setup.bat (functionality moved to CLI)"
    rm "scripts/setup.bat"
fi

echo "   ✅ End-user scripts removed from contributor directory"
echo ""

echo "2️⃣  Fixing file permissions..."
./scripts/fix-permissions.sh
echo ""

echo "3️⃣  Updating git index..."
# Update git to track permission changes and removals
git add -A
echo "   ✅ Changes staged"
echo ""

echo "4️⃣  Testing CLI compilation..."
if cargo check --bin cargo-optimize; then
    echo "   ✅ CLI compiles successfully"
else
    echo "   ❌ CLI compilation failed - please fix before committing"
    exit 1
fi
echo ""

echo "5️⃣  Creating commit..."
git commit -m "refactor: move setup functionality from scripts to CLI

BREAKING CHANGE: Removed setup.sh and setup.bat scripts

- Remove setup.{sh,bat} from ./scripts/ directory  
- Add 'cargo optimize setup' CLI subcommand (replaces setup scripts)
- Add 'cargo optimize init' CLI subcommand for project initialization
- Add dev-setup.rs for contributor environment setup
- Update scripts/README.md with proper architecture documentation
- Fix file permissions across the project

End users should now use:
  cargo install cargo-optimize
  cargo optimize setup

Instead of downloading/running setup scripts.

Contributors can use:
  rust-script scripts/dev-setup.rs

For development environment setup."

echo ""
echo "✅ Architecture improvements applied successfully!"
echo ""
echo "📊 Summary of changes:"
echo "   ❌ Removed: scripts/setup.sh, scripts/setup.bat"
echo "   ✅ Added: CLI subcommands 'setup' and 'init'"
echo "   ✅ Added: scripts/dev-setup.rs for contributors"
echo "   ✅ Updated: scripts/README.md with proper documentation"
echo "   ✅ Fixed: File permissions across project"
echo ""
echo "🎯 New user workflow:"
echo "   cargo install cargo-optimize"
echo "   cargo optimize setup"
echo ""
echo "🛠️  New contributor workflow:"
echo "   rust-script scripts/dev-setup.rs"
echo ""
echo "🎉 Architecture is now properly organized!"
