#!/usr/bin/env bash
# Check for OpenAPI spec drift between committed and generated versions

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}\")\" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
COMMITTED_SPEC="$ROOT_DIR/openapi.json"
TEMP_SPEC="$ROOT_DIR/openapi-generated.json"

echo "Checking for OpenAPI drift..."

# Generate fresh spec
bash "$SCRIPT_DIR/generate-openapi.sh"
mv "$ROOT_DIR/openapi.json" "$TEMP_SPEC"

# Restore committed spec
git checkout "$COMMITTED_SPEC" 2>/dev/null || true

# Compare
if ! diff -q "$COMMITTED_SPEC" "$TEMP_SPEC" > /dev/null 2>&1; then
    echo "❌ OpenAPI drift detected!"
    echo ""
    echo "The committed openapi.json differs from the generated spec."
    echo "Run 'pnpm openapi:generate' and commit the updated file."
    echo ""
    echo "Diff:"
    diff "$COMMITTED_SPEC" "$TEMP_SPEC" || true
    rm "$TEMP_SPEC"
    exit 1
fi

echo "✓ OpenAPI spec is up to date"
rm "$TEMP_SPEC"
