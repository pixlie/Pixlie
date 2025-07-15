#!/bin/bash
set -euo pipefail

# Update version across all project files
# Usage: ./scripts/update-version.sh <new-version>

NEW_VERSION=${1:-}

if [[ -z "$NEW_VERSION" ]]; then
    echo "Usage: $0 <new-version>"
    echo "Example: $0 1.2.3"
    exit 1
fi

# Validate version format (semantic versioning)
if ! echo "$NEW_VERSION" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9]+(\.[a-zA-Z0-9]+)*)?$'; then
    echo "Error: Version must follow semantic versioning format (e.g., 1.2.3 or 1.2.3-alpha.1)"
    exit 1
fi

echo "Updating version to: $NEW_VERSION"

# Update Rust Cargo.toml
echo "Updating pixlie/Cargo.toml..."
sed -i "s/^version = \".*\"/version = \"$NEW_VERSION\"/" pixlie/Cargo.toml

# Update frontend package.json
if [[ -f "webapp/package.json" ]]; then
    echo "Updating webapp/package.json..."
    sed -i "s/\"version\": \".*\"/\"version\": \"$NEW_VERSION\"/" webapp/package.json
fi

# Update website package.json if it exists
if [[ -f "website/package.json" ]]; then
    echo "Updating website/package.json..."
    sed -i "s/\"version\": \".*\"/\"version\": \"$NEW_VERSION\"/" website/package.json
fi

# Regenerate Cargo.lock
echo "Updating Cargo.lock..."
cd pixlie
cargo check --quiet
cd ..

echo "Version updated successfully to $NEW_VERSION"
echo ""
echo "Next steps:"
echo "1. Review changes: git diff"
echo "2. Commit changes: git add -A && git commit -m \"chore: bump version to v$NEW_VERSION\""
echo "3. Create tag: git tag v$NEW_VERSION"
echo "4. Push: git push origin main --tags"