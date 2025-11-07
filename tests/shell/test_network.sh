#!/usr/bin/env bash
# Network subsystem tests
# Phase 1.2 - Modular shell tests

run_test_impl() {
    # Test netstat command
    send_command "netstat" || return 1
    expect_output "Interface" || return 1

    # Should show at least one interface
    expect_output "UP\|DOWN" || return 1

    return 0
}
