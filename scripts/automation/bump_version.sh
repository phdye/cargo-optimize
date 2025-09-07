#!/bin/bash
# Bump version in Cargo.toml

set -e

if [ $# -ne 1 ]; then
    echo "Usage: $0 <new_version>"
    echo "Example: $0 0.2.0"
    exit 1
fi

NEW_VERSION=$1

# Update Cargo.toml
sed -i "s/^version = .*/version = \"$NEW_VERSION\"/" Cargo.toml

# Update Cargo.lock
cargo update

# Commit changes
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to $NEW_VERSION"

echo "Version bumped to $NEW_VERSION"
echo "Don't forget to:"
echo "  1. Update CHANGELOG.md"
echo "  2. Create git tag: git tag -a v$NEW_VERSION -m 'Release v$NEW_VERSION'"
echo "  3. Push changes: git push && git push --tags"
