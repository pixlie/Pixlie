#!/bin/bash
set -euo pipefail

# Generate checksums for release artifacts
# Usage: ./scripts/generate-checksums.sh [output-dir]

OUTPUT_DIR=${1:-"."}
CHECKSUM_FILE="${OUTPUT_DIR}/checksums.txt"

echo "Generating checksums for release artifacts..."

cd "${OUTPUT_DIR}"

# Remove existing checksum file
rm -f checksums.txt

# Generate checksums for all release artifacts
for file in *.tar.gz *.zip *.deb *.rpm *.dmg *.msi *.pkg *.exe; do
    if [[ -f "$file" ]]; then
        echo "Processing: $file"
        sha256sum "$file" >> checksums.txt
    fi
done

if [[ -f checksums.txt ]]; then
    echo "Checksums generated in: ${CHECKSUM_FILE}"
    echo "Contents:"
    cat checksums.txt
else
    echo "No release artifacts found to checksum"
    exit 1
fi