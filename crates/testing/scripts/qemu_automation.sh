#!/bin/bash
# SIS Kernel QEMU Test Environment Automation
# Industry-grade automated testing with QEMU virtual machines

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../../" && pwd)"
TEST_OUTPUT_DIR="$PROJECT_ROOT/target/testing/qemu"
LOG_DIR="$TEST_OUTPUT_DIR/logs"
RESULTS_DIR="$TEST_OUTPUT_DIR/results"

# Test configuration
DEFAULT_NODES=10
DEFAULT_TIMEOUT=300
DEFAULT_MEMORY="256M"
DEFAULT_CPU="4"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $(date '+%Y-%m-%d %H:%M:%S') $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $(date '+%Y-%m-%d %H:%M:%S') $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $(date '+%Y-%m-%d %H:%M:%S') $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $(date '+%Y-%m-%d %H:%M:%S') $1"
}

# Help function
show_help() {
    cat << EOF
SIS Kernel QEMU Test Environment Automation

USAGE:
    $0 [OPTIONS] [COMMAND]

COMMANDS:
    setup           Setup test environment and dependencies
    test            Run comprehensive test suite
    performance     Run performance benchmarks
    stress          Run stress testing
    distributed     Run distributed consensus tests
    security        Run security validation tests
    cleanup         Clean up test artifacts
    report          Generate test reports
    help            Show this help message

OPTIONS:
    -n, --nodes NUM     Number of QEMU nodes to spawn (default: $DEFAULT_NODES)
    -t, --timeout SEC   Test timeout in seconds (default: $DEFAULT_TIMEOUT)
    -m, --memory SIZE   Memory per QEMU instance (default: $DEFAULT_MEMORY)
    -c, --cpus NUM      CPU cores per QEMU instance (default: $DEFAULT_CPU)
    -o, --output DIR    Output directory (default: $TEST_OUTPUT_DIR)
    -v, --verbose       Verbose output
    -q, --quiet         Quiet output
    -h, --help          Show this help message

EXAMPLES:
    $0 setup                           # Setup test environment
    $0 test --nodes 5 --timeout 600    # Run tests with 5 nodes, 10min timeout
    $0 performance --verbose           # Run performance tests with verbose output
    $0 distributed --nodes 20          # Run distributed tests with 20 nodes
    $0 cleanup                         # Clean up all test artifacts

ENVIRONMENT VARIABLES:
    SIS_QEMU_PATH       Path to QEMU binary (auto-detected if not set)
    SIS_TEST_TIMEOUT    Default test timeout
    SIS_TEST_NODES      Default number of nodes
    SIS_VERBOSE         Enable verbose output (1/true/yes)
    SIS_QUIET           Enable quiet output (1/true/yes)

EOF
}

# Parse command line arguments
parse_args() {
    NODES="${SIS_TEST_NODES:-$DEFAULT_NODES}"
    TIMEOUT="${SIS_TEST_TIMEOUT:-$DEFAULT_TIMEOUT}"
    MEMORY="$DEFAULT_MEMORY"
    CPUS="$DEFAULT_CPU"
    OUTPUT_DIR="$TEST_OUTPUT_DIR"
    VERBOSE="${SIS_VERBOSE:-0}"
    QUIET="${SIS_QUIET:-0}"
    COMMAND=""

    while [[ $# -gt 0 ]]; do
        case $1 in
            -n|--nodes)
                NODES="$2"
                shift 2
                ;;
            -t|--timeout)
                TIMEOUT="$2"
                shift 2
                ;;
            -m|--memory)
                MEMORY="$2"
                shift 2
                ;;
            -c|--cpus)
                CPUS="$2"
                shift 2
                ;;
            -o|--output)
                OUTPUT_DIR="$2"
                shift 2
                ;;
            -v|--verbose)
                VERBOSE=1
                shift
                ;;
            -q|--quiet)
                QUIET=1
                shift
                ;;
            -h|--help)
                show_help
                exit 0
                ;;
            setup|test|performance|stress|distributed|security|cleanup|report|help)
                COMMAND="$1"
                shift
                ;;
            *)
                log_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done

    if [[ -z "$COMMAND" ]]; then
        log_error "No command specified"
        show_help
        exit 1
    fi

    # Setup logging based on verbosity
    if [[ "$QUIET" == "1" ]]; then
        exec 3>&1 4>&2 >/dev/null 2>&1
    elif [[ "$VERBOSE" == "1" ]]; then
        set -x
    fi
}

