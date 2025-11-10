#!/usr/bin/env bash
# Test graceful handling of disk full (ENOSPC) errors
# Phase 3.1 - Chaos Testing

run_test_impl() {
    echo "  Testing disk full scenario..."

    # Enable chaos mode: disk_full
    send_command "chaos mode disk_full" || return 1
    expect_output "Chaos mode set to: disk_full" || return 1

    # Set failure rate to 50%
    send_command "chaos rate 50" || return 1
    expect_output "Failure rate set to 50%" || return 1

    # Try to create a file (should fail with ENOSPC sometimes)
    send_command "touch /test.txt" || return 1
    sleep 2

    # Check that kernel didn't panic
    expect_no_output "KERNEL PANIC" || return 1

    # Try a few more file operations
    send_command "ls /" || return 1
    send_command "cat /test.txt" || return 1
    sleep 1

    # Check statistics
    send_command "chaos stats" || return 1
    expect_output "Chaos Statistics" || return 1

    # Disable chaos mode
    send_command "chaos mode none" || return 1
    expect_output "Chaos mode set to: none" || return 1

    echo "  ✓ Disk full scenario handled gracefully"
    return 0
}
