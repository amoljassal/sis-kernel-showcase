# Security Policy

## Overview

SIS Kernel takes security seriously. This document outlines our security practices, how to report vulnerabilities, and what security features are implemented.

---

## Supported Versions

| Version | Supported          | Status |
| ------- | ------------------ | ------ |
| main    | :white_check_mark: | Active development, security patches applied |
| < 1.0   | :x:                | Experimental, no security guarantees |

**Recommendation:** Always use the latest commit from `main` branch for most recent security fixes.

---

## Security Features Implemented

### ✅ Cryptographic Verification
- **Ed25519 Signatures:** Model packages verified with public-key cryptography
- **SHA-256 Hashing:** Integrity checks for all model binaries
- **Implementation:** `crates/kernel/src/crypto/` (feature: `crypto-real`)
- **Status:** Production-ready when `SIS_ED25519_PUBKEY` is set at build time

### ✅ Memory Safety
- **Rust Language:** Memory-safe by default, prevents buffer overflows and use-after-free
- **Unsafe Code Audited:** All `unsafe` blocks documented with safety invariants
- **Heap Bounds Checking:** Allocation validation in `crates/kernel/src/heap/`
- **ASLR:** Address Space Layout Randomization for process memory

### ✅ Access Control
- **Credential System:** UID/GID-based permissions (`crates/kernel/src/security/`)
- **Audit Logging:** All model loads and security events logged
- **Decision Tracing:** 1024-entry ring buffer for forensic analysis
- **Incident Bundles:** Automatic export for security events

### ✅ Isolation & Containment
- **VFS Security:** Filesystem permissions and ownership
- **Process Isolation:** Memory protection via MMU
- **Network Security:** Packet filtering (basic implementation)
- **Resource Limits:** Memory quotas and allocation caps

### 🚧 Planned Security Features
- **SELinux-style MAC:** Mandatory Access Control (roadmap)
- **Capabilities:** Fine-grained privilege separation (roadmap)
- **TLS Support:** Network encryption with rustls (roadmap)
- **Secure Boot:** UEFI secure boot integration (future)

---

## Known Security Limitations

| Area | Limitation | Mitigation | Priority |
|------|------------|------------|----------|
| **Kernel Privilege** | All kernel code runs at EL1 (privileged) | Minimize unsafe code, extensive testing | High |
| **Network Stack** | No TLS/encryption yet | Use trusted networks only | Medium |
| **User Auth** | Basic UID/GID, no password verification | Physical security required | Medium |
| **Side Channels** | No Spectre/Meltdown mitigations | Trusted execution environment assumed | Low |
| **Fuzzing** | Not yet fuzzed with AFL/LibFuzzer | Planned in roadmap | High |

**Important:** SIS Kernel is currently intended for research, development, and demonstration purposes. Not recommended for production deployment in hostile environments without additional hardening.

---

## Reporting a Vulnerability

### How to Report

**DO NOT** open a public GitHub issue for security vulnerabilities.

Instead:

1. **Email:** security@sis-kernel.dev (preferred)
   - Subject: `[SECURITY] Brief description`
   - Include: Detailed description, reproduction steps, impact assessment

