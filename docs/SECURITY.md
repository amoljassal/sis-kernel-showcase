# Security & Fuzzing

**Phase 4 - Production Readiness Plan**

## Overview

This document describes the security hardening and fuzzing infrastructure implemented for the SIS Kernel. The focus is on syscall input validation, fuzzing, and security best practices.

## Input Validation

### Syscall Validation Framework

All syscalls undergo comprehensive input validation before processing:

**Location**: `crates/kernel/src/syscall/validation.rs`

### Validation Categories

#### 1. Syscall Number Validation

```rust
// Validate syscall number is within supported range
SyscallValidator::validate_syscall_number(nr)?;
// Returns ENOSYS if invalid
```

**Checks**:
- Syscall number ≤ MAX_SYSCALL_NUM (512)

#### 2. File Descriptor Validation

```rust
// Validate file descriptor
SyscallValidator::validate_fd(fd)?;
// Returns EBADF if invalid
```

**Checks**:
- FD ≥ 0
- FD < MAX_FD (1024)

#### 3. Pointer Validation

```rust
// Validate user-space pointer
SyscallValidator::validate_user_ptr(ptr, len)?;
// Returns EFAULT if invalid
```

**Checks**:
- Not null
- Not in kernel space (< 0xffff_0000_0000_0000)
- Within user space bounds
- No overflow in ptr + len
- No wrap-around

#### 4. Buffer Validation

```rust
// Validate read buffer
SyscallValidator::validate_read_buffer(ptr, len)?;

// Validate write buffer
SyscallValidator::validate_write_buffer(ptr, len)?;
```

**Checks**:
- Pointer validity
- Length ≤ MAX_IO_SIZE (~2GB)
- No integer overflow

#### 5. String/Path Validation

```rust
// Validate null-terminated string
SyscallValidator::validate_string_ptr(ptr, max_len)?;

// Validate filesystem path
SyscallValidator::validate_path(ptr)?;
```

**Checks**:
- Null terminator within MAX_PATH_LEN (4096)
- No buffer overflow
- Valid pointer

#### 6. Flags & Mode Validation

```rust
// Validate flags against valid mask
SyscallValidator::validate_flags(flags, valid_mask)?;

// Validate file mode bits
SyscallValidator::validate_mode(mode)?;
```

**Checks**:
- Only valid bits set
- Mode ≤ 0o7777

#### 7. Signal Validation

```rust
// Validate signal number
SyscallValidator::validate_signal(sig)?;
```

**Checks**:
- Signal number ≥ 0
- Signal number ≤ MAX_SIGNAL (64)

#### 8. PID Validation

```rust
// Validate process ID
SyscallValidator::validate_pid(pid)?;
```

**Checks**:
- PID ≥ -1 (allows -1 for "all processes")

#### 9. Socket Validation

```rust
// Validate socket domain
SyscallValidator::validate_socket_domain(domain)?;

// Validate socket type
SyscallValidator::validate_socket_type(sock_type)?;
```

**Checks**:
- Domain in {AF_UNIX, AF_INET, AF_INET6}
- Type in {SOCK_STREAM, SOCK_DGRAM, SOCK_RAW}

#### 10. mmap Validation

```rust
// Validate protection flags
SyscallValidator::validate_mmap_prot(prot)?;

// Validate mmap flags
SyscallValidator::validate_mmap_flags(flags)?;
```

**Checks**:
- Protection flags valid
- Exactly one of MAP_SHARED or MAP_PRIVATE
- Alignment requirements

## Fuzzing Infrastructure

### Syscall Fuzzer

**Location**: `tests/fuzz/syscall_fuzzer.sh`

Fuzzes syscall interface with random inputs to detect crashes and hangs.

#### Configuration

```bash
# Number of iterations
ITERATIONS=1000000

# Timeout (seconds)
TIMEOUT=3600

# Maximum parallel fuzzers
MAX_PARALLEL=4

# Run fuzzer
./tests/fuzz/syscall_fuzzer.sh
```

#### Features

- Random syscall number generation
- Random argument generation
- Crash detection
- Statistics collection
- JSON reporting

