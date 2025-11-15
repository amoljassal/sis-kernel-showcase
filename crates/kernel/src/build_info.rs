// Build Information Module
// Phase 5 - Production Readiness Plan
//
// Provides build metadata for forensics and debugging

// Build info is provided via environment variables set in build.rs
use core::cmp::min;

use alloc::string::String;
use alloc::string::ToString;
use alloc::format;

/// Get complete build information as a formatted string
pub fn get_build_info() -> String {
    let commit = option_env!("GIT_COMMIT").unwrap_or("unknown");
    let branch = option_env!("GIT_BRANCH").unwrap_or("unknown");
    let dirty = option_env!("GIT_DIRTY").map(|s| s == "1" || s.eq_ignore_ascii_case("true")).unwrap_or(false);
    let ts = option_env!("BUILD_TIMESTAMP").unwrap_or("0");
    let rustv = option_env!("RUST_VERSION").unwrap_or("unknown");
    let feats = option_env!("FEATURES").unwrap_or("");
    let profile = option_env!("PROFILE").unwrap_or("unknown");
    let target = option_env!("TARGET").unwrap_or("unknown");

    let dirty_marker = if dirty { " (dirty)" } else { "" };
    format!(
        "SIS Kernel Build Information\n\
         Git:       {} @ {}{}\n\
         Built:     {}\n\
         Rust:      {}\n\
         Features:  {}\n\
         Profile:   {}\n\
         Target:    {}",
        &commit[..min(12, commit.len())],
        branch,
        dirty_marker,
        ts,
        rustv,
        if feats.is_empty() { "(none)".to_string() } else { feats.to_string() },
        profile,
        target
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
    let commit = option_env!("GIT_COMMIT").unwrap_or("unknown");
    let branch = option_env!("GIT_BRANCH").unwrap_or("unknown");
    let dirty = option_env!("GIT_DIRTY").map(|s| s == "1" || s.eq_ignore_ascii_case("true")).unwrap_or(false);
    let ts = option_env!("BUILD_TIMESTAMP").unwrap_or("0");
    let dirty_marker = if dirty { "+" } else { "" };
    let date = if ts.len() >= 10 { &ts[..10] } else { ts };
    format!(
        "SIS Kernel {} ({}{}) built {}",
        &commit[..min(7, commit.len())],
        branch,
        dirty_marker,
        date
    )
}

/// Get build information as JSON (for programmatic access)
pub fn get_build_info_json() -> String {
    let commit = option_env!("GIT_COMMIT").unwrap_or("unknown");
    let branch = option_env!("GIT_BRANCH").unwrap_or("unknown");
    let dirty = option_env!("GIT_DIRTY").map(|s| s == "1" || s.eq_ignore_ascii_case("true")).unwrap_or(false);
    let ts = option_env!("BUILD_TIMESTAMP").unwrap_or("0");
    let rustv = option_env!("RUST_VERSION").unwrap_or("unknown");
    let feats = option_env!("FEATURES").unwrap_or("");
    let profile = option_env!("PROFILE").unwrap_or("unknown");
    let target = option_env!("TARGET").unwrap_or("unknown");
    alloc::format!(
        "{{\n  \"git_commit\": \"{}\",\n  \"git_branch\": \"{}\",\n  \"git_dirty\": {},\n  \"build_timestamp\": \"{}\",\n  \"rust_version\": \"{}\",\n  \"features\": \"{}\",\n  \"profile\": \"{}\",\n  \"target\": \"{}\"\n}}",
        commit, branch, dirty, ts, rustv, feats, profile, target
    )
}