2. **GitHub Security Advisory:** Use [private vulnerability reporting](https://github.com/YOUR_USERNAME/sis-kernel/security/advisories/new)

3. **PGP Encrypted:** If you prefer encrypted communication, use our PGP key:
   ```
   Fingerprint: [YOUR_PGP_FINGERPRINT]
   Key: Available at https://YOUR_USERNAME.github.io/pgp.txt
   ```

### What to Include

Please provide:
- **Description:** Clear explanation of the vulnerability
- **Location:** File path and line number (if known)
- **Impact:** What an attacker could do (privilege escalation, DoS, info disclosure, etc.)
- **Reproduction:** Step-by-step instructions to reproduce
- **Environment:** OS, QEMU version, build configuration
- **Proof of Concept:** Code or commands demonstrating the issue (if applicable)
- **Suggested Fix:** Optional, but appreciated

### Example Report

```markdown
**Vulnerability:** Buffer overflow in VFS path parsing

**Location:** `crates/kernel/src/vfs/path.rs:142`

**Impact:** Attacker can provide long path (>4096 bytes) causing buffer overflow,
potentially leading to code execution or kernel panic.

**Reproduction:**
1. Build kernel with VFS enabled
2. Run: `sis> cat /$(python3 -c 'print("A"*5000)')`
3. Observe: Kernel panic or corruption

**Environment:** QEMU 8.1.0, macOS 15.0, commit abc123

**Suggested Fix:** Add bounds check before copying to path buffer
```

---

## Response Timeline

We aim to:
- **Acknowledge** your report within **24 hours**
- **Provide initial assessment** within **72 hours**
- **Release a fix** within **7-14 days** for critical issues
- **Publish advisory** after fix is available

### Severity Classification

| Severity | Definition | Response Time |
|----------|------------|---------------|
| **Critical** | Remote code execution, privilege escalation to kernel | 7 days |
| **High** | Local privilege escalation, DoS, memory corruption | 14 days |
| **Medium** | Information disclosure, logic errors | 30 days |
| **Low** | Minor issues, edge cases | 90 days |

---

## Disclosure Policy

We follow **coordinated disclosure**:

1. You report the vulnerability privately
2. We confirm and develop a fix
3. We notify you when fix is ready
4. We publish a security advisory with credit to you (if desired)
5. We release the fix and update this document

**Public disclosure before fix:** If you plan to publicly disclose, please give us at least 90 days to develop and release a fix.

---

## Security Advisories

| Advisory ID | Date | Severity | Description | Fix Commit |
|-------------|------|----------|-------------|------------|
| *None yet* | - | - | No security issues reported to date | - |

**Subscribe:** Watch this repository to receive notifications of future security advisories.

---

## Security Testing

### Current Testing

✅ **Manual Code Review**
- All unsafe blocks reviewed and documented
- Clippy lints enforced (`-D warnings`)
- Memory safety validation

✅ **Stress Testing**
- 7 stress tests including chaos engineering
- Failure injection (0-100% configurable)
- OOM recovery testing

✅ **Integration Testing**
- 1000+ boot cycles without crash
- 24-hour stability testing
- Edge case validation

### Planned Testing

🚧 **Fuzzing** (High Priority)
- AFL/LibFuzzer integration for VFS, network, syscalls
- Continuous fuzzing in CI/CD

🚧 **Static Analysis**
- KASAN (Kernel Address Sanitizer)
- MIRI for unsafe code validation
- Formal verification for critical paths

🚧 **External Audit**
- Third-party security audit (when funding available)
- Penetration testing
- CVE assignment for discovered issues

---

## Security Best Practices for Users

### Build Security

```bash
# Always verify source integrity
git clone https://github.com/YOUR_USERNAME/sis-kernel.git
cd sis-kernel
git verify-commit HEAD  # If commits are signed

# Use crypto-real feature for production
export SIS_ED25519_PUBKEY=0x<your-64-hex-pubkey>
SIS_FEATURES="llm,ai-ops,crypto-real" BRINGUP=1 ./scripts/uefi_run.sh

# Verify build artifacts
sha256sum crates/kernel/target/aarch64-unknown-none/debug/sis_kernel
```

### Runtime Security

```bash
# Enable audit logging
sis> autoctl on

# Check model signatures
sis> modelctl verify v1

# Export decision traces for review
sis> tracectl export --recent 100

# Monitor security events
sis> grep "SECURITY" /var/log/kernel.log
```

### Network Security

```bash
# Isolate QEMU network
# Use user networking (NAT), not bridge mode in untrusted environments
QEMU_NET_MODE=user ./scripts/uefi_run.sh

# Firewall rules (if running on real hardware)
# Block all incoming except SSH/monitoring
```

---

## Security Contact

- **Primary:** security@sis-kernel.dev
- **Backup:** Issue a private security advisory on GitHub
- **Response Time:** Within 24 hours for critical issues

---

## Attribution

We appreciate responsible disclosure and will credit researchers (with your permission) in:
- Security advisories
- Release notes
- Hall of Fame (if you've reported multiple issues)

**Thank you for helping keep SIS Kernel secure!**

---

## Additional Resources

- [OWASP Secure Coding Practices](https://owasp.org/www-project-secure-coding-practices-quick-reference-guide/)
- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)
- [CWE Top 25 Most Dangerous Software Weaknesses](https://cwe.mitre.org/top25/)
- [CVE Database](https://cve.mitre.org/)

---

**Last Updated:** 2025-01-11
**Version:** 1.0
