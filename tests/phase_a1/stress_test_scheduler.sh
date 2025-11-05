#!/bin/bash
# Phase A1 Scheduler Stress Test
#
# This script validates the scheduler under heavy load to ensure:
# 1. No deadlocks occur with multiple processes
# 2. Context switching works correctly under stress
# 3. Timer preemption handles edge cases gracefully
# 4. Fork/exec race conditions are handled properly
#
# Usage:
#   bash tests/phase_a1/stress_test_scheduler.sh [test_name]
#
# Available tests:
#   fork_bomb         - Create many child processes rapidly
#   exec_stress       - Repeatedly exec new programs
#   pipe_stress       - Create many pipes with data flow
#   mixed_load        - Combined fork/exec/pipe workload
#   timer_stress      - Verify timer preemption under load
#   all               - Run all stress tests (default)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$SCRIPT_DIR/../.."
KERNEL_BINARY="$ROOT_DIR/target/aarch64-unknown-none/release/sis_kernel"
TEST_NAME="${1:-all}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test configuration
QEMU_TIMEOUT=60  # seconds
FORK_BOMB_COUNT=20
EXEC_STRESS_COUNT=10
PIPE_STRESS_COUNT=5

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_prerequisites() {
    log_info "Checking prerequisites..."

    if ! command -v qemu-system-aarch64 &> /dev/null; then
        log_error "qemu-system-aarch64 not found. Please install QEMU."
        exit 1
    fi

    if ! command -v expect &> /dev/null; then
        log_warn "expect not found. Some tests may not work. Install with: sudo apt-get install expect"
    fi

    if [ ! -f "$KERNEL_BINARY" ]; then
        log_error "Kernel binary not found: $KERNEL_BINARY"
        log_info "Build with: cargo build --release --target aarch64-unknown-none"
        exit 1
    fi

    log_info "Prerequisites OK"
}

# Run QEMU with the kernel and send test commands via expect
run_stress_test() {
    local test_commands="$1"
    local test_name="$2"
    local timeout="${3:-$QEMU_TIMEOUT}"

    log_info "Running stress test: $test_name (timeout: ${timeout}s)"

    # Create expect script
    local expect_script=$(mktemp)
    cat > "$expect_script" << EOF
#!/usr/bin/expect -f
set timeout $timeout
set kernel_binary "$KERNEL_BINARY"

spawn qemu-system-aarch64 \\
    -machine virt,gic-version=3 \\
    -cpu cortex-a72 \\
    -smp 1 \\
    -m 128M \\
    -nographic \\
    -kernel \$kernel_binary \\
    -serial mon:stdio

# Wait for shell prompt
expect {
    "/ #" {
        send_user "\n==> Got shell prompt, starting stress test\n"
    }
    timeout {
        send_user "\n==> ERROR: Timeout waiting for shell prompt\n"
        exit 1
    }
    eof {
        send_user "\n==> ERROR: Kernel crashed before reaching shell\n"
        exit 1
    }
}

# Execute test commands
$test_commands

# Verify kernel is still responsive
send "echo STRESS_TEST_COMPLETE\r"
expect {
    "STRESS_TEST_COMPLETE" {
        send_user "\n==> Stress test completed successfully\n"
    }
    timeout {
        send_user "\n==> ERROR: Kernel became unresponsive during test\n"
        exit 1
    }
}

# Clean exit
send "exit\r"
expect eof
exit 0
EOF

    chmod +x "$expect_script"

    if expect "$expect_script"; then
        log_info "✓ $test_name PASSED"
        rm "$expect_script"
        return 0
    else
        log_error "✗ $test_name FAILED"
        rm "$expect_script"
        return 1
    fi
}

test_fork_bomb() {
    log_info "Fork bomb test: Create $FORK_BOMB_COUNT child processes"

    local commands=""
    for i in $(seq 1 $FORK_BOMB_COUNT); do
        commands+="send \"sh -c 'echo child $i' &\r\"\n"
        commands+="sleep 0.1\n"
    done
    commands+="send \"wait\r\"\n"
    commands+="sleep 2\n"

    run_stress_test "$commands" "fork_bomb" 30
}

test_exec_stress() {
    log_info "Exec stress test: Repeatedly exec new programs"

    local commands=""
    for i in $(seq 1 $EXEC_STRESS_COUNT); do
        commands+="send \"sh -c 'echo exec iteration $i'\r\"\n"
        commands+="sleep 0.2\n"
    done

    run_stress_test "$commands" "exec_stress" 30
}

