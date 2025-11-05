#!/usr/bin/env bash
# Generate OpenAPI specification from daemon

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
DAEMON_DIR="$ROOT_DIR/apps/daemon"
OUTPUT_FILE="$ROOT_DIR/openapi.json"

echo "Building daemon..."
cd "$DAEMON_DIR"
cargo build --release 2>&1 | grep -E "(Compiling|Finished)" || true

echo "Starting daemon to generate OpenAPI spec..."
DAEMON_PID=""

# Function to cleanup daemon on exit
cleanup() {
    if [ -n "$DAEMON_PID" ]; then
        echo "Stopping daemon (PID: $DAEMON_PID)..."
        kill $DAEMON_PID 2>/dev/null || true
    fi
}
trap cleanup EXIT

# Start daemon in background
"$DAEMON_DIR/../../target/release/sisctl" &
DAEMON_PID=$!

echo "Daemon started with PID: $DAEMON_PID"
echo "Waiting for daemon to be ready..."

# Wait for health endpoint
for i in {1..30}; do
    if curl -s http://127.0.0.1:8871/health > /dev/null 2>&1; then
        echo "Daemon is ready!"
        break
    fi
    if [ $i -eq 30 ]; then
        echo "ERROR: Daemon failed to start within 30 seconds"
        exit 1
    fi
    sleep 1
done

echo "Fetching OpenAPI spec..."
curl -s http://127.0.0.1:8871/api-docs/openapi.json | jq '.' > "$OUTPUT_FILE"

echo "OpenAPI spec saved to: $OUTPUT_FILE"
echo "Spec version: $(jq -r '.info.version' "$OUTPUT_FILE")"
echo "Endpoints: $(jq -r '.paths | keys | length' "$OUTPUT_FILE")"
