// Build Information Module
// Phase 5 - Production Readiness Plan
//
// Provides build metadata for forensics and debugging

// Include auto-generated build information
include!(concat!(env!("OUT_DIR"), "/build_info.rs"));

use alloc::string::String;
use alloc::format;

/// Get complete build information as a formatted string
pub fn get_build_info() -> String {
    let dirty_marker = if GIT_DIRTY { " (dirty)" } else { "" };

    format!(
        "SIS Kernel Build Information\n\
         Git:       {} @ {}{}\n\
         Built:     {}\n\
         Rust:      {}\n\
         Features:  {}\n\
         Profile:   {}\n\
         Target:    {}",
        &GIT_COMMIT[..core::cmp::min(12, GIT_COMMIT.len())],
        GIT_BRANCH,
        dirty_marker,
        BUILD_TIMESTAMP,
        RUST_VERSION,
        if FEATURES.is_empty() { "(none)" } else { FEATURES },
        PROFILE,
        TARGET
    )
}

/// Print build information to console
pub fn print_build_info() {
    let info = get_build_info();
    unsafe {
        crate::uart_print(b"\n");
        crate::uart_print(b"========================================\n");
        crate::uart_print(info.as_bytes());
        crate::uart_print(b"\n");
        crate::uart_print(b"========================================\n");
    }
}

/// Get short version string (for shell version command)
pub fn get_version_string() -> String {
    let dirty_marker = if GIT_DIRTY { "+" } else { "" };

    format!(
        "SIS Kernel {} ({}{}) built {}",
        &GIT_COMMIT[..core::cmp::min(7, GIT_COMMIT.len())],
        GIT_BRANCH,
        dirty_marker,
        &BUILD_TIMESTAMP[..10] // Just the date
    )
}

/// Get build information as JSON (for programmatic access)
pub fn get_build_info_json() -> &'static str {
    BUILD_INFO_JSON
}
