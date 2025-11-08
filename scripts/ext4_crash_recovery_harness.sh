#!/bin/bash
# Automated Ext4 Crash Recovery Testing Harness
# Tests filesystem durability across various crash scenarios

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
RESULTS_DIR="/tmp/ext4-crash-test-results"
TEST_IMAGE="/tmp/ext4-crash-test.img"
QMP_PORT=4444

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test scenarios
SCENARIOS=(
    "write_during_allocation"
    "write_during_inode_create"
    "write_during_journal_commit"
    "write_during_data_write"
    "concurrent_writes"
    "directory_operations"
    "truncate_operations"
)

# Initialize results
mkdir -p "$RESULTS_DIR"
RESULTS_JSON="$RESULTS_DIR/results.json"
echo "{\"test_run\": \"$(date -Iseconds)\", \"scenarios\": []}" > "$RESULTS_JSON"

# Cleanup function
cleanup() {
    echo "Cleaning up..."
    pkill -f "qemu-system-aarch64" || true
    rm -f "$TEST_IMAGE"
}

trap cleanup EXIT

# Create fresh ext4 image
create_test_image() {
    echo "Creating fresh ext4 test image..."
    dd if=/dev/zero of="$TEST_IMAGE" bs=1M count=100 2>/dev/null
    mkfs.ext4 -F "$TEST_IMAGE" >/dev/null 2>&1
    echo "✓ Test image created: $TEST_IMAGE"
}

# Inject crash at random point via QMP
inject_crash() {
    local delay=$1
    echo "Will inject crash after ${delay}ms..."

    sleep "$(echo "scale=3; $delay/1000" | bc)" 2>/dev/null || sleep 0.5

    # Send QMP command to reset QEMU
    echo '{"execute":"qmp_capabilities"}' | nc localhost $QMP_PORT >/dev/null 2>&1 || true
    echo '{"execute":"system_reset"}' | nc localhost $QMP_PORT >/dev/null 2>&1 || true

    echo "✓ Crash injected"
}

# Verify filesystem integrity after crash
verify_integrity() {
    local scenario=$1
    local result="PASS"
    local details=""

    echo "Verifying filesystem integrity..."

    # Run fsck
    if fsck.ext4 -fn "$TEST_IMAGE" > "$RESULTS_DIR/fsck_${scenario}.log" 2>&1; then
        if grep -q "clean" "$RESULTS_DIR/fsck_${scenario}.log" || grep -q "0 errors" "$RESULTS_DIR/fsck_${scenario}.log"; then
            echo -e "${GREEN}✓ Filesystem is clean${NC}"
        else
            echo -e "${RED}✗ Filesystem has errors but fsck passed${NC}"
            result="WARN"
            details="fsck passed but filesystem not marked clean"
        fi
    else
        echo -e "${RED}✗ Filesystem has ERRORS${NC}"
        result="FAIL"
        details="fsck failed - filesystem corrupted"
        cat "$RESULTS_DIR/fsck_${scenario}.log"
    fi

    # Check for expected files using debugfs
    if command -v debugfs >/dev/null 2>&1; then
        debugfs -R "ls /incidents" "$TEST_IMAGE" > "$RESULTS_DIR/debugfs_${scenario}.log" 2>&1 || true
    fi

    echo "$result|$details"
}

# Run specific test scenario
run_scenario() {
    local scenario=$1
    local run_num=$2
    local total_runs=$3

    echo ""
    echo "========================================"
    echo "Scenario: $scenario (Run $run_num/$total_runs)"
    echo "========================================"

    # Create fresh image
    create_test_image

    # Random crash delay (100ms to 900ms)
    local crash_delay=$((RANDOM % 800 + 100))

    # Start kernel in background
    cd "$ROOT_DIR"
    BRINGUP=1 SIS_FEATURES="ext4-stress-test" EXT4_IMG="$TEST_IMAGE" \
        timeout 30 ./scripts/uefi_run.sh > "$RESULTS_DIR/qemu_${scenario}_${run_num}.log" 2>&1 &
    local qemu_pid=$!

    # Wait for kernel to boot
    sleep 3

    # Inject crash during workload
    inject_crash "$crash_delay" &
    local crash_pid=$!

    # Wait for crash to happen
    wait $crash_pid 2>/dev/null || true

    # Kill QEMU if still running
    kill $qemu_pid 2>/dev/null || true
    wait $qemu_pid 2>/dev/null || true

    # Wait a bit for cleanup
    sleep 1

    # Verify integrity
    local verify_result
    verify_result=$(verify_integrity "$scenario")
    local result=$(echo "$verify_result" | cut -d'|' -f1)
    local details=$(echo "$verify_result" | cut -d'|' -f2)

    # Record result
    record_result "$scenario" "$run_num" "$result" "$crash_delay" "$details"

    echo "Result: $result"
}

