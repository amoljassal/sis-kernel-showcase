# Code Conventions and Style Guide

**Status:** Production Standards
**Audience:** Contributors, reviewers, maintainers
**Last Updated:** 2025-01-11

---

## Table of Contents

1. [Quick Reference](#quick-reference)
2. [Rust Style Guide](#rust-style-guide)
3. [Code Organization](#code-organization)
4. [Documentation Standards](#documentation-standards)
5. [Testing Requirements](#testing-requirements)
6. [PR Process](#pr-process)
7. [Static Analysis & Linting](#static-analysis--linting)
8. [CI/CD Enforcement](#cicd-enforcement)

---

## Quick Reference

**Before submitting code:**
```bash
# 1. Format code
cargo fmt --all

# 2. Run clippy with strict lints
cargo clippy --all-targets --all-features -- -D warnings

# 3. Run tests
cargo test --package sis-kernel

# 4. Build with strict mode
SIS_FEATURES="llm,ai-ops,crypto-real" ./scripts/uefi_run.sh build

# 5. Verify no new warnings
cargo build 2>&1 | grep warning
```

**Key rules:**
- ✅ Always run `cargo fmt` before committing
- ✅ Fix all clippy warnings (`-D warnings` in CI)
- ✅ Add tests for new functionality
- ✅ Update docs when changing APIs
- ✅ Keep PRs focused (<500 lines preferred)

---

## Rust Style Guide

### Formatting

We use **rustfmt** with default settings:

```toml
# .rustfmt.toml (if needed for customization)
max_width = 100
tab_spaces = 4
edition = "2021"
```

**Automatic formatting:**
```bash
# Format all code
cargo fmt --all

# Check formatting without modifying
cargo fmt --all -- --check
```

**IDE integration:**
- **VS Code:** Install rust-analyzer, enable format-on-save
- **IntelliJ IDEA:** Enable rustfmt in Preferences → Languages & Frameworks → Rust
- **Vim/Neovim:** Use `:RustFmt` or ALE plugin

### Naming Conventions

| Item | Convention | Example |
|------|------------|---------|
| **Modules** | `snake_case` | `meta_agent.rs`, `stress_test.rs` |
| **Structs** | `PascalCase` | `AutonomyMetrics`, `StressTestConfig` |
| **Enums** | `PascalCase` | `ChaosModeType`, `FailureInjectionType` |
| **Functions** | `snake_case` | `run_memory_stress()`, `calculate_pressure()` |
| **Constants** | `SCREAMING_SNAKE_CASE` | `MAX_ALLOCATIONS`, `DEFAULT_THRESHOLD` |
| **Statics** | `SCREAMING_SNAKE_CASE` | `AUTONOMY_METRICS`, `GLOBAL_ALLOCATOR` |
| **Type aliases** | `PascalCase` | `Result<T>`, `AllocResult` |

**Examples:**
```rust
// ✅ Good
const MAX_COMPACTION_RATE: u64 = 1500; // milliseconds
static AUTONOMY_METRICS: AutonomyMetrics = AutonomyMetrics::new();

pub struct StressTestConfig {
    pub duration_ms: u64,
    pub failure_rate: u8,
}

pub fn run_chaos_test(config: &StressTestConfig) -> Result<Metrics> {
    // ...
}

// ❌ Bad
const maxCompactionRate: u64 = 1500;  // Wrong case
struct stressTestConfig { }           // Wrong case
fn RunChaosTest() { }                 // Wrong case
```

### Code Organization

**Module structure:**
```rust
// File: crates/kernel/src/stress_test.rs

// 1. Module documentation
//! Stress testing framework for chaos engineering and memory validation.

// 2. Imports (grouped)
use core::sync::atomic::{AtomicU64, Ordering};
use alloc::vec::Vec;
use crate::prng;
use crate::autonomy::AUTONOMY_METRICS;

// 3. Constants
const DEFAULT_DURATION_MS: u64 = 10_000;
const MAX_FAILURE_RATE: u8 = 100;

// 4. Type definitions
pub type TestResult = Result<StressTestMetrics, StressTestError>;

// 5. Structs and enums
pub struct StressTestConfig { /* ... */ }
pub enum ChaosModeType { /* ... */ }

// 6. Implementations
impl StressTestConfig {
    pub fn new() -> Self { /* ... */ }
}

// 7. Public functions
pub fn run_stress_test(config: &StressTestConfig) -> TestResult { /* ... */ }

// 8. Private helper functions
fn calculate_pressure(used: usize, total: usize) -> u8 { /* ... */ }

// 9. Tests (at end of file)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pressure_calculation() { /* ... */ }
}
```

**Import grouping:**
```rust
// 1. Standard library (core, alloc)
use core::sync::atomic::AtomicU64;
use alloc::vec::Vec;

// 2. External crates
use smoltcp::iface::Interface;

// 3. Internal crate modules
use crate::prng;
use crate::heap::HEAP;

// 4. Parent/current module
use super::config::Config;
```

### Error Handling

**Prefer `Result` over panic:**
```rust
// ✅ Good - Returns Result
pub fn allocate_memory(size: usize) -> Result<*mut u8, AllocError> {
    if size == 0 {
        return Err(AllocError::InvalidSize);
    }
    // Attempt allocation
    HEAP.lock().alloc(size)
        .ok_or(AllocError::OutOfMemory)
}

// ❌ Bad - Panics on error
pub fn allocate_memory(size: usize) -> *mut u8 {
    assert!(size > 0, "Size must be non-zero");  // Panics!
    HEAP.lock().alloc(size).unwrap()             // Panics!
}
```

**When to panic:**
- Invariant violations that indicate bugs (use `debug_assert!`)
- Unrecoverable initialization failures (early boot)
- Test code (`#[cfg(test)]`)

**Error types:**
```rust
// Define domain-specific error types
#[derive(Debug, Clone, Copy)]
pub enum StressTestError {
    InvalidConfig,
    AllocationFailed,
    TimeoutExceeded,
}

impl core::fmt::Display for StressTestError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            Self::InvalidConfig => write!(f, "Invalid stress test configuration"),
            Self::AllocationFailed => write!(f, "Memory allocation failed"),
            Self::TimeoutExceeded => write!(f, "Test duration exceeded"),
        }
    }
}
```

### Atomic Operations

**Always specify ordering:**
```rust
use core::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

// ✅ Good - Explicit ordering
fn increment() {
    COUNTER.fetch_add(1, Ordering::Relaxed);
}

fn load_counter() -> u64 {
    COUNTER.load(Ordering::Acquire)
}

// ❌ Bad - Ambiguous (doesn't compile)
fn increment_bad() {
    COUNTER.fetch_add(1);  // Missing ordering!
}
```

**Ordering guidelines:**
- **Relaxed:** Counters, metrics (no synchronization needed)
- **Acquire/Release:** Producer-consumer patterns
- **SeqCst:** When in doubt (conservative, slightly slower)

### Unsafe Code

**Minimize unsafe, document thoroughly:**
```rust
// ✅ Good - Justified and documented
/// # Safety
/// Caller must ensure:
/// - `ptr` is valid and properly aligned
/// - `ptr` points to initialized memory
/// - No other references to `ptr` exist during this call
pub unsafe fn read_volatile_u64(ptr: *const u64) -> u64 {
    core::ptr::read_volatile(ptr)
}

// ❌ Bad - Unjustified, undocumented
pub fn read_value(ptr: *const u64) -> u64 {
    unsafe { *ptr }  // Why unsafe? What are the invariants?
}
```

**Unsafe checklist:**
- [ ] Is unsafe necessary? (Try safe alternatives first)
- [ ] Are safety invariants documented?
- [ ] Are preconditions checked in debug builds?
- [ ] Is the unsafe block minimal in scope?
- [ ] Have edge cases been tested?

---

## Documentation Standards

### Module-Level Documentation

Every module needs a header comment:

```rust
//! Stress testing framework for chaos engineering.
//!
//! This module provides:
//! - Memory pressure stress tests with OOM simulation
//! - Chaos engineering with 12 failure injection types
//! - Autonomy impact measurement and comparison
//!
//! # Examples
//!
//! ```rust
//! let config = StressTestConfig::new()
//!     .duration_ms(10_000)
//!     .failure_rate(10);
//! let metrics = run_chaos_test(&config)?;
//! println!("Success rate: {}", metrics.success_rate());
//! ```
//!
//! # Architecture
//!
//! Tests run in three phases:
//! 1. Setup: Initialize PRNG, reset metrics
//! 2. Execution: Run test loop with failure injection
//! 3. Analysis: Calculate statistics and latency percentiles
```

### Function Documentation

**Public functions need doc comments:**
```rust
/// Runs memory stress test with autonomy comparison.
///
/// This test exercises the memory allocator by creating allocation pressure,
/// triggering OOM events, and measuring autonomy impact on peak pressure
/// and OOM frequency.
///
/// # Arguments
///
/// * `config` - Test configuration (duration, noise level, failure rate)
///
/// # Returns
///
/// * `Ok(StressTestMetrics)` - Test results with pressure, OOM count, latency
/// * `Err(StressTestError::AllocationFailed)` - If initial allocation fails
///
/// # Examples
///
/// ```rust
/// let config = StressTestConfig::new().duration_ms(20_000);
/// let metrics = run_memory_stress(&config)?;
/// println!("Peak pressure: {}%", metrics.peak_pressure);
/// println!("OOM events: {}", metrics.oom_events);
/// ```
///
/// # Performance
///
/// Typical runtime: 10-20 seconds
/// Memory overhead: ~540 bytes
/// CPU overhead: <100 cycles per iteration
pub fn run_memory_stress(config: &StressTestConfig) -> Result<StressTestMetrics> {
    // Implementation
}
```

**When to document:**
- ✅ All public functions, structs, enums
- ✅ Complex algorithms (explain the "why")
- ✅ Non-obvious behavior or edge cases
- ✅ Safety requirements for unsafe code
- ❌ Self-explanatory private helpers (unless complex)

### Inline Comments

**Explain "why", not "what":**
```rust
// ✅ Good - Explains reasoning
// Use exponential distribution (60% small, 30% medium, 10% large)
// to simulate realistic workload patterns seen in production
let event_size = if rand < 60 {
    small()
} else if rand < 90 {
    medium()
} else {
    large()
};

// ❌ Bad - States the obvious
// Check if rand is less than 60
if rand < 60 {
    small();
}
```

**Document tuning history:**
```rust
// TUNING HISTORY:
// v1: threshold=48%, rate=20iter → 417 compactions/10s → OOM
// v2: threshold=40%, rate=1000ms → 0 OOMs but zero impact
// v3: threshold=46%, rate=1000ms, 5-10% → -5% pressure, still 1 OOM
// v4: threshold=46%, rate=1500ms, 3-5%, pop() → -5% pressure, 0 OOMs ✓
const COMPACTION_THRESHOLD: u8 = 46;
const COMPACTION_COOLDOWN_MS: u64 = 1500;
```

---

## Testing Requirements

### Test Coverage

**Required test coverage for new code:**
- ✅ Public APIs: 80%+ coverage
- ✅ Critical paths: 100% coverage
- ✅ Error handling: All error branches tested
- ✅ Edge cases: Boundary conditions, empty inputs, max values

**Test structure:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_functionality() {
        // Arrange
        let config = StressTestConfig::new();

        // Act
        let result = run_test(&config);

        // Assert
        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert!(metrics.success_rate() >= 0.90);
    }

    #[test]
    fn test_edge_case_zero_duration() {
        let config = StressTestConfig::new().duration_ms(0);
        let result = run_test(&config);
        assert!(matches!(result, Err(StressTestError::InvalidConfig)));
    }

    #[test]
    #[should_panic(expected = "invariant violation")]
    fn test_invariant_violation() {
        // Test that invariants are checked
        unsafe { violate_invariant() };
    }
}
```

### Integration Tests

**Location:** `tests/integration/`

```rust
// tests/integration/stress_tests.rs
use sis_kernel::stress_test::*;

#[test]
fn test_chaos_with_failure_injection() {
    let config = StressTestConfig::new()
        .duration_ms(5_000)
        .failure_rate(20);

    let metrics = run_chaos_test(&config).expect("Test failed");

    // Verify failure rate within tolerance
    let actual_rate = metrics.failure_rate();
    assert!((actual_rate - 20.0).abs() < 5.0,
            "Failure rate {} outside tolerance", actual_rate);
}
```

### Test Naming

| Test Type | Prefix | Example |
|-----------|--------|---------|
| Unit | `test_` | `test_pressure_calculation()` |
| Integration | `integration_` | `integration_full_stack()` |
| Benchmark | `bench_` | `bench_allocation_latency()` |
| Regression | `regression_` | `regression_oom_bug_fix()` |

---

## PR Process

### Before Submitting

**Checklist:**
- [ ] Code formatted (`cargo fmt --all`)
- [ ] Clippy passes (`cargo clippy -- -D warnings`)
- [ ] Tests pass (`cargo test`)
- [ ] Build succeeds in strict mode
- [ ] Documentation updated
- [ ] CHANGELOG.md updated (if applicable)
- [ ] Commits are atomic and well-described

### PR Description Template

```markdown
## Summary
Brief description of changes (1-2 sentences)

## Motivation
Why is this change needed? What problem does it solve?

## Changes
- Added X functionality
- Fixed Y bug
- Refactored Z for performance

## Testing
- Unit tests: Added 5 tests covering edge cases
- Integration: Tested full chaos test with failure injection
- Manual: Ran in QEMU for 20 minutes, no regressions

## Performance Impact
- Allocation latency: 500ns → 450ns (10% faster)
- Memory overhead: +120 bytes
- Binary size: No change

## Checklist
- [x] Tests added/updated
- [x] Documentation updated
- [x] Ran `cargo fmt`
- [x] Ran `cargo clippy`
- [x] CI passes
```

### Commit Message Format

Follow **conventional commits:**

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `refactor`: Code refactoring
- `docs`: Documentation only
- `test`: Test additions/changes
- `perf`: Performance improvement
- `chore`: Maintenance tasks

**Examples:**
```
feat(stress-tests): add exponential event distribution

Implemented exponential distribution for chaos events to simulate
realistic production workload patterns:
- 60% small events (5-15 allocations) → fast
- 30% medium events (15-40 allocations) → moderate
- 10% large events (40-100 allocations) → slow

This reduces p50 latency from 5ms to 0.5ms while maintaining
realistic tail latencies (p99=500ms).

Closes #42
```

```
fix(memory): eliminate OOM regression from excessive compaction

Root cause: 417 compactions/10s caused allocator thrashing.

Solution: Added 1.5-second cooldown rate limiting, reducing
compaction rate to 0.5-0.6/sec while maintaining -5% peak
pressure reduction.

Tested: 10s and 20s runs show 0 OOMs with autonomy ON.

Fixes #38
```

### Code Review Standards

**What reviewers check:**
1. **Correctness:** Does the code do what it claims?
2. **Tests:** Are edge cases covered?
3. **Safety:** Are unsafe blocks justified?
4. **Performance:** Any regressions?
5. **Style:** Follows conventions?
6. **Documentation:** Is behavior explained?

**Review response time:**
- Small PRs (<100 lines): 24 hours
- Medium PRs (100-500 lines): 48 hours
- Large PRs (>500 lines): 72 hours (prefer splitting)

---

## Static Analysis & Linting

### Clippy Configuration

**Enabled lints:**
```toml
# Cargo.toml
[lints.clippy]
all = "warn"
pedantic = "warn"
nursery = "warn"
cargo = "warn"

# Allow some pedantic lints for no_std environment
missing_errors_doc = "allow"
missing_panics_doc = "allow"
```

**Running clippy:**
```bash
# All warnings as errors (CI mode)
cargo clippy --all-targets --all-features -- -D warnings

# Fix automatically where possible
cargo clippy --fix

# Check specific crate
cargo clippy --package sis-kernel -- -D warnings
```

### Common Clippy Fixes

**Unnecessary `clone()`:**
```rust
// ❌ Before
let value = config.clone();
process(value);

// ✅ After (if config not used again)
process(config);
```

**Explicit `return`:**
```rust
// ❌ Before
fn calculate() -> u64 {
    let result = 42;
    return result;
}

// ✅ After
fn calculate() -> u64 {
    42
}
```

**Redundant pattern matching:**
```rust
// ❌ Before
match result {
    Ok(val) => Ok(val),
    Err(e) => Err(e),
}

// ✅ After
result
```

### rustfmt Configuration

**Formatting checks:**
```bash
# Check formatting (CI)
cargo fmt --all -- --check

# Apply formatting
cargo fmt --all
```

---

## CI/CD Enforcement

### GitHub Actions Workflow

**CI checks on every PR:**
```yaml
name: CI

on: [push, pull_request]

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          components: rustfmt, clippy

      - name: Check formatting
        run: cargo fmt --all -- --check

      - name: Run clippy
        run: cargo clippy --all-targets --all-features -- -D warnings

  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Run tests
        run: cargo test --all-features

  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Build (strict mode)
        run: cargo build --release --all-features
        env:
          RUSTFLAGS: "-D warnings"
```

### Pre-commit Hooks

**Install pre-commit hooks:**
```bash
# .git/hooks/pre-commit
#!/bin/bash
set -e

echo "Running pre-commit checks..."

# Format check
cargo fmt --all -- --check || {
    echo "❌ Format check failed. Run: cargo fmt --all"
    exit 1
}

# Clippy
cargo clippy --all-targets -- -D warnings || {
    echo "❌ Clippy failed. Fix warnings and try again."
    exit 1
}

# Tests
cargo test || {
    echo "❌ Tests failed."
    exit 1
}

echo "✅ All checks passed!"
```

**Enable hooks:**
```bash
chmod +x .git/hooks/pre-commit
git config core.hooksPath .git/hooks
```

---

## Summary

**Key principles:**
1. **Consistency:** Follow rustfmt and clippy recommendations
2. **Safety:** Minimize unsafe, document invariants
3. **Testing:** 80%+ coverage for public APIs
4. **Documentation:** Explain "why", not just "what"
5. **Reviews:** All code reviewed before merge
6. **CI:** Automated checks enforce standards

**Resources:**
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Clippy Lint List](https://rust-lang.github.io/rust-clippy/)
- [Conventional Commits](https://www.conventionalcommits.org/)

**Questions?**
- Open an issue with label `question`
- Ask in #engineering Slack channel
- Email: engineering@sis-kernel.dev