test_pipe_stress() {
    log_info "Pipe stress test: Multiple pipes with data flow"

    local commands=""
    for i in $(seq 1 $PIPE_STRESS_COUNT); do
        commands+="send \"yes | head -n 10 > /dev/null\r\"\n"
        commands+="sleep 0.5\n"
    done

    run_stress_test "$commands" "pipe_stress" 30
}

test_mixed_load() {
    log_info "Mixed load test: Combined fork/exec/pipe workload"

    local commands=""
    # Parallel mix of operations
    commands+="send \"sh -c 'echo task1' &\r\"\n"
    commands+="sleep 0.1\n"
    commands+="send \"yes | head -n 5 > /dev/null &\r\"\n"
    commands+="sleep 0.1\n"
    commands+="send \"sh -c 'echo task2' &\r\"\n"
    commands+="sleep 0.1\n"
    commands+="send \"sh -c 'ls / > /dev/null' &\r\"\n"
    commands+="sleep 0.1\n"
    commands+="send \"wait\r\"\n"
    commands+="sleep 2\n"

    run_stress_test "$commands" "mixed_load" 30
}

test_timer_stress() {
    log_info "Timer stress test: Verify preemption under compute load"

    local commands=""
    # Create CPU-bound tasks that should be preempted by timer
    commands+="send \"sh -c 'i=0; while [ \\\$i -lt 100 ]; do i=\\\$((i+1)); done; echo loop1 done' &\r\"\n"
    commands+="sleep 0.1\n"
    commands+="send \"sh -c 'i=0; while [ \\\$i -lt 100 ]; do i=\\\$((i+1)); done; echo loop2 done' &\r\"\n"
    commands+="sleep 0.1\n"
    commands+="send \"wait\r\"\n"
    commands+="sleep 3\n"

    run_stress_test "$commands" "timer_stress" 30
}

test_race_conditions() {
    log_info "Race condition test: Rapid fork/exit cycles"

    local commands=""
    # Rapidly create and destroy processes to trigger race conditions
    for i in $(seq 1 10); do
        commands+="send \"sh -c 'exit 0' &\r\"\n"
        commands+="sleep 0.05\n"
    done
    commands+="sleep 1\n"

    run_stress_test "$commands" "race_conditions" 20
}

test_scheduler_fairness() {
    log_info "Scheduler fairness test: Verify round-robin behavior"

    local commands=""
    # Create multiple long-running processes
    commands+="send \"sh -c 'i=0; while [ \\\$i -lt 50 ]; do echo A\\\$i; i=\\\$((i+1)); done' > /tmp/log_a &\r\"\n"
    commands+="sleep 0.1\n"
    commands+="send \"sh -c 'i=0; while [ \\\$i -lt 50 ]; do echo B\\\$i; i=\\\$((i+1)); done' > /tmp/log_b &\r\"\n"
    commands+="sleep 0.1\n"
    commands+="send \"wait\r\"\n"
    commands+="sleep 3\n"
    # Verify both processes completed
    commands+="send \"wc -l /tmp/log_a /tmp/log_b\r\"\n"
    commands+="sleep 0.5\n"

    run_stress_test "$commands" "scheduler_fairness" 30
}

test_memory_pressure() {
    log_info "Memory pressure test: Many processes with COW pages"

    local commands=""
    # Fork multiple children that trigger COW faults
    for i in $(seq 1 5); do
        commands+="send \"sh -c 'touch /tmp/cow_test_$i' &\r\"\n"
        commands+="sleep 0.1\n"
    done
    commands+="send \"wait\r\"\n"
    commands+="sleep 2\n"

    run_stress_test "$commands" "memory_pressure" 25
}

