// Build script for SIS Kernel
// Phase 5 - Production Readiness Plan
//
// Generates build metadata for forensics and debugging

use std::env;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // Get git information
    let git_commit = get_git_commit();
    let git_branch = get_git_branch();
    let git_dirty = is_git_dirty();

    // Get build timestamp (unix seconds)
    let build_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string());

    // Get Rust version (unused, but kept for consistency with runtime exports)
    let _rust_version = get_rust_version();

    // Get enabled features (unused, but kept for consistency with runtime exports)
    let _features = get_enabled_features();

    // Get build profile (debug/release) (unused, but kept for consistency with runtime exports)
    let _profile = env::var("PROFILE").unwrap_or_else(|_| "unknown".to_string());

    // Get target triple (unused, but kept for consistency with runtime exports)
    let _target = env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());

    // Export as environment variables only; runtime module reads these via option_env!
    println!("cargo:rustc-env=GIT_COMMIT={}", git_commit);
    println!("cargo:rustc-env=GIT_BRANCH={}", git_branch);
    println!("cargo:rustc-env=GIT_DIRTY={}", if git_dirty { "1" } else { "0" });
    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", build_timestamp);
    println!("cargo:rustc-env=RUST_VERSION={}", get_rust_version());
    println!("cargo:rustc-env=FEATURES={}", get_enabled_features());
    println!("cargo:rustc-env=PROFILE={}", env::var("PROFILE").unwrap_or_else(|_| "unknown".to_string()));
    println!("cargo:rustc-env=TARGET={}", env::var("TARGET").unwrap_or_else(|_| "unknown".to_string()));

    // Optional: embed models initramfs for integration tests
    if let Ok(initramfs_path) = env::var("INITRAMFS_MODELS") {
        println!("cargo:rerun-if-changed={}", initramfs_path);
        println!("cargo:rustc-env=INITRAMFS_MODELS_FILE={}", initramfs_path);
        println!("cargo:rustc-cfg=have_initramfs_models");
    }
}

fn get_git_commit() -> String {
    Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn get_git_branch() -> String {
    Command::new("git")
        .args(&["branch", "--show-current"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn is_git_dirty() -> bool {
    Command::new("git")
        .args(&["status", "--porcelain"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                Some(!output.stdout.is_empty())
            } else {
                None
            }
        })
        .unwrap_or(false)
}

fn get_rust_version() -> String {
    Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn get_enabled_features() -> String {
    // Get features from CARGO_FEATURE_* environment variables
    let mut features = Vec::new();

    for (key, _) in env::vars() {
        if key.starts_with("CARGO_FEATURE_") {
            let feature = key["CARGO_FEATURE_".len()..].to_lowercase();
            features.push(feature);
        }
    }

    features.sort();
    features.join(",")
}

#[allow(dead_code)]
fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[allow(dead_code)]
fn escape_rust(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
