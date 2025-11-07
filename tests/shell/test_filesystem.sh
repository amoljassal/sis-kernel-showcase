#!/usr/bin/env bash
# Filesystem tests
# Phase 1.2 - Modular shell tests

run_test_impl() {
    # Test ls command
    send_command "ls" || return 1
    expect_output "/" || return 1

    # Test mount command if available
    send_command "mount" || return 1
    # Just check it doesn't crash
    sleep 1

    return 0
}
