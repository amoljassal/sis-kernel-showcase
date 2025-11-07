#!/usr/bin/env bash
# Test handling of slow I/O operations
# Phase 3.1 - Chaos Testing

run_test_impl() {
    echo "  Testing slow I/O scenario..."

    # Enable chaos mode: slow_io
    send_command "chaos mode slow_io" || return 1
    expect_output "Chaos mode set to: slow_io" || return 1

    # Set failure rate to 40%
    send_command "chaos rate 40" || return 1
    expect_output "Failure rate set to 40%" || return 1

    # Measure time for operations (should be slower)
    START_TIME=$(date +%s)
    send_command "ls /" || return 1
    send_command "cat /dev/null" || return 1
    send_command "memstats" || return 1
    END_TIME=$(date +%s)

    ELAPSED=$((END_TIME - START_TIME))
    echo "  Operations took ${ELAPSED}s (with I/O delays)"

    # Check that kernel is still responsive
    send_command "uptime" || return 1
    expect_output "uptime" || return 1

    # Disable chaos mode
    send_command "chaos mode none" || return 1

    echo "  ✓ Slow I/O scenario handled (kernel remained responsive)"
    return 0
}
