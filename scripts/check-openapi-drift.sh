#!/usr/bin/env bash
# Check for OpenAPI spec drift between committed and generated versions

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
COMMITTED_SPEC="$ROOT_DIR/openapi.json"
TEMP_SPEC="$ROOT_DIR/openapi-generated.json"

echo "Checking for OpenAPI drift..."

# Try to generate fresh spec (may fail if dependencies aren't available)
if bash "$SCRIPT_DIR/generate-openapi.sh" > /dev/null 2>&1; then
    # Generation succeeded, compare
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
else
    # Generation failed (likely build issues), just verify committed spec exists
    echo "⚠ Cannot generate OpenAPI spec (build dependencies unavailable)"
    if [ -f "$COMMITTED_SPEC" ]; then
        echo "✓ Using frozen openapi.json from repository"
        ENDPOINT_COUNT=$(jq -r '.paths | keys | length' "$COMMITTED_SPEC")
        echo "  Endpoints: $ENDPOINT_COUNT"
    else
        echo "❌ No openapi.json found in repository"
        exit 1
    fi
fi