# Environment setup
setup_environment() {
    log_info "Setting up SIS Kernel QEMU test environment"
    
    # Create directories
    mkdir -p "$OUTPUT_DIR" "$LOG_DIR" "$RESULTS_DIR"
    
    # Detect QEMU
    if [[ -z "${SIS_QEMU_PATH:-}" ]]; then
        if command -v qemu-system-x86_64 >/dev/null 2>&1; then
            QEMU_PATH="qemu-system-x86_64"
        elif command -v qemu-system-aarch64 >/dev/null 2>&1; then
            QEMU_PATH="qemu-system-aarch64"
        else
            log_error "QEMU not found. Please install QEMU or set SIS_QEMU_PATH"
            exit 1
        fi
    else
        QEMU_PATH="$SIS_QEMU_PATH"
    fi
    
    log_info "Using QEMU: $QEMU_PATH"
    
    # Check if kernel binary exists
    if [[ ! -f "$PROJECT_ROOT/target/x86_64-unknown-uefi/debug/sis-kernel.efi" ]] && 
       [[ ! -f "$PROJECT_ROOT/scripts/esp/EFI/SIS/KERNEL.ELF" ]]; then
        log_warning "Kernel binary not found. Building kernel..."
        cd "$PROJECT_ROOT"
        ./scripts/build.sh || {
            log_error "Failed to build kernel"
            exit 1
        }
    fi
    
    log_success "Environment setup completed"
}

# Start QEMU instance
start_qemu_node() {
    local node_id="$1"
    local log_file="$LOG_DIR/node_${node_id}.log"
    local monitor_socket="$OUTPUT_DIR/monitor_${node_id}.sock"
    local serial_port=$((10000 + node_id))
    
    log_info "Starting QEMU node $node_id"
    
    # QEMU command
    local qemu_cmd=(
        "$QEMU_PATH"
        -machine q35
        -cpu host
        -smp "$CPUS"
        -m "$MEMORY"
        -drive "if=pflash,format=raw,readonly=on,file=$PROJECT_ROOT/scripts/OVMF_CODE.fd"
        -drive "if=pflash,format=raw,file=$OUTPUT_DIR/OVMF_VARS_${node_id}.fd"
        -drive "format=raw,file=fat:rw:$PROJECT_ROOT/scripts/esp"
        -netdev "user,id=net0,hostfwd=tcp::${serial_port}-:22"
        -device "virtio-net-pci,netdev=net0"
        -monitor "unix:$monitor_socket,server,nowait"
        -serial "file:$log_file"
        -display none
        -daemonize
    )
    
    # Copy OVMF variables template
    cp "$PROJECT_ROOT/scripts/OVMF_VARS.fd" "$OUTPUT_DIR/OVMF_VARS_${node_id}.fd" 2>/dev/null || {
        log_warning "OVMF_VARS.fd not found, creating dummy file"
        touch "$OUTPUT_DIR/OVMF_VARS_${node_id}.fd"
    }
    
    # Start QEMU
    if "${qemu_cmd[@]}" 2>&1; then
        log_success "QEMU node $node_id started (port: $serial_port, log: $log_file)"
        echo "$serial_port" > "$OUTPUT_DIR/node_${node_id}.port"
        return 0
    else
        log_error "Failed to start QEMU node $node_id"
        return 1
    fi
}