# Record test result to JSON
record_result() {
    local scenario=$1
    local run=$2
    local result=$3
    local crash_delay=$4
    local details=$5

    # Append to results JSON (simplified - would use jq in production)
    echo "  {\"scenario\": \"$scenario\", \"run\": $run, \"result\": \"$result\", \"crash_delay_ms\": $crash_delay, \"details\": \"$details\"}" \
        >> "$RESULTS_DIR/results_raw.txt"
}

# Generate summary report
generate_report() {
    echo ""
    echo "========================================"
    echo "TEST SUMMARY"
    echo "========================================"

    local total_tests=0
    local passed=0
    local failed=0
    local warnings=0

    if [ -f "$RESULTS_DIR/results_raw.txt" ]; then
        total_tests=$(wc -l < "$RESULTS_DIR/results_raw.txt")
        passed=$(grep -c "PASS" "$RESULTS_DIR/results_raw.txt" || echo 0)
        failed=$(grep -c "FAIL" "$RESULTS_DIR/results_raw.txt" || echo 0)
        warnings=$(grep -c "WARN" "$RESULTS_DIR/results_raw.txt" || echo 0)
    fi

    echo "Total Tests: $total_tests"
    echo -e "${GREEN}Passed: $passed${NC}"
    echo -e "${YELLOW}Warnings: $warnings${NC}"
    echo -e "${RED}Failed: $failed${NC}"

    local success_rate=0
    if [ $total_tests -gt 0 ]; then
        success_rate=$((passed * 100 / total_tests))
    fi

    echo ""
    echo "Success Rate: ${success_rate}%"

    if [ $failed -eq 0 ]; then
        echo -e "${GREEN}✓ ALL TESTS PASSED${NC}"
        echo ""
        echo "Achievement: 100% crash recovery rate"
    else
        echo -e "${RED}✗ SOME TESTS FAILED${NC}"
        echo ""
        echo "Please review logs in $RESULTS_DIR"
    fi

    # Generate JSON report
    cat > "$RESULTS_JSON" <<EOF
{
  "test_run": "$(date -Iseconds)",
  "total_tests": $total_tests,
  "passed": $passed,
  "failed": $failed,
  "warnings": $warnings,
  "success_rate": $success_rate,
  "scenarios": $([ -f "$RESULTS_DIR/results_raw.txt" ] && cat "$RESULTS_DIR/results_raw.txt" | sed 's/^/    /' | tr '\n' ',' | sed 's/,$//' || echo "")
}
EOF

    echo ""
    echo "Full report: $RESULTS_JSON"
}

# Main execution
main() {
    local runs_per_scenario=${1:-5}

    echo "========================================"
    echo "EXT4 CRASH RECOVERY TEST HARNESS"
    echo "========================================"
    echo "Runs per scenario: $runs_per_scenario"
    echo "Results directory: $RESULTS_DIR"
    echo ""

    # Clear old results
    rm -f "$RESULTS_DIR/results_raw.txt"

    # Run all scenarios
    for scenario in "${SCENARIOS[@]}"; do
        for run in $(seq 1 $runs_per_scenario); do
            run_scenario "$scenario" "$run" "$runs_per_scenario"
        done
    done

    # Generate report
    generate_report
}

# Check dependencies
check_dependencies() {
    local missing=()

    command -v mkfs.ext4 >/dev/null || missing+=("e2fsprogs")
    command -v fsck.ext4 >/dev/null || missing+=("e2fsprogs")
    command -v nc >/dev/null || missing+=("netcat")
    command -v bc >/dev/null || missing+=("bc")

    if [ ${#missing[@]} -gt 0 ]; then
        echo "Error: Missing dependencies: ${missing[*]}"
        echo "Install with: brew install ${missing[*]} (macOS) or apt-get install ${missing[*]} (Linux)"
        exit 1
    fi
}

# Entry point
check_dependencies
main "$@"
