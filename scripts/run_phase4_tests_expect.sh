#!/bin/bash
# Phase 4 Solidification Test Suite Runner with Expect
# Orchestrates all validation tests using expect for QEMU interaction
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

# Check if expect is installed
if ! command -v expect &> /dev/null; then
    log_error "expect is not installed"
    log_error "Install with: brew install expect"
    exit 1
fi

# Display usage
usage() {
    cat << EOF
Phase 4 Solidification Test Suite (Expect-based Automation)

Usage: $0 [options] <test-suite>

Test Suites:
  quick       Quick validation (AI verification only)
  standard    Standard testing (AI + benchmarks)
  compliance  Compliance testing only
  full        Full validation (AI + benchmarks + compliance)

Options:
  -d DURATION Set benchmark duration in seconds (default: 15)
  -h, --help  Show this help message

Examples:
  $0 quick              Run quick AI verification
  $0 standard           Run standard benchmarks
  $0 -d 30 full         Run full suite with 30s benchmarks

EOF
}

# Quick validation suite
run_quick_suite() {
    log_section "Running Quick Validation Suite (Expect)"
    echo ""

    log_info "Step 1/1: AI Activity Verification"
    "$SCRIPT_DIR/verify_ai_active_expect.sh" || {
        log_error "AI verification failed"
        return 1
    }

    log_info "Quick suite complete"
}

# Standard test suite
run_standard_suite() {
    local duration=${1:-15}

    log_section "Running Standard Test Suite (Expect)"
    echo ""

    log_info "Step 1/2: AI Activity Verification"
    "$SCRIPT_DIR/verify_ai_active_expect.sh" || {
        log_error "AI verification failed"
        return 1
    }

    echo ""
    log_info "Waiting 30s before benchmark suite..."
    sleep 30

    log_info "Step 2/2: Benchmark Suite"
    "$SCRIPT_DIR/benchmark_suite_expect.sh" "$duration" || {
        log_error "Benchmark suite failed"
        return 1
    }

    log_info "Standard suite complete"
}

# Compliance test suite
run_compliance_suite() {
    log_section "Running Compliance Test Suite (Expect)"
    echo ""

    log_info "Step 1/1: Compliance Suite"
    "$SCRIPT_DIR/compliance_suite_expect.sh" || {
        log_error "Compliance suite failed"
        return 1
    }

    log_info "Compliance suite complete"
}

# Full validation suite
run_full_suite() {
    local duration=${1:-15}

    log_section "Running Full Validation Suite (Expect)"
    echo ""

    log_info "Step 1/3: AI Activity Verification"
    "$SCRIPT_DIR/verify_ai_active_expect.sh" || {
        log_error "AI verification failed"
        return 1
    }

    echo ""
    log_info "Waiting 30s before benchmark suite..."
    sleep 30

    log_info "Step 2/3: Benchmark Suite"
    "$SCRIPT_DIR/benchmark_suite_expect.sh" "$duration" || {
        log_error "Benchmark suite failed"
        return 1
    }

    echo ""
    log_info "Waiting 30s before compliance suite..."
    sleep 30

    log_info "Step 3/3: Compliance Suite"
    "$SCRIPT_DIR/compliance_suite_expect.sh" || {
        log_error "Compliance suite failed"
        return 1
    }

    log_info "Full suite complete"
}

# Main execution
main() {
    local duration=15

    # Parse options
    while [[ $# -gt 0 ]]; do
        case "$1" in
            -d)
                duration="$2"
                shift 2
                ;;
            -h|--help)
                usage
                exit 0
                ;;
            quick|standard|compliance|full)
                local suite="$1"
                shift
                break
                ;;
            *)
                log_error "Unknown option: $1"
                echo ""
                usage
                exit 1
                ;;
        esac
    done

    if [ -z "$suite" ]; then
        usage
        exit 1
    fi

    case "$suite" in
        quick)
            run_quick_suite
            ;;
        standard)
            run_standard_suite "$duration"
            ;;
        compliance)
            run_compliance_suite
            ;;
        full)
            run_full_suite "$duration"
            ;;
        *)
            log_error "Unknown test suite: $suite"
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