# Stop QEMU instance
stop_qemu_node() {
    local node_id="$1"
    local monitor_socket="$OUTPUT_DIR/monitor_${node_id}.sock"
    
    if [[ -S "$monitor_socket" ]]; then
        echo "quit" | nc -U "$monitor_socket" 2>/dev/null || true
        log_info "Stopped QEMU node $node_id"
    fi
    
    # Cleanup
    rm -f "$OUTPUT_DIR/OVMF_VARS_${node_id}.fd"
    rm -f "$OUTPUT_DIR/node_${node_id}.port"
}

# Wait for kernel boot
wait_for_kernel_boot() {
    local node_id="$1"
    local log_file="$LOG_DIR/node_${node_id}.log"
    local timeout=60
    local elapsed=0
    
    log_info "Waiting for kernel boot on node $node_id"
    
    while [[ $elapsed -lt $timeout ]]; do
        if grep -q "SIS Kernel Ready" "$log_file" 2>/dev/null; then
            log_success "Kernel booted on node $node_id"
            return 0
        fi
        
        if grep -q "PANIC\|ERROR\|FATAL" "$log_file" 2>/dev/null; then
            log_error "Kernel panic detected on node $node_id"
            return 1
        fi
        
        sleep 2
        elapsed=$((elapsed + 2))
    done
    
    log_error "Kernel boot timeout on node $node_id"
    return 1
}

# Run test command on node
run_test_on_node() {
    local node_id="$1"
    local test_command="$2"
    local log_file="$LOG_DIR/node_${node_id}_test.log"
    local monitor_socket="$OUTPUT_DIR/monitor_${node_id}.sock"
    
    log_info "Running test on node $node_id: $test_command"
    
    # Send command via monitor
    echo "sendkey ret" | nc -U "$monitor_socket" 2>/dev/null
    sleep 1
    echo "type $test_command" | nc -U "$monitor_socket" 2>/dev/null
    sleep 1
    echo "sendkey ret" | nc -U "$monitor_socket" 2>/dev/null
    
    # Wait for results
    sleep 5
    
    # Extract results from log
    tail -n 50 "$LOG_DIR/node_${node_id}.log" > "$log_file"
    
    if grep -q "TEST PASSED\|SUCCESS" "$log_file" 2>/dev/null; then
        log_success "Test passed on node $node_id"
        return 0
    else
        log_error "Test failed on node $node_id"
        return 1
    fi
}

# Performance test suite
run_performance_tests() {
    log_info "Starting performance test suite with $NODES nodes"
    
    local start_time=$(date +%s)
    local passed=0
    local failed=0
    
    # Start all nodes
    for ((i=1; i<=NODES; i++)); do
        start_qemu_node "$i" &
    done
    
    wait
    log_info "All nodes started, waiting for boot completion"
    
    # Wait for all nodes to boot
    for ((i=1; i<=NODES; i++)); do
        if ! wait_for_kernel_boot "$i"; then
            log_error "Node $i failed to boot"
            ((failed++))
            continue
        fi
    done
    
    # Run performance benchmarks
    log_info "Running AI inference benchmarks"
    for ((i=1; i<=NODES; i++)); do
        if run_test_on_node "$i" "perf_test ai_inference"; then
            ((passed++))
        else
            ((failed++))
        fi
    done
    
    log_info "Running context switch benchmarks"
    for ((i=1; i<=NODES; i++)); do
        if run_test_on_node "$i" "perf_test context_switch"; then
            ((passed++))
        else
            ((failed++))
        fi
    done
    
    log_info "Running memory allocation benchmarks"
    for ((i=1; i<=NODES; i++)); do
        if run_test_on_node "$i" "perf_test memory_alloc"; then
            ((passed++))
        else
            ((failed++))
        fi
    done
    
    # Cleanup
    for ((i=1; i<=NODES; i++)); do
        stop_qemu_node "$i"
    done
    
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    log_info "Performance tests completed in ${duration}s"
    log_info "Results: $passed passed, $failed failed"
    
    # Generate results file
    cat > "$RESULTS_DIR/performance_results.json" << EOF
{
    "test_suite": "performance",
    "start_time": $start_time,
    "end_time": $end_time,
    "duration": $duration,
    "nodes": $NODES,
    "tests_passed": $passed,
    "tests_failed": $failed,
    "success_rate": $(echo "scale=4; $passed / ($passed + $failed)" | bc -l 2>/dev/null || echo "0")
}
EOF
    
    return $(( failed > 0 ? 1 : 0 ))
}