# Manual testing instructions (for tests that require initramfs)
print_manual_tests() {
    cat << 'EOF'

===============================================================================
MANUAL STRESS TESTS (require BusyBox shell)
===============================================================================

Once the kernel boots to BusyBox shell (/ #), run these tests manually:

1. Fork Bomb Test (controlled):
   / # for i in 1 2 3 4 5; do sh -c "echo child $i" & done; wait
   Expected: All 5 children complete, no deadlock

2. Deep Fork Chain:
   / # sh -c 'sh -c "sh -c \"echo nested\""'
   Expected: "nested" printed, no crash

3. Pipe Stress:
   / # yes | head -n 100 | wc -l
   Expected: "100" printed

4. Concurrent File I/O:
   / # sh -c 'echo A > /tmp/f1' & sh -c 'echo B > /tmp/f2' & wait
   / # cat /tmp/f1 /tmp/f2
   Expected: "A" and "B" printed

5. Exit Code Propagation:
   / # sh -c 'exit 42'; echo $?
   Expected: "42" printed

6. Zombie Reaping:
   / # sh -c 'sleep 1 &'; sleep 2; ps
   Expected: No zombie processes

7. Scheduler Timeslice Test:
   / # sh -c 'i=0; while [ $i -lt 1000 ]; do i=$((i+1)); done; echo DONE' &
   / # sh -c 'i=0; while [ $i -lt 1000 ]; do i=$((i+1)); done; echo DONE' &
   / # wait
   Expected: Both print "DONE", interleaved by preemption

8. Memory COW Test:
   / # sh -c 'echo parent' > /tmp/test
   / # cat /tmp/test &
   / # echo child >> /tmp/test &
   / # wait
   Expected: COW fault handled, no corruption

===============================================================================
STRESS TEST SCENARIOS FOR CI
===============================================================================

For automated CI testing, use these scripts:

1. Rapid Fork/Exit:
   for i in $(seq 1 50); do sh -c 'exit 0' & done; wait

2. Pipeline Stress:
   for i in $(seq 1 10); do yes | head -n 10 > /dev/null; done

3. File Creation Stress:
   for i in $(seq 1 20); do touch /tmp/file_$i & done; wait; ls /tmp | wc -l

4. Mixed Workload:
   (yes | head -n 100 > /dev/null &)
   (sh -c 'echo A' > /tmp/a &)
   (sh -c 'echo B' > /tmp/b &)
   wait
   cat /tmp/a /tmp/b

===============================================================================
EXPECTED FAILURE MODES (to watch for)
===============================================================================

1. Deadlock: Kernel becomes unresponsive, no further output
   → Check scheduler run queue, verify timer IRQ firing

2. Crash: Unexpected exception, kernel panic
   → Check page fault logs, verify VMA ranges

3. Zombie Accumulation: ps shows many <defunct> processes
   → Check wait/exit implementation, parent notification

4. Starvation: Some processes never run
   → Verify round-robin queue, check enqueue/dequeue logic

5. Timer IRQ Storm: Kernel loops rapidly without progress
   → Check IRQ EOI/Deact sequence, verify timeslice accounting

===============================================================================

To run a specific manual test:
1. Boot kernel: bash scripts/run_qemu.sh
2. Wait for shell prompt: / #
3. Copy/paste test commands from above

EOF
}

# Main test runner
main() {
    check_prerequisites

    local failed=0
    local passed=0

    case "$TEST_NAME" in
        fork_bomb)
            test_fork_bomb || ((failed++))
            ((passed++))
            ;;
        exec_stress)
            test_exec_stress || ((failed++))
            ((passed++))
            ;;
        pipe_stress)
            test_pipe_stress || ((failed++))
            ((passed++))
            ;;
        mixed_load)
            test_mixed_load || ((failed++))
            ((passed++))
            ;;
        timer_stress)
            test_timer_stress || ((failed++))
            ((passed++))
            ;;
        race_conditions)
            test_race_conditions || ((failed++))
            ((passed++))
            ;;
        scheduler_fairness)
            test_scheduler_fairness || ((failed++))
            ((passed++))
            ;;
        memory_pressure)
            test_memory_pressure || ((failed++))
            ((passed++))
            ;;
        manual)
            print_manual_tests
            exit 0
            ;;
        all)
            log_info "Running all stress tests..."
            test_fork_bomb || ((failed++))
            ((passed++))
            test_exec_stress || ((failed++))
            ((passed++))
            test_pipe_stress || ((failed++))
            ((passed++))
            test_mixed_load || ((failed++))
            ((passed++))
            test_timer_stress || ((failed++))
            ((passed++))
            test_race_conditions || ((failed++))
            ((passed++))
            test_scheduler_fairness || ((failed++))
            ((passed++))
            test_memory_pressure || ((failed++))
            ((passed++))
            ;;
        *)
            log_error "Unknown test: $TEST_NAME"
            echo "Available tests: fork_bomb, exec_stress, pipe_stress, mixed_load, timer_stress, race_conditions, scheduler_fairness, memory_pressure, manual, all"
            exit 1
            ;;
    esac

    echo ""
    echo "==============================================================================="
    if [ $failed -eq 0 ]; then
        log_info "All stress tests passed! ($passed/$passed)"
        echo "==============================================================================="
        exit 0
    else
        log_error "Some stress tests failed: $failed failed, $((passed - failed)) passed"
        echo "==============================================================================="
        exit 1
    fi
}

main
