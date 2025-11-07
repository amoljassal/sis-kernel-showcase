#!/usr/bin/env bash
# Memory subsystem tests
# Phase 1.2 - Modular shell tests

run_test_impl() {
    # Test memstats command
    send_command "memstats" || return 1
    expect_output "Heap" || return 1
    expect_output "allocs=" || return 1
    expect_output "deallocs=" || return 1

    # Memory stats should show reasonable values
    expect_output "MiB" || return 1

    return 0
}