#### Output

```json
{
  "start_time": "2025-11-07T12:00:00+00:00",
  "end_time": "2025-11-07T13:00:00+00:00",
  "iterations": 1000000,
  "completed": 1000000,
  "crashes": 0,
  "hangs": 0,
  "errors": 0,
  "total_time": 3600,
  "rate": 277
}
```

### Validation Test Suite

**Location**: `tests/fuzz/validation_tests.sh`

Tests validation with known-bad inputs:

```bash
./tests/fuzz/validation_tests.sh
```

**Test Categories**:
- Syscall number validation (valid/invalid ranges)
- File descriptor validation (negative, too large)
- Pointer validation (null, kernel space, overflow)
- Buffer size validation (too large, overflow)
- Path validation (too long, no null terminator)
- Flags validation (invalid bits)
- Signal validation (out of range)
- PID validation (invalid values)
- Socket validation (invalid domain/type)
- mmap validation (invalid prot/flags)
- Alignment validation (invalid values)

## GitHub Actions Integration

**Location**: `.github/workflows/fuzz.yml`

### Nightly Fuzzing

Runs automatically at 2am UTC:

```yaml
schedule:
  - cron: '0 2 * * *'
```

### Manual Fuzzing

Trigger manually with custom parameters:

```bash
# Via GitHub Actions UI
Iterations: 1000000
Timeout: 3600
```

### Jobs

1. **syscall-fuzz**: Runs fuzzing campaign
2. **input-validation**: Runs validation tests
3. **security-audit**: Runs cargo-audit and security checks

### Artifacts

- Fuzzing logs (`/tmp/sis-fuzz.log`)
- Statistics (`/tmp/sis-fuzz-stats.json`)
- Crash files (`/tmp/sis-fuzz-crashes/`)

## Security Best Practices

### 1. Input Validation

**Always validate ALL syscall inputs before use:**

```rust
// BAD: No validation
pub fn sys_write(fd: i32, buf: *const u8, len: usize) -> isize {
    // Direct use - UNSAFE!
    let data = unsafe { core::slice::from_raw_parts(buf, len) };
    // ...
}

// GOOD: Validate first
pub fn sys_write(fd: i32, buf: *const u8, len: usize) -> isize {
    // Validate inputs
    SyscallValidator::validate_fd(fd)?;
    let (buf, len) = SyscallValidator::validate_read_buffer(buf, len)?;

    // Safe to use
    let data = unsafe { core::slice::from_raw_parts(buf, len) };
    // ...
}
```

### 2. Integer Overflow Protection

**Use checked arithmetic:**

```rust
// BAD: Potential overflow
let total = count * size;

// GOOD: Checked arithmetic
let total = count.checked_mul(size).ok_or(Errno::EINVAL)?;
```

### 3. Pointer Safety

**Always validate before dereferencing:**

```rust
// Validate pointer is in user space
SyscallValidator::validate_user_ptr(ptr, len)?;

// Safe to dereference
unsafe { ptr.read_volatile() }
```

### 4. Buffer Bounds

**Prevent buffer overflows:**

```rust
// Validate buffer size
if len > MAX_BUFFER_SIZE {
    return Err(Errno::EINVAL);
}

// Check destination has enough space
if dest_len < src_len {
    return Err(Errno::ENOSPC);
}
```

### 5. No Trust in User Input

**Treat all user input as potentially malicious:**

- Validate everything
- Use defensive programming
- Assume worst-case scenarios
- Add assertions in debug builds

## Common Vulnerabilities & Prevention

### 1. Buffer Overflow

**Prevention**:
- Validate buffer sizes
- Use checked indexing
- Bounds checking before copy

```rust
// Validate size first
SyscallValidator::validate_size(size, MAX_BUFFER_SIZE)?;
```

### 2. Integer Overflow

**Prevention**:
- Use `checked_*` arithmetic
- Validate ranges
- Check for wrap-around

```rust
// Prevent overflow
let end = start.checked_add(length).ok_or(Errno::EINVAL)?;
```

### 3. Null Pointer Dereference

**Prevention**:
- Check for null before dereferencing
- Validate pointer is in user space

