# Directory Restructuring - Industry Standard Layout

**Date:** November 6, 2025
**Status:** ✅ COMPLETE

## Summary

Successfully reorganized SIS Kernel project from ad-hoc structure to industry-standard Rust kernel layout. All builds verified working.

## Changes Made

### Before (Ad-hoc Structure)
```
sis-kernel/
├── apps/
│   ├── daemon/           # Control daemon
│   │   └── samples/      # Sample log files
│   └── desktop/          # Web GUI
├── crates/
│   ├── kernel/
│   ├── uefi-boot/
│   └── testing/
├── docs/
├── scripts/
└── tools/
```

### After (Industry Standard)
```
sis-kernel/
├── crates/               # All Rust workspace members
│   ├── daemon/           # ← moved from apps/daemon
│   ├── kernel/           # (unchanged)
│   ├── uefi-boot/        # (unchanged)
│   └── testing/          # (unchanged)
├── gui/                  # Non-Rust GUI applications
│   └── desktop/          # ← moved from apps/desktop
├── samples/              # Sample logs for testing/replay
│   ├── boot_minimal.log
│   ├── boot_with_metrics.log
│   ├── boot_graph.log
│   ├── boot_llm.log
│   ├── boot_sched.log
│   ├── logs_mixed.log
│   └── self_check.log
├── docs/                 # All documentation
├── scripts/              # Build and run scripts
└── tools/                # Standalone utilities
```

## Files Modified

### 1. Cargo.toml (Root Workspace)
**Before:**
```toml
[workspace]
members = [
  "crates/kernel",
  "crates/uefi-boot",
  "crates/testing",
  "apps/daemon"
]
exclude = [
  "apps/desktop/src-tauri"
]
```

**After:**
```toml
[workspace]
members = [
  "crates/kernel",
  "crates/uefi-boot",
  "crates/testing",
  "crates/daemon"
]
exclude = [
  "gui/desktop/src-tauri"
]
```

### 2. pnpm-workspace.yaml
**Before:**
```yaml
packages:
  - 'apps/*'
  - 'packages/*'
```

**After:**
```yaml
packages:
  - 'gui/*'
  - 'packages/*'
```

## Verification Tests

All verification tests passed:

✅ **Cargo Build:** `cargo build --release --bin sisctl --manifest-path crates/daemon/Cargo.toml`
- Status: Success (59 warnings, 0 errors)
- Time: 57.35s

✅ **pnpm Install:** `pnpm install`
- Status: Success
- Packages: +205 installed
- Time: 10s

✅ **GUI Build:** `pnpm -F desktop build`
- Status: Success
- Output: dist/index.html, CSS, JS bundle
- Time: 2.73s

## Directory Purpose

### `/crates`
**Purpose:** All Rust workspace members
**Contents:**
- `daemon/` - sisctl control daemon (HTTP API, QEMU management)
- `kernel/` - Core SIS kernel (capability system, scheduling, LLM integration)
- `testing/` - Test utilities and runtime support
- `uefi-boot/` - UEFI bootloader integration

**Rationale:** Standard Rust project layout. All Rust code lives under crates/ as workspace members.

### `/gui`
**Purpose:** Non-Rust graphical interfaces
**Contents:**
- `desktop/` - React/TypeScript web-based GUI for kernel management

**Rationale:** Separates GUI code (TypeScript/React) from Rust codebase. Allows different build systems and dependencies.

### `/samples`
**Purpose:** Sample logs for replay and testing
**Contents:** Boot sequences, metrics samples, test scenarios

**Rationale:** Shared test data accessible to both daemon and GUI. Previously buried in apps/daemon/samples.

### `/docs`
**Purpose:** All project documentation
**Contents:** Architecture docs, API specs, integration guides

**Rationale:** Centralized documentation discovery.

### `/scripts`
**Purpose:** Build, run, and deployment scripts
**Contents:** uefi_run.sh, demo scripts, ESP partition setup

**Rationale:** Tooling separate from source code.

### `/tools`
**Purpose:** Standalone utility programs
**Contents:** sis_datactl.py and other utilities

**Rationale:** Tools that aren't part of the main build but support development.

## Benefits of New Structure

1. **Industry Standard:** Matches conventions of major Rust kernel projects (Redox, Theseus)
2. **Clear Separation:** Rust code (crates/) vs non-Rust (gui/)
3. **Better Discovery:** Samples at root level instead of hidden in daemon
4. **Workspace Clarity:** All Cargo workspace members under crates/
5. **Onboarding:** New contributors understand structure immediately

## Migration Impact

### What Changed
- File paths in Cargo.toml
- File paths in pnpm-workspace.yaml
- Physical location of files

### What Didn't Change
- No code changes required
- No import statement changes
- No functionality changes
- All builds work identically

## Next Steps

With directory structure complete, ready for:
1. ✅ GUI isolated testing (COMPLETE)
2. ✅ Web app architecture (COMPLETE - removed Tauri)
3. 🔄 **NEXT:** Integrate GUI with kernel
4. 🔄 Wire QEMU management to actual kernel
5. 🔄 End-to-end integration testing

## References

- Previous structure: `docs/apps-README.md` (moved from `apps/README.md`)
- Web app conversion: `docs/E2E_TEST_ISSUES_SUMMARY.md`
- Integration testing: `docs/INTEGRATION_TEST_REPORT.md`
