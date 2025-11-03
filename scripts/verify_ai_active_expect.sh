#!/bin/bash
# AI Neural Network Activity Verification Script with Expect
# Uses expect to send commands to QEMU interactive shell
# NO EMOJIS - ASCII only - Industry standard shell script

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
RESULTS_DIR="$PROJECT_ROOT/ai_verification_results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
OUTPUT_FILE="$RESULTS_DIR/ai_verification_${TIMESTAMP}.log"

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_pass() {
    echo -e "${GREEN}[PASS]${NC} $1"
}

log_fail() {
    echo -e "${RED}[FAIL]${NC} $1"
}

# Create results directory
mkdir -p "$RESULTS_DIR"

# Check if expect is installed
if ! command -v expect &> /dev/null; then
    log_error "expect is not installed. Install with: brew install expect"
    exit 1
fi

# Main verification test
main() {
    log_info "========================================"
    log_info "  AI Neural Network Activity Verification"
    log_info "========================================"
    log_info "Output: $OUTPUT_FILE"
    echo ""

    log_info "Starting QEMU with expect automation..."

    # Create expect script
    cat > /tmp/ai_verify_expect_$$.exp <<'EXPECT_EOF'
#!/usr/bin/expect -f
set timeout 120

# Start QEMU
spawn env SIS_FEATURES=llm,crypto-real BRINGUP=1 ./scripts/uefi_run.sh build

# Wait for shell prompt
expect {
    "sis>" {
        send_user "\n\[EXPECT\] Shell prompt detected\n"
    }
    timeout {
        send_user "\n\[EXPECT\] Timeout waiting for shell\n"
        exit 1
    }
}

# Run fullautodemo to trigger AI activity
send_user "\n\[EXPECT\] Running fullautodemo...\n"
send "fullautodemo\r"

# Wait for demo to complete (with any key prompt)
expect {
    "Press any key to begin" {
        send_user "\n\[EXPECT\] Demo starting, sending keypress...\n"
        send "\r"
        exp_continue
    }
    "DEMO COMPLETE" {
        send_user "\n\[EXPECT\] Demo completed successfully\n"
    }
    "Autonomous mode re-enabled" {
        send_user "\n\[EXPECT\] Autonomy re-enabled\n"
    }
    timeout {
        send_user "\n\[EXPECT\] Demo timeout\n"
    }
}

# Wait for shell prompt
expect "sis>"

# Run autoctl status to get AI metrics
send_user "\n\[EXPECT\] Getting autonomy statistics...\n"
send "autoctl status\r"
expect "sis>"

# Run agentctl stats to get agent metrics
send_user "\n\[EXPECT\] Getting agent statistics...\n"
send "agentctl stats\r"
expect "sis>"

# Exit QEMU
send_user "\n\[EXPECT\] Exiting QEMU...\n"
send "\x01"
send "x"

expect eof
EXPECT_EOF

    chmod +x /tmp/ai_verify_expect_$$.exp

    # Run expect script
    cd "$PROJECT_ROOT"
    /tmp/ai_verify_expect_$$.exp 2>&1 | tee "$OUTPUT_FILE"
    local exit_code=${PIPESTATUS[0]}

    # Cleanup
    rm -f /tmp/ai_verify_expect_$$.exp

    # Analysis
    echo ""
    log_info "========================================"
    log_info "  AI Verification Results"
    log_info "========================================"

    # Extract metrics from output (trim whitespace)
    local nn_infer_count=$(grep "METRIC nn_infer_count=" "$OUTPUT_FILE" | tail -1 | cut -d'=' -f2 | tr -d '[:space:]' || echo "0")
    local ai_decisions=$(grep "Total AI decisions:" "$OUTPUT_FILE" | tail -1 | awk '{print $4}' | tr -d '[:space:]' || echo "0")
    local autonomy_level=$(grep "Autonomy level:" "$OUTPUT_FILE" | tail -1 | awk '{print $3}' | tr -d '[:space:]' || echo "0")
    local demo_complete=$(grep -c "DEMO COMPLETE" "$OUTPUT_FILE" || echo "0")

    # Ensure we have valid numbers (default to 0 if empty)
    nn_infer_count=${nn_infer_count:-0}
    ai_decisions=${ai_decisions:-0}
    autonomy_level=${autonomy_level:-0}
    demo_complete=${demo_complete:-0}

    echo "Neural Network Inferences: $nn_infer_count"
    echo "AI Decisions Made:         $ai_decisions"
    echo "Autonomy Level:            $autonomy_level"
    echo "Demo Completed:            $demo_complete times"
    echo ""

    # Validation checks
    log_info "Validation Checks:"
    echo ""

    local pass_count=0
    local fail_count=0

    # Check 1: Demo completed
    if [ "$demo_complete" -gt 0 ]; then
        log_pass "Full autonomous demo completed successfully"
        pass_count=$((pass_count + 1))
    else
        log_fail "Demo did not complete"
        fail_count=$((fail_count + 1))
    fi

    # Check 2: Neural network active
    if [ "$nn_infer_count" -gt 0 ] 2>/dev/null; then
        log_pass "Neural network generating inferences ($nn_infer_count total)"
        pass_count=$((pass_count + 1))
    else
        log_fail "Neural network NOT generating inferences (count: $nn_infer_count)"
        fail_count=$((fail_count + 1))
    fi

    # Check 3: Shell responded
    local shell_prompts=$(grep -c "sis>" "$OUTPUT_FILE" || echo "0")
    if [ "$shell_prompts" -gt 3 ]; then
        log_pass "Shell interaction successful ($shell_prompts prompts)"
        pass_count=$((pass_count + 1))
    else
        log_warn "Limited shell interaction ($shell_prompts prompts)"
    fi

    # Overall result
    echo ""
    echo "========================================"
    echo "Overall Result: $pass_count passed, $fail_count failed"

    if [ $fail_count -eq 0 ] && [ $pass_count -ge 2 ]; then
        log_pass "AI VERIFICATION SUCCESSFUL"
        echo "========================================"
        exit 0
    else
        log_fail "AI VERIFICATION FAILED"
        echo "========================================"
        echo ""
        echo "Troubleshooting:"
        echo "  1. Check full log: $OUTPUT_FILE"
        echo "  2. Verify SIS_FEATURES=llm is set"
        echo "  3. Check if demo timed out"
        exit 1
    fi
}

main "$@"
