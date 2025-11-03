#!/bin/bash
# Comprehensive Compliance Suite with Expect
# Runs all Week 12 compliance commands programmatically
# NO EMOJIS - ASCII only - Industry standard shell script

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
RESULTS_DIR="$PROJECT_ROOT/compliance_results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
OUTPUT_FILE="$RESULTS_DIR/compliance_suite_${TIMESTAMP}.log"

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

# Main test execution
main() {
    log_info "========================================"
    log_info "  Comprehensive Compliance Suite"
    log_info "========================================"
    log_info "Output: $OUTPUT_FILE"
    echo ""

    log_info "Starting QEMU with expect automation..."

    # Create expect script
    cat > /tmp/compliance_expect_$$.exp <<'EXPECT_EOF'
#!/usr/bin/expect -f
set timeout 120

# Start QEMU
spawn env SIS_FEATURES=llm,crypto-real BRINGUP=1 ./scripts/uefi_run.sh build

# Wait for shell prompt
expect {
    "sis>" {
        send_user "\n\[EXPECT\] Shell ready\n"
    }
    timeout {
        send_user "\n\[EXPECT\] Timeout waiting for shell\n"
        exit 1
    }
}

# Test 1: EU AI Act Compliance
send_user "\n\[EXPECT\] ========== TEST 1: EU AI Act Compliance ==========\n"
send "compliance eu-ai-act\r"
expect {
    "OVERALL COMPLIANCE SCORE" {
        send_user "\n\[EXPECT\] EU AI Act report completed\n"
    }
    timeout {
        send_user "\n\[EXPECT\] EU AI Act timeout\n"
    }
}
expect "sis>"

# Test 2: Audit package
send_user "\n\[EXPECT\] ========== TEST 2: Audit Package ==========\n"
send "compliance audit\r"
expect {
    "Package ready for third-party review" {
        send_user "\n\[EXPECT\] Audit package generated\n"
    }
    timeout {
        send_user "\n\[EXPECT\] Audit timeout\n"
    }
}
expect "sis>"

# Test 3: Transparency report
send_user "\n\[EXPECT\] ========== TEST 3: Transparency Report ==========\n"
send "compliance transparency\r"
expect {
    "Report generated for stakeholder review" {
        send_user "\n\[EXPECT\] Transparency report generated\n"
    }
    timeout {
        send_user "\n\[EXPECT\] Transparency timeout\n"
    }
}
expect "sis>"

# Test 4: Safety checklist
send_user "\n\[EXPECT\] ========== TEST 4: Safety Checklist ==========\n"
send "compliance checklist\r"
expect {
    "Production Ready:" {
        send_user "\n\[EXPECT\] Safety checklist completed\n"
    }
    timeout {
        send_user "\n\[EXPECT\] Checklist timeout\n"
    }
}
expect "sis>"

# Test 5: Incident log
send_user "\n\[EXPECT\] ========== TEST 5: Incident Log ==========\n"
send "compliance incidents\r"
expect {
    "Total Incidents:" {
        send_user "\n\[EXPECT\] Incident log retrieved\n"
    }
    timeout {
        send_user "\n\[EXPECT\] Incidents timeout\n"
    }
}
expect "sis>"

# Exit QEMU
send_user "\n\[EXPECT\] All compliance tests complete. Exiting...\n"
send "\x01"
send "x"

expect eof
EXPECT_EOF

    chmod +x /tmp/compliance_expect_$$.exp

    # Run expect script
    cd "$PROJECT_ROOT"
    /tmp/compliance_expect_$$.exp 2>&1 | tee "$OUTPUT_FILE"
    local exit_code=${PIPESTATUS[0]}

    # Cleanup
    rm -f /tmp/compliance_expect_$$.exp

    # Analysis
    echo ""
    log_info "========================================"
    log_info "  Compliance Results Analysis"
    log_info "========================================"

    # Extract key metrics (trim whitespace)
    # Note: Some metrics may be on the next line, so use grep -A to get context
    # Filter out [EXPECT] lines that get inserted by expect script
    local compliance_score=$(grep -A 2 "OVERALL COMPLIANCE SCORE:" "$OUTPUT_FILE" | grep -v "\[EXPECT\]" | tr -d '\n' | grep -o "[0-9]\+%" | head -1 | tr -d '%[:space:]' || echo "0")
    local safety_score=$(grep "Safety score:" "$OUTPUT_FILE" | head -1 | grep -o "[0-9]\+/100" | cut -d'/' -f1 | tr -d '[:space:]' || echo "0")
    local checklist_completion=$(grep "Completion:" "$OUTPUT_FILE" | tail -1 | grep -o "[0-9]\+%" | tr -d '%[:space:]' || echo "0")
    local critical_incidents=$(grep "Critical:" "$OUTPUT_FILE" | grep -o "Critical: [0-9]\+" | head -1 | awk '{print $2}' | tr -d '[:space:]' || echo "0")
    local production_ready=$(grep -c "\[OK\] YES" "$OUTPUT_FILE" | tr -d '[:space:]' || echo "0")

    # Ensure we have valid numbers (default to 0 if empty)
    compliance_score=${compliance_score:-0}
    safety_score=${safety_score:-0}
    checklist_completion=${checklist_completion:-0}
    critical_incidents=${critical_incidents:-0}
    production_ready=${production_ready:-0}

    echo "EU AI Act Compliance:         $compliance_score%"
    echo "Safety Score:                 $safety_score/100"
    echo "Checklist Completion:         $checklist_completion%"
    echo "Critical Incidents:           $critical_incidents"
    echo "Production Ready:             $([ "$production_ready" -gt 0 ] && echo "YES" || echo "NO")"
    echo ""

    # Article-level compliance (check if any [OK] appears under each article)
    local article13=$(grep -A 5 "Article 13:" "$OUTPUT_FILE" | grep -c "\[OK\]" || echo "0")
    local article14=$(grep -A 5 "Article 14:" "$OUTPUT_FILE" | grep -c "\[OK\]" || echo "0")
    local article15=$(grep -A 10 "Article 15:" "$OUTPUT_FILE" | grep -c "\[OK\]" || echo "0")
    local article16=$(grep -A 5 "Article 16:" "$OUTPUT_FILE" | grep -c "\[OK\]" || echo "0")

    echo "Article Compliance:"
    echo "  Article 13 (Transparency):   $([ "$article13" -gt 0 ] && echo "PASS" || echo "FAIL")"
    echo "  Article 14 (Human Oversight):$([ "$article14" -gt 0 ] && echo "PASS" || echo "FAIL")"
    echo "  Article 15 (Accuracy/Robust):$([ "$article15" -gt 0 ] && echo "PASS" || echo "FAIL")"
    echo "  Article 16 (Recordkeeping):  $([ "$article16" -gt 0 ] && echo "PASS" || echo "FAIL")"
    echo ""

    # Validation checks
    log_info "Validation Checks:"
    echo ""

    local pass_count=0
    local fail_count=0

    # Check 1: All compliance tests completed
    local test_count=$(grep -c "\[EXPECT\] .*completed\|generated\|retrieved" "$OUTPUT_FILE" || echo "0")
    if [ "$test_count" -ge 5 ]; then
        log_pass "All compliance tests completed ($test_count/5)"
        pass_count=$((pass_count + 1))
    else
        log_warn "Some compliance tests incomplete ($test_count/5)"
    fi

    # Check 2: Overall compliance score
    if [ "$compliance_score" -ge 85 ] 2>/dev/null; then
        log_pass "EU AI Act compliance: $compliance_score% (>= 85% required)"
        pass_count=$((pass_count + 1))
    else
        log_fail "EU AI Act compliance: $compliance_score% (< 85%)"
        fail_count=$((fail_count + 1))
    fi

    # Check 3: Safety score
    if [ "$safety_score" -ge 90 ] 2>/dev/null; then
        log_pass "Safety score: $safety_score/100 (>= 90 required)"
        pass_count=$((pass_count + 1))
    else
        log_fail "Safety score: $safety_score/100 (< 90)"
        fail_count=$((fail_count + 1))
    fi

    # Check 4: Checklist completion
    if [ "$checklist_completion" -ge 90 ] 2>/dev/null; then
        log_pass "Safety checklist: $checklist_completion% complete"
        pass_count=$((pass_count + 1))
    else
        log_warn "Safety checklist: $checklist_completion% complete"
    fi

    # Check 5: Critical incidents
    if [ "$critical_incidents" -eq 0 ] 2>/dev/null; then
        log_pass "No critical incidents"
        pass_count=$((pass_count + 1))
    else
        log_fail "Critical incidents detected: $critical_incidents"
        fail_count=$((fail_count + 1))
    fi

    # Check 6: Production ready
    if [ "$production_ready" -gt 0 ] 2>/dev/null; then
        log_pass "System marked production ready"
        pass_count=$((pass_count + 1))
    else
        log_fail "System NOT production ready"
        fail_count=$((fail_count + 1))
    fi

    # Overall result
    echo ""
    echo "========================================"
    echo "Overall Result: $pass_count passed, $fail_count failed"
    echo ""

    if [ $fail_count -eq 0 ] && [ $pass_count -ge 5 ]; then
        log_pass "COMPLIANCE SUITE SUCCESSFUL"
        echo "========================================"
        echo ""
        echo "Key Achievements:"
        echo "  - EU AI Act: $compliance_score% compliant"
        echo "  - Safety: $safety_score/100"
        echo "  - Checklist: $checklist_completion% complete"
        echo "  - Critical incidents: $critical_incidents"
        echo "  - Production ready: YES"
        echo ""
        echo "Full results: $OUTPUT_FILE"
        exit 0
    else
        log_fail "COMPLIANCE SUITE FAILED"
        echo "========================================"
        echo ""
        echo "Review full log: $OUTPUT_FILE"
        exit 1
    fi
}

main "$@"