# Distributed consensus tests
run_distributed_tests() {
    log_info "Starting distributed consensus tests with $NODES nodes"
    
    # Distributed tests require at least 4 nodes for BFT
    if [[ $NODES -lt 4 ]]; then
        log_error "Distributed tests require at least 4 nodes (current: $NODES)"
        return 1
    fi
    
    local start_time=$(date +%s)
    local passed=0
    local failed=0
    
    # Start all nodes
    for ((i=1; i<=NODES; i++)); do
        start_qemu_node "$i" &
    done
    
    wait
    
    # Wait for all nodes to boot
    for ((i=1; i<=NODES; i++)); do
        if ! wait_for_kernel_boot "$i"; then
            log_error "Node $i failed to boot"
            ((failed++))
        fi
    done
    
    # Run consensus tests
    log_info "Testing Byzantine consensus with f=$(($NODES/3)) Byzantine nodes"
    
    # Start consensus on all nodes
    for ((i=1; i<=NODES; i++)); do
        run_test_on_node "$i" "consensus_test start_node $i $NODES" &
    done
    
    wait
    
    # Check consensus results
    for ((i=1; i<=NODES; i++)); do
        if grep -q "CONSENSUS_SUCCESS" "$LOG_DIR/node_${i}_test.log" 2>/dev/null; then
            ((passed++))
        else
            ((failed++))
        fi
    done
    
    # Cleanup
    for ((i=1; i<=NODES; i++)); do
        stop_qemu_node "$i"
    done
    
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    log_info "Distributed tests completed in ${duration}s"
    log_info "Results: $passed passed, $failed failed"
    
    cat > "$RESULTS_DIR/distributed_results.json" << EOF
{
    "test_suite": "distributed",
    "start_time": $start_time,
    "end_time": $end_time,
    "duration": $duration,
    "nodes": $NODES,
    "byzantine_tolerance": $(($NODES/3)),
    "tests_passed": $passed,
    "tests_failed": $failed,
    "success_rate": $(echo "scale=4; $passed / ($passed + $failed)" | bc -l 2>/dev/null || echo "0")
}
EOF
    
    return $(( failed > 0 ? 1 : 0 ))
}

# Comprehensive test suite
run_comprehensive_tests() {
    log_info "Starting comprehensive test suite"
    
    setup_environment
    
    local overall_result=0
    
    # Run performance tests
    if run_performance_tests; then
        log_success "Performance tests passed"
    else
        log_error "Performance tests failed"
        overall_result=1
    fi
    
    # Run distributed tests if we have enough nodes
    if [[ $NODES -ge 4 ]]; then
        if run_distributed_tests; then
            log_success "Distributed tests passed"
        else
            log_error "Distributed tests failed"
            overall_result=1
        fi
    else
        log_warning "Skipping distributed tests (need ≥4 nodes, have $NODES)"
    fi
    
    return $overall_result
}