```rust
// Null check included in validation
SyscallValidator::validate_user_ptr(ptr, len)?;
```

### 4. Race Conditions

**Prevention**:
- Use atomic operations
- Proper locking
- TOCTTOU awareness (Time Of Check To Time Of Use)

### 5. Format String Bugs

**Prevention**:
- Never use user input directly in format strings
- Sanitize all user input

```rust
// BAD
println!("{}", user_string);  // If user_string contains format specifiers

// GOOD
println!("{}", user_string.replace("%", "%%"));
```

### 6. Path Traversal

**Prevention**:
- Validate paths
- Normalize paths
- Check for ".." and symbolic links

```rust
// Validate path doesn't escape root
if path.contains("..") {
    return Err(Errno::EACCES);
}
```

## Security Checklist

### For New Syscalls

- [ ] All inputs validated
- [ ] Pointer validation in place
- [ ] Buffer size checks
- [ ] Integer overflow protection
- [ ] Error handling complete
- [ ] No unsafe without justification
- [ ] Fuzzing tests added
- [ ] Documentation updated

### For Code Review

- [ ] Input validation comprehensive
- [ ] No TOCTTOU vulnerabilities
- [ ] No information leaks
- [ ] Proper error codes returned
- [ ] No uninitialized memory used
- [ ] Safe unsafe code with comments
- [ ] Tests cover edge cases

## Fuzzing Strategy

### 1. Coverage-Guided Fuzzing

Future enhancement: AFL/LibFuzzer integration

### 2. Property-Based Testing

Future enhancement: QuickCheck/PropTest

### 3. Differential Fuzzing

Future enhancement: Compare with Linux syscall behavior

### 4. Stress Testing

Current: Random input fuzzing with validation

## Threat Model

### In Scope

- Malicious userspace programs
- Invalid syscall arguments
- Resource exhaustion attacks
- Privilege escalation attempts

### Out of Scope (for now)

- Hardware attacks (Spectre/Meltdown)
- Side-channel attacks
- Physical attacks
- Supply chain attacks

## Incident Response

### If a Vulnerability is Found

1. **Assess severity**: Use CVSS scoring
2. **Develop fix**: Patch vulnerability
3. **Test fix**: Ensure no regressions
4. **Document**: Update security advisories
5. **Deploy**: Push fix to production
6. **Post-mortem**: Analyze root cause

### Reporting Security Issues

Email: security@sis-kernel.org (placeholder)

Include:
- Description of vulnerability
- Reproduction steps
- Impact assessment
- Suggested fix (if any)

## Future Work

### Phase 4.2: Advanced Fuzzing (TODO)

- AFL integration for coverage-guided fuzzing
- LibFuzzer corpus collection
- Continuous fuzzing infrastructure
- Differential fuzzing vs Linux

### Phase 4.3: Memory Safety (TODO)

- AddressSanitizer integration
- MemorySanitizer for uninitialized reads
- LeakSanitizer for memory leaks
- UndefinedBehaviorSanitizer

### Phase 4.4: Formal Verification (TODO)

- Prove absence of buffer overflows
- Verify pointer safety
- Model check critical paths

## References

- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [CWE/SANS Top 25](https://cwe.mitre.org/top25/)
- [Linux Kernel Security](https://www.kernel.org/doc/html/latest/security/index.html)
- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)

## Testing

### Run Validation Tests

```bash
./tests/fuzz/validation_tests.sh
```

### Run Fuzzer

```bash
ITERATIONS=1000000 ./tests/fuzz/syscall_fuzzer.sh
```

### Run Security Audit

```bash
cargo audit
```

## Metrics

Track security metrics:
- Fuzzing iterations per day
- Crashes found
- Vulnerabilities fixed
- Mean time to patch
- Code coverage of validation

---

**Last Updated**: 2025-11-07
**Version**: 1.0
**Status**: Phase 4 Complete

**See Also**:
- [Production Readiness Plan](./plans/PRODUCTION-READINESS-PLAN.md)
- [Testing Guide](./TESTING.md)
- [Development Guide](./DEVELOPMENT.md)
