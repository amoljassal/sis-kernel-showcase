#!/bin/bash
# Week 8 Autonomous Predictive Memory Test
# Tests autonomous decision loop integration with predictive memory

set -e

echo "=== Week 8 Autonomous Predictive Memory Test ==="
echo ""
echo "This test validates:"
echo "  1. Strategy selection logic (Conservative/Balanced/Aggressive)"
echo "  2. Compaction prediction with 5-second lookahead"
echo "  3. Autonomous decision loop integration"
echo "  4. Learning statistics accumulation over time"
echo ""

# Function to send command and wait
send_cmd() {
    echo "$1"
    sleep 0.5
}

# Test 1: Basic functionality
echo "=== Test 1: Basic Functionality ==="
send_cmd "memctl strategy status"
send_cmd "memctl strategy test"
send_cmd "memctl predict compaction"
send_cmd "memctl learn stats"
echo ""

# Test 2: Build some history with stress testing
echo "=== Test 2: Build History with Stress Testing ==="
send_cmd "memctl stress 50"
send_cmd "memctl learn stats"
echo ""

# Test 3: Enable autonomous mode
echo "=== Test 3: Enable Autonomous Mode ==="
send_cmd "autoctl on"
send_cmd "autoctl interval 1000"
echo "Autonomous mode enabled with 1000ms decision interval"
echo ""

# Test 4: Wait for autonomous decisions
echo "=== Test 4: Wait for Autonomous Decisions ==="
echo "Waiting 30 seconds for autonomous decision loop to run..."
echo "Expected: ~30 autonomous decisions with predictive memory calls"
echo ""
for i in {1..30}; do
    echo -n "."
    sleep 1
done
echo ""
echo "Done waiting"
echo ""

# Test 5: Check learning statistics
echo "=== Test 5: Check Learning Statistics ==="
send_cmd "memctl learn stats"
echo ""
echo "Expected results:"
echo "  - Compaction predictions > 0"
echo "  - Strategy changes (if memory pressure varied)"
echo "  - Shows autonomous integration is working"
echo ""

# Test 6: Check autonomous audit log
echo "=== Test 6: Check Autonomous Audit Log ==="
send_cmd "autoctl audit last 10"
echo ""

# Test 7: Disable autonomy
echo "=== Test 7: Cleanup ==="
send_cmd "autoctl off"
send_cmd "memctl learn stats"
echo ""

echo "=== Test Complete ==="
echo ""
echo "To run this test interactively in QEMU:"
echo "  1. Boot kernel: SIS_FEATURES=\"llm,crypto-real\" BRINGUP=1 ./scripts/uefi_run.sh"
echo "  2. In shell, run each command and observe outputs"
echo "  3. After 'autoctl on', wait ~30 seconds before checking stats"
echo ""
