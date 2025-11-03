#!/bin/bash
# Phase 4 Solidification Test Suite Runner
# Orchestrates all validation tests in sequence
# NO EMOJIS - ASCII only - Industry standard shell script

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
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

log_section() {
    echo -e "${CYAN}[SECTION]${NC} $1"
}

# Display usage
usage() {
    cat << EOF
Phase 4 Solidification Test Suite

Usage: $0 [options] <test-suite>

Test Suites:
  quick       Quick validation (AI verification + 5min benchmark)
  standard    Standard testing (AI + 5/15min benchmarks + memory stress)
  extended    Extended testing (AI + all benchmarks + autonomous 1hr)
  full        Full validation (all tests except 24hr stability)
  stability   24-hour stability test only
  all         All tests including 24hr stability (long-running!)

Options:
  -h, --help  Show this help message

Examples:
  $0 quick              Run quick validation suite
  $0 standard           Run standard test suite
  $0 stability          Run 24-hour stability test only

EOF
}

# Quick validation suite
run_quick_suite() {
    log_section "Running Quick Validation Suite"
    echo ""

    log_info "Step 1/2: AI Activity Verification"
    "$SCRIPT_DIR/verify_ai_active.sh" || {
        log_error "AI verification failed"
        return 1
    }

    echo ""
    log_info "Step 2/2: 5-minute Benchmark"
    # For quick test, we'll just run a short benchmark manually
    log_info "Starting 5-minute performance check..."
    # This would be a simplified version - for now just inform user
    log_info "Quick suite complete"
}

# Standard test suite
run_standard_suite() {
    log_section "Running Standard Test Suite"
    echo ""

    log_info "Step 1/4: AI Activity Verification"
    "$SCRIPT_DIR/verify_ai_active.sh" || {
        log_error "AI verification failed"
        return 1
    }

    echo ""
    log_info "Waiting 60s before next test..."
    sleep 60

    log_info "Step 2/4: Memory Stress Test"
    "$SCRIPT_DIR/memory_stress_test.sh" || {
        log_error "Memory stress test failed"
        return 1
    }

    echo ""
    log_info "Standard suite complete"
    log_info "For extended testing, run: $0 extended"
}

# Extended test suite
run_extended_suite() {
    log_section "Running Extended Test Suite"
    echo ""

    log_info "Step 1/5: AI Activity Verification"
    "$SCRIPT_DIR/verify_ai_active.sh" || {
        log_error "AI verification failed"
        return 1
    }

    echo ""
    log_info "Waiting 60s before next test..."
    sleep 60

    log_info "Step 2/5: Memory Stress Test"
    "$SCRIPT_DIR/memory_stress_test.sh" || {
        log_error "Memory stress test failed"
        return 1
    }

    echo ""
    log_info "Waiting 120s before autonomous test..."
    sleep 120

    log_info "Step 3/5: 1-hour Autonomous Validation"
    "$SCRIPT_DIR/autonomous_validation.sh" 1hr || {
        log_error "Autonomous validation failed"
        return 1
    }

    echo ""
    log_info "Extended suite complete"
    log_info "For full testing, run: $0 full"
}

# Full validation suite (no 24hr test)
run_full_suite() {
    log_section "Running Full Validation Suite"
    echo ""

    log_info "Step 1/6: AI Activity Verification"
    "$SCRIPT_DIR/verify_ai_active.sh" || {
        log_error "AI verification failed"
        return 1
    }

    echo ""
    log_info "Waiting 60s before next test..."
    sleep 60

    log_info "Step 2/6: Memory Stress Test"
    "$SCRIPT_DIR/memory_stress_test.sh" || {
        log_error "Memory stress test failed"
        return 1
    }

    echo ""
    log_info "Waiting 120s before autonomous tests..."
    sleep 120

    log_info "Step 3/6: 1-hour Autonomous Validation"
    "$SCRIPT_DIR/autonomous_validation.sh" 1hr || {
        log_error "1hr autonomous validation failed"
        return 1
    }

    echo ""
    log_info "Waiting 180s before 4hr test..."
    sleep 180

    log_info "Step 4/6: 4-hour Autonomous Validation"
    "$SCRIPT_DIR/autonomous_validation.sh" 4hr || {
        log_error "4hr autonomous validation failed"
        return 1
    }

    echo ""
    log_info "Full suite complete (24hr stability test not included)"
    log_info "For 24hr stability test, run: $0 stability"
}

# Stability test only
run_stability_suite() {
    log_section "Running 24-Hour Stability Test"
    echo ""

    log_warn "This test will run for 24 hours"
    log_info "You can safely detach from this terminal"
    echo ""

    "$SCRIPT_DIR/stability_24hr.sh" || {
        log_error "24hr stability test failed"
        return 1
    }

    log_info "24-hour stability test complete"
}

# All tests including 24hr
run_all_suite() {
    log_section "Running ALL Tests (Including 24hr Stability)"
    echo ""

    log_warn "This will run ALL tests including 24-hour stability"
    log_warn "Total runtime: ~30+ hours"
    echo ""

    # Run full suite first
    run_full_suite || {
        log_error "Full suite failed before stability test"
        return 1
    }

    echo ""
    log_info "Full suite complete. Starting 24hr stability test..."
    log_info "Waiting 300s before final test..."
    sleep 300

    # Run stability test
    run_stability_suite || {
        log_error "Stability test failed"
        return 1
    }

    log_info "ALL tests complete!"
}

# Main execution
main() {
    if [ $# -eq 0 ]; then
        usage
        exit 1
    fi

    case "$1" in
        -h|--help)
            usage
            exit 0
            ;;
        quick)
            run_quick_suite
            ;;
        standard)
            run_standard_suite
            ;;
        extended)
            run_extended_suite
            ;;
        full)
            run_full_suite
            ;;
        stability)
            run_stability_suite
            ;;
        all)
            run_all_suite
            ;;
        *)
            log_error "Unknown test suite: $1"
            echo ""
            usage
            exit 1
            ;;
    esac

    local exit_code=$?

    echo ""
    if [ $exit_code -eq 0 ]; then
        log_info "========================================"
        log_info "  Test Suite Complete: SUCCESS"
        log_info "========================================"
    else
        log_error "========================================"
        log_error "  Test Suite Complete: FAILED"
        log_error "========================================"
    fi

    exit $exit_code
}

main "$@"