# Cleanup function
cleanup_environment() {
    log_info "Cleaning up test environment"
    
    # Stop any running QEMU instances
    for ((i=1; i<=50; i++)); do
        stop_qemu_node "$i" 2>/dev/null || true
    done
    
    # Kill any orphaned QEMU processes
    pkill -f "qemu.*sis-kernel" 2>/dev/null || true
    
    # Remove temporary files
    rm -rf "$OUTPUT_DIR"/*.sock
    rm -rf "$OUTPUT_DIR"/*.fd
    rm -rf "$OUTPUT_DIR"/*.port
    
    log_success "Cleanup completed"
}

# Generate test report
generate_report() {
    log_info "Generating test report"
    
    local report_file="$RESULTS_DIR/test_report.html"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    
    cat > "$report_file" << EOF
<!DOCTYPE html>
<html>
<head>
    <title>SIS Kernel QEMU Test Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        .header { background: #f5f5f5; padding: 20px; border-radius: 8px; }
        .results { margin: 20px 0; }
        .pass { color: #28a745; }
        .fail { color: #dc3545; }
        .warning { color: #ffc107; }
    </style>
</head>
<body>
    <div class="header">
        <h1>SIS Kernel QEMU Test Report</h1>
        <p>Generated: $timestamp</p>
        <p>Test Configuration: $NODES nodes, ${TIMEOUT}s timeout</p>
    </div>
    
    <div class="results">
        <h2>Test Results</h2>
EOF
    
    # Add performance results if available
    if [[ -f "$RESULTS_DIR/performance_results.json" ]]; then
        local perf_passed=$(jq -r '.tests_passed' "$RESULTS_DIR/performance_results.json" 2>/dev/null || echo "0")
        local perf_failed=$(jq -r '.tests_failed' "$RESULTS_DIR/performance_results.json" 2>/dev/null || echo "0")
        
        cat >> "$report_file" << EOF
        <h3>Performance Tests</h3>
        <p>Passed: <span class="pass">$perf_passed</span> | Failed: <span class="fail">$perf_failed</span></p>
EOF
    fi
    
    # Add distributed results if available
    if [[ -f "$RESULTS_DIR/distributed_results.json" ]]; then
        local dist_passed=$(jq -r '.tests_passed' "$RESULTS_DIR/distributed_results.json" 2>/dev/null || echo "0")
        local dist_failed=$(jq -r '.tests_failed' "$RESULTS_DIR/distributed_results.json" 2>/dev/null || echo "0")
        
        cat >> "$report_file" << EOF
        <h3>Distributed Tests</h3>
        <p>Passed: <span class="pass">$dist_passed</span> | Failed: <span class="fail">$dist_failed</span></p>
EOF
    fi
    
    cat >> "$report_file" << EOF
    </div>
    
    <div class="logs">
        <h2>Log Files</h2>
        <ul>
EOF
    
    # Add log file links
    for log_file in "$LOG_DIR"/*.log; do
        if [[ -f "$log_file" ]]; then
            local basename=$(basename "$log_file")
            echo "            <li><a href=\"../logs/$basename\">$basename</a></li>" >> "$report_file"
        fi
    done
    
    cat >> "$report_file" << EOF
        </ul>
    </div>
</body>
</html>
EOF
    
    log_success "Test report generated: $report_file"
}

# Main execution
main() {
    parse_args "$@"
    
    case "$COMMAND" in
        setup)
            setup_environment
            ;;
        test)
            run_comprehensive_tests
            ;;
        performance)
            setup_environment
            run_performance_tests
            ;;
        distributed)
            setup_environment
            run_distributed_tests
            ;;
        stress)
            log_warning "Stress testing not yet implemented"
            exit 1
            ;;
        security)
            log_warning "Security testing not yet implemented"
            exit 1
            ;;
        cleanup)
            cleanup_environment
            ;;
        report)
            generate_report
            ;;
        help)
            show_help
            ;;
        *)
            log_error "Unknown command: $COMMAND"
            show_help
            exit 1
            ;;
    esac
}

# Trap cleanup on exit
trap 'cleanup_environment 2>/dev/null || true' EXIT INT TERM

# Execute main function
main "$@"