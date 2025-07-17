#!/bin/bash
set -euo pipefail

# Build script for cross-platform releases
# Usage: ./scripts/build-release.sh [target]

TARGET=${1:-"x86_64-unknown-linux-gnu"}
VERSION=${2:-"$(cd pixlie && cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version')"}

echo "Building Pixlie v${VERSION} for target: ${TARGET}"

# Create output directory
mkdir -p dist

# Build frontend first
echo "Building frontend..."
cd webapp
if command -v pnpm &> /dev/null; then
    pnpm install
    pnpm build
else
    npm install
    npm run build
fi
cd ..

# Build Rust backend
echo "Building backend for ${TARGET}..."
cd pixlie

# Install target if not already installed
rustup target add "${TARGET}" 2>/dev/null || true

# Build with optimizations
cargo build --release --target "${TARGET}"

echo "Build completed successfully!"

# Copy binaries to dist
if [[ "${TARGET}" == *"windows"* ]]; then
    cp "target/${TARGET}/release/pixlie.exe" ../dist/
    cp "target/${TARGET}/release/export_types.exe" ../dist/
    echo "Windows binaries copied to dist/"
else
    cp "target/${TARGET}/release/pixlie" ../dist/
    cp "target/${TARGET}/release/export_types" ../dist/
    strip ../dist/pixlie 2>/dev/null || true
    strip ../dist/export_types 2>/dev/null || true
    echo "Unix binaries copied to dist/ (stripped)"
fi

cd ..

# Copy webapp build
cp -r webapp/dist dist/webapp

# Copy documentation
cp LICENSE dist/
cp README.md dist/PROJECT_README.md

# Create a simple README for the binary distribution
cat > dist/README.md << EOF
# Pixlie v${VERSION}

AI-powered data analysis for Hacker News discussions using natural language queries.

## Quick Start

1. Run the binary:
   \`\`\`bash
   ./pixlie --help
   \`\`\`

2. Start the server:
   \`\`\`bash
   ./pixlie server --port 8080
   \`\`\`

3. Open your browser to \`http://localhost:8080\`

## Documentation

For full documentation, visit: https://github.com/pixlie/Pixlie

## License

MIT License - see LICENSE file for details.
EOF

echo "Release build ready in dist/ directory"