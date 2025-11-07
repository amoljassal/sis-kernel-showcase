#!/usr/bin/env bash
# Test graceful handling of memory pressure (allocation failures)
# Phase 3.1 - Chaos Testing

run_test_impl() {
    echo "  Testing memory pressure scenario..."

    # Enable chaos mode: memory_pressure
    send_command "chaos mode memory_pressure" || return 1
    expect_output "Chaos mode set to: memory_pressure" || return 1

    # Set failure rate to 20%
    send_command "chaos rate 20" || return 1
    expect_output "Failure rate set to 20%" || return 1

    # Try operations that may allocate memory
    send_command "memstats" || return 1
    send_command "help" || return 1
    sleep 1

    # Check that kernel didn't panic (should handle ENOMEM gracefully)
    expect_no_output "KERNEL PANIC" || return 1

    # Check statistics
    send_command "chaos stats" || return 1
    expect_output "Alloc failures" || return 1

    # Disable chaos mode
    send_command "chaos mode none" || return 1

    echo "  ✓ Memory pressure scenario handled gracefully"
    return 0
}
