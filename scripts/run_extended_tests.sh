#!/bin/bash
# Extended Duration Testing Suite Runner
# Orchestrates long-duration stability and performance tests
# NO EMOJIS - ASCII only - Industry standard shell script

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_section() {
    echo -e "${CYAN}[SECTION]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Display usage
usage() {
    cat << EOF
Extended Duration Testing Suite

Usage: $0 <test-type>

Test Types:
  benchmark-5min      5-minute benchmark suite
  benchmark-15min     15-minute benchmark suite
  benchmark-1hr       1-hour benchmark suite

  memory-stress       Memory stress test (10min, 95% pressure)
  memory-stress-30min Memory stress test (30min, 95% pressure)

  autonomous-1hr      Autonomous control validation (1 hour)
  autonomous-4hr      Autonomous control validation (4 hours)
  autonomous-24hr     Autonomous control validation (24 hours)

  stability-24hr      Full stability test (24 hours)

Examples:
  $0 benchmark-5min       Run 5-minute benchmarks
  $0 memory-stress        Run 10-minute 95% memory stress test
  $0 autonomous-1hr       Run 1-hour autonomous validation

Duration Notes:
  - 5min tests: ~8 minutes total (including overhead)
  - 15min tests: ~18 minutes total
  - 1hr tests: ~65 minutes total
  - 4hr tests: ~4.1 hours total
  - 24hr tests: ~24.5 hours total

EOF
}

# Check if expect is installed
if ! command -v expect &> /dev/null; then
    log_error "expect is not installed"
    log_error "Install with: brew install expect"
    exit 1
fi

# Main execution
main() {
    if [ $# -eq 0 ]; then
        usage
        exit 1
    fi

    local test_type="$1"

    case "$test_type" in
        benchmark-5min)
            log_section "Running 5-Minute Benchmark Suite"
            echo ""
            "$SCRIPT_DIR/benchmark_suite_expect.sh" 300
            ;;

        benchmark-15min)
            log_section "Running 15-Minute Benchmark Suite"
            echo ""
            "$SCRIPT_DIR/benchmark_suite_expect.sh" 900
            ;;

        benchmark-1hr)
            log_section "Running 1-Hour Benchmark Suite"
            echo ""
            "$SCRIPT_DIR/benchmark_suite_expect.sh" 3600
            ;;

        memory-stress)
            log_section "Running Memory Stress Test (10min, 95%)"
            echo ""
            "$SCRIPT_DIR/memory_stress_expect.sh" 600000 95
            ;;

        memory-stress-30min)
            log_section "Running Memory Stress Test (30min, 95%)"
            echo ""
            "$SCRIPT_DIR/memory_stress_expect.sh" 1800000 95
            ;;

        autonomous-1hr)
            log_section "Running Autonomous Validation (1 hour)"
            echo ""
            "$SCRIPT_DIR/autonomous_validation_expect.sh" 3600
            ;;

        autonomous-4hr)
            log_section "Running Autonomous Validation (4 hours)"
            echo ""
            "$SCRIPT_DIR/autonomous_validation_expect.sh" 14400
            ;;

        autonomous-24hr)
            log_section "Running Autonomous Validation (24 hours)"
            echo ""
            "$SCRIPT_DIR/autonomous_validation_expect.sh" 86400
            ;;

        stability-24hr)
            log_section "Running 24-Hour Stability Test Suite"
            echo ""
            log_info "This will run:"
            log_info "  1. Autonomous validation (24hr)"
            log_info "  2. Memory stress test (1hr at 95%)"
            log_info "  3. Extended benchmark (1hr)"
            echo ""
            log_info "Starting 24-hour autonomous validation..."
            "$SCRIPT_DIR/autonomous_validation_expect.sh" 86400 || {
                log_error "24-hour autonomous validation failed"
                exit 1
            }

            echo ""
            log_info "Waiting 1 hour before memory stress test..."
            sleep 3600

            log_info "Starting 1-hour memory stress test..."
            "$SCRIPT_DIR/memory_stress_expect.sh" 3600000 95 || {
                log_error "Memory stress test failed"
                exit 1
            }

            echo ""
            log_info "Waiting 1 hour before final benchmark..."
            sleep 3600

            log_info "Starting 1-hour benchmark..."
            "$SCRIPT_DIR/benchmark_suite_expect.sh" 3600 || {
                log_error "Benchmark failed"
                exit 1
            }

            log_info "24-hour stability test suite complete"
            ;;

        *)
            log_error "Unknown test type: $test_type"
            echo ""
            usage
            exit 1
            ;;
    esac

    local exit_code=$?

    echo ""
    if [ $exit_code -eq 0 ]; then
        log_info "========================================"
        log_info "  Extended Test Complete: SUCCESS"
        log_info "========================================"
    else
        log_error "========================================"
        log_error "  Extended Test Complete: FAILED"
        log_error "========================================"
    fi

    exit $exit_code
}

main "$@"
