# Contributing to SIS Kernel

Thank you for your interest in contributing to SIS Kernel! This guide will help you get started whether you're fixing a bug, adding a feature, forking for your own project, or extending the architecture.

---

## Table of Contents

1. [Quick Start for Contributors](#quick-start-for-contributors)
2. [Ways to Contribute](#ways-to-contribute)
3. [Development Setup](#development-setup)
4. [Making Your First Contribution](#making-your-first-contribution)
5. [Code Review Process](#code-review-process)
6. [Forking for Your Own Project](#forking-for-your-own-project)
7. [Extending the Architecture](#extending-the-architecture)
8. [Communication Channels](#communication-channels)
9. [License and CLA](#license-and-cla)

---

## Quick Start for Contributors

**30-second start:**
```bash
# 1. Fork and clone
git clone https://github.com/YOUR_USERNAME/sis-kernel.git
cd sis-kernel

# 2. Build and test
cargo build --package sis-kernel
cargo test --package sis-kernel

# 3. Run in QEMU
SIS_FEATURES="llm,crypto-real" BRINGUP=1 ./scripts/uefi_run.sh

# 4. Make changes, then format and lint
cargo fmt --all
cargo clippy --all-targets -- -D warnings

# 5. Submit PR
git checkout -b feature/your-feature
git commit -m "feat(scope): description"
git push origin feature/your-feature
```

---

## Ways to Contribute

### 🐛 Bug Reports

Found a bug? Help us fix it!

**Before reporting:**
- [ ] Search existing issues to avoid duplicates
- [ ] Verify the bug on latest `main` branch
- [ ] Collect reproduction steps and logs

**Bug report template:**
```markdown
**Describe the bug**
Clear description of what went wrong.

**To Reproduce**
1. Build with: `SIS_FEATURES="llm" BRINGUP=1 ./scripts/uefi_run.sh`
2. Run command: `stresstest chaos --duration 10000`
3. Observe: OOM panic at line 342

**Expected behavior**
Test should complete without panic, showing graceful degradation.

**Environment**
- OS: macOS 15.0 / Ubuntu 22.04
- Rust version: 1.75.0
- QEMU version: 8.1.0
- Commit: `a2741bc4`

**Logs**
```
[KERNEL] Starting chaos test
[ALLOC] OOM at stress_test.rs:342
panic: allocation failed
```
```

### ✨ Feature Requests

Have an idea? We'd love to hear it!

**Feature request template:**
```markdown
**Problem statement**
What problem does this solve? Who benefits?

**Proposed solution**
How would this work? Any design considerations?

**Alternatives considered**
What other approaches did you consider?

**Additional context**
Links to research, similar implementations, etc.
```

### 📝 Documentation Improvements

Documentation contributions are highly valued!

**Areas needing help:**
- Beginner tutorials and walkthroughs
- Architecture diagrams and visual aids
- API documentation and examples
- Troubleshooting guides
- Translation to other languages

**Documentation PRs:**
- No code changes needed
- Fast review turnaround (<24 hours)
- Great for first-time contributors

### 🔧 Code Contributions

Ready to write code? Here's how:

**Good first issues:**
- Label: `good-first-issue`
- Typically: <100 lines, well-scoped, documented
- Examples: Add test coverage, fix clippy warnings, improve error messages

**Medium complexity:**
- Label: `help-wanted`
- Typically: New feature, refactoring, performance optimization
- Examples: New stress test, additional failure mode, GUI enhancement

**Advanced:**
- Label: `architecture`
- Typically: Core subsystem changes, scheduler modifications
- Requires: Deep understanding of kernel internals
- Examples: New scheduling algorithm, memory allocator improvement

---

## Development Setup

### Prerequisites

**Required:**
- **Rust:** 1.75.0 or newer (nightly recommended for kernel development)
- **QEMU:** 8.0+ with AArch64 support (`qemu-system-aarch64`)
- **Git:** For version control
- **Build tools:** `make`, `gcc` (for building OVMF)

**Optional:**
- **Docker:** For containerized builds
- **ARM hardware:** Raspberry Pi 4, NVIDIA Jetson, or similar for hardware validation
- **IDE:** VS Code with rust-analyzer, or IntelliJ IDEA with Rust plugin

### Installation

**macOS:**
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default nightly

# Install QEMU
brew install qemu

# Install build tools
xcode-select --install
```

**Linux (Ubuntu/Debian):**
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default nightly

# Install QEMU and build tools
sudo apt update
sudo apt install qemu-system-aarch64 build-essential
```

**Verify installation:**
```bash
rustc --version    # Should show 1.75.0+
qemu-system-aarch64 --version  # Should show 8.0+
```

### Environment Setup

**Clone repository:**
```bash
git clone https://github.com/YOUR_USERNAME/sis-kernel.git
cd sis-kernel

# Add upstream remote
git remote add upstream https://github.com/ORIGINAL_OWNER/sis-kernel.git
```

**Build OVMF (UEFI firmware):**
```bash
cd firmware/ovmf-prebuilt
make
cd ../..
```

**Test your setup:**
```bash
# Build kernel
cargo build --package sis-kernel

# Run in QEMU (should boot to shell)
SIS_FEATURES="llm" BRINGUP=1 ./scripts/uefi_run.sh

# If successful, you'll see:
# [BOOT] SIS Kernel starting...
# sis>
```

**IDE Setup (VS Code):**
```bash
# Install extensions
code --install-extension rust-lang.rust-analyzer
code --install-extension tamasfe.even-better-toml

# Open workspace
code .
```

**Configuration (`.vscode/settings.json`):**
```json
{
  "rust-analyzer.cargo.features": ["llm", "ai-ops"],
  "rust-analyzer.checkOnSave.command": "clippy",
  "editor.formatOnSave": true,
  "editor.rulers": [100]
}
```

---

## Making Your First Contribution

### Step 1: Find an Issue

**Browse issues:**
- [Good first issues](https://github.com/OWNER/sis-kernel/issues?q=label%3A%22good+first+issue%22)
- [Help wanted](https://github.com/OWNER/sis-kernel/issues?q=label%3A%22help+wanted%22)
- [Documentation](https://github.com/OWNER/sis-kernel/issues?q=label%3Adocumentation)

**Or propose your own:**
- Open an issue describing what you want to work on
- Wait for maintainer feedback (usually within 48 hours)
- Get assigned to avoid duplicate work

### Step 2: Create a Branch

**Branch naming:**
```bash
# For features
git checkout -b feature/add-new-stress-test

# For bug fixes
git checkout -b fix/oom-regression-issue-42

# For documentation
git checkout -b docs/add-glossary

# For refactoring
git checkout -b refactor/improve-memory-allocation
```

### Step 3: Make Your Changes

**Follow the code conventions:**
- Read [`docs/CODE_CONVENTIONS.md`](./CODE_CONVENTIONS.md)
- Run `cargo fmt --all` before committing
- Fix all clippy warnings: `cargo clippy -- -D warnings`
- Add tests for new functionality
- Update documentation

**Example workflow:**
```bash
# Make changes in your editor
vim crates/kernel/src/stress_test.rs

# Format code
cargo fmt --all

# Check for warnings
cargo clippy --all-targets -- -D warnings

# Run tests
cargo test --package sis-kernel

# Test in QEMU
SIS_FEATURES="llm,crypto-real" BRINGUP=1 ./scripts/uefi_run.sh
```

### Step 4: Write Tests

**Add unit tests:**
```rust
// In your module file (e.g., stress_test.rs)

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_your_new_feature() {
        let config = StressTestConfig::new();
        let result = your_new_function(&config);
        assert!(result.is_ok());
    }
}
```

**Run tests:**
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_your_new_feature

# Run with output
cargo test -- --nocapture
```

### Step 5: Commit Your Changes

**Follow conventional commits:**
```bash
# Stage changes
git add crates/kernel/src/stress_test.rs

# Commit with descriptive message
git commit -m "feat(stress-tests): add exponential event distribution

Implemented 60/30/10 distribution for realistic workload simulation.
Reduces p50 latency from 5ms to 0.5ms while maintaining tail latency.

Closes #42"
```

**Commit message format:**
```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types:** `feat`, `fix`, `docs`, `refactor`, `test`, `perf`, `chore`

**Examples:**
- `feat(gui): add real-time autonomy dashboard`
- `fix(memory): eliminate OOM regression from excessive compaction`
- `docs(readme): add prerequisites section for beginners`
- `test(stress): add coverage for edge cases`

### Step 6: Push and Create PR

**Push to your fork:**
```bash
git push origin feature/your-feature
```

**Create pull request:**
1. Go to GitHub repository
2. Click "Compare & pull request"
3. Fill out PR template (see below)
4. Click "Create pull request"

**PR template:**
```markdown
## Summary
Brief description of changes (1-2 sentences)

## Related Issues
Closes #42
Related to #38

## Changes
- Added X functionality
- Fixed Y bug
- Refactored Z for performance

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests pass
- [ ] Tested in QEMU (10 minute run)
- [ ] No performance regressions

## Screenshots (if applicable)
[Add before/after screenshots for UI changes]

## Checklist
- [x] Code formatted (`cargo fmt`)
- [x] Clippy passes (`cargo clippy -- -D warnings`)
- [x] Tests pass (`cargo test`)
- [x] Documentation updated
- [x] CHANGELOG.md updated (if user-facing change)
```

---

## Code Review Process

### What to Expect

**Review timeline:**
- **Small PRs** (<100 lines): Within 24 hours
- **Medium PRs** (100-500 lines): Within 48 hours
- **Large PRs** (>500 lines): Within 72 hours (consider splitting)

**Reviewer checks:**
1. **Correctness:** Does it solve the problem?
2. **Tests:** Are edge cases covered?
3. **Style:** Follows conventions?
4. **Performance:** Any regressions?
5. **Documentation:** Is behavior explained?

### Responding to Feedback

**Good practices:**
- ✅ Respond to all comments (even if just "Fixed")
- ✅ Push additional commits (don't force-push during review)
- ✅ Ask questions if requirements unclear
- ✅ Mark conversations as resolved when addressed

**Example responses:**
```markdown
> Could you add a test for the edge case where duration is 0?

Good catch! Added test_zero_duration() in latest commit.

> This could be simplified using the ? operator.

Agreed, simplified in commit abc123.

> I'm not sure I understand the tradeoff here. Can you explain?

Sure! The reason we chose approach A over B is...
[Detailed explanation]
```

### After Approval

**Merge process:**
1. All CI checks pass ✅
2. At least one approval from maintainer ✅
3. No unresolved conversations ✅
4. Maintainer merges (or you merge if you have permissions)

**Post-merge:**
- Delete your feature branch
- Update your local main branch
- Celebrate! 🎉

---

## Forking for Your Own Project

Want to use SIS Kernel as a foundation for your own OS? Great!

### Licensing

SIS Kernel is licensed under [MIT License](../LICENSE). This means:

**You CAN:**
- ✅ Use it commercially
- ✅ Modify the code
- ✅ Distribute your version
- ✅ Use it privately

**You MUST:**
- ✅ Include the original license and copyright notice
- ✅ State changes made to the code (if you redistribute)

### Forking Steps

**1. Fork the repository:**
```bash
# On GitHub, click "Fork" button
# Then clone your fork
git clone https://github.com/YOUR_USERNAME/your-os-name.git
cd your-os-name
```

**2. Rebrand:**
```bash
# Update project name in Cargo.toml
sed -i 's/sis-kernel/your-os-name/g' Cargo.toml

# Update README
vim README.md

# Update branding in code
find . -name "*.rs" -exec sed -i 's/SIS Kernel/Your OS Name/g' {} +
```

**3. Customize:**
- Modify boot banner (`crates/kernel/src/main.rs`)
- Update feature flags for your use case
- Remove unused subsystems
- Add your own modules

**4. Maintain attribution:**
Keep the original LICENSE file and add a section:
```markdown
## Attribution

This project is based on SIS Kernel (https://github.com/ORIGINAL/sis-kernel)
Licensed under MIT License. See LICENSE for details.

Modifications made:
- Added XYZ feature
- Removed ABC subsystem
- Modified scheduler for real-time constraints
```

### Staying Updated

**Pull upstream changes:**
```bash
# Add upstream remote (if not already done)
git remote add upstream https://github.com/ORIGINAL/sis-kernel.git

# Fetch upstream changes
git fetch upstream

# Merge into your main branch
git checkout main
git merge upstream/main

# Resolve conflicts if any
# Then push to your fork
git push origin main
```

**Selective merging:**
```bash
# Cherry-pick specific commits
git cherry-pick <commit-hash>

# Or merge specific files
git checkout upstream/main -- crates/kernel/src/stress_test.rs
```

---

## Extending the Architecture

### Adding a New Subsystem

**Example: Adding a new filesystem**

**1. Create module structure:**
```bash
mkdir -p crates/kernel/src/vfs/myfs
touch crates/kernel/src/vfs/myfs/mod.rs
touch crates/kernel/src/vfs/myfs/inode.rs
touch crates/kernel/src/vfs/myfs/superblock.rs
```

**2. Implement VFS interfaces:**
```rust
// crates/kernel/src/vfs/myfs/mod.rs

use crate::vfs::{Inode, InodeOps, FileSystem};

pub struct MyFS {
    // Your filesystem state
}

impl FileSystem for MyFS {
    fn mount(&mut self, device: &str) -> Result<()> {
        // Mount logic
    }

    fn unmount(&mut self) -> Result<()> {
        // Unmount logic
    }
}

impl InodeOps for MyFSInode {
    fn read(&self, offset: usize, buf: &mut [u8]) -> Result<usize> {
        // Read implementation
    }

    fn write(&mut self, offset: usize, buf: &[u8]) -> Result<usize> {
        // Write implementation
    }

    // Implement other InodeOps methods...
}
```

**3. Register with VFS:**
```rust
// In crates/kernel/src/vfs/mod.rs

pub mod myfs;

pub fn register_filesystems() {
    register_filesystem("myfs", Box::new(myfs::MyFS::new()));
}
```

**4. Add tests:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_myfs_mount() {
        let mut fs = MyFS::new();
        assert!(fs.mount("/dev/vda1").is_ok());
    }
}
```

**5. Document:**
```rust
//! MyFS - A custom filesystem implementation
//!
//! MyFS provides [describe features].
//!
//! # Architecture
//! [Explain design decisions]
//!
//! # Usage
//! ```rust
//! let fs = MyFS::new();
//! fs.mount("/dev/vda1")?;
//! ```
```

### Adding a New Shell Command

**Example: Adding `mycommand`**

**1. Create command file:**
```bash
touch crates/kernel/src/shell/mycommand.rs
```

**2. Implement command:**
```rust
// crates/kernel/src/shell/mycommand.rs

use crate::shell::ShellCommand;

pub struct MyCommand;

impl ShellCommand for MyCommand {
    fn name(&self) -> &'static str {
        "mycommand"
    }

    fn help(&self) -> &'static str {
        "mycommand <arg> - Description of what it does"
    }

    fn execute(&self, args: &[&str]) -> Result<(), &'static str> {
        if args.is_empty() {
            return Err("Usage: mycommand <arg>");
        }

        // Implementation
        println!("Executing mycommand with: {}", args[0]);
        Ok(())
    }
}
```

**3. Register command:**
```rust
// In crates/kernel/src/shell/mod.rs

mod mycommand;

pub fn register_commands(shell: &mut Shell) {
    shell.register(Box::new(mycommand::MyCommand));
}
```

**4. Add feature flag (optional):**
```rust
#[cfg(feature = "myfeature")]
mod mycommand;
```

**5. Test:**
```bash
# Build with your feature
SIS_FEATURES="myfeature" BRINGUP=1 ./scripts/uefi_run.sh

# In QEMU shell:
sis> mycommand test
Executing mycommand with: test
```

### Extension Points

**Common extension points:**

| Extension | Location | Interface | Use Case |
|-----------|----------|-----------|----------|
| **Filesystem** | `crates/kernel/src/vfs/` | `FileSystem`, `InodeOps` | Custom storage backend |
| **Scheduler** | `crates/kernel/src/scheduler/` | `SchedulerPolicy` | New scheduling algorithm |
| **Allocator** | `crates/kernel/src/heap/` | `GlobalAlloc` | Custom memory allocator |
| **Driver** | `crates/kernel/src/drivers/` | `Device` trait | Hardware support |
| **Shell Command** | `crates/kernel/src/shell/` | `ShellCommand` | User interaction |
| **Network Protocol** | `crates/kernel/src/net/` | smoltcp traits | Protocol implementation |
| **Stress Test** | `crates/kernel/src/stress_test.rs` | Test config | Validation scenario |

---

## Communication Channels

### GitHub

**For code-related discussions:**
- **Issues:** Bug reports, feature requests
- **Discussions:** General questions, ideas
- **Pull Requests:** Code review

### Community

**Real-time chat:**
- **Discord:** [Join server](https://discord.gg/sis-kernel) - #dev channel for development
- **Slack:** #sis-kernel-dev workspace

**Mailing lists:**
- **Dev list:** dev@sis-kernel.dev - Development discussions
- **Announce:** announce@sis-kernel.dev - Release announcements

**Social media:**
- **Twitter:** @SISKernel - Updates and news
- **Blog:** blog.sis-kernel.dev - Technical deep dives

### Getting Help

**Before asking:**
- [ ] Check README and docs
- [ ] Search existing issues
- [ ] Read error messages carefully
- [ ] Try minimal reproduction

**When asking:**
- ✅ Provide environment details (OS, Rust version, commit)
- ✅ Include full error messages and logs
- ✅ Share reproduction steps
- ✅ Mention what you've already tried

**Response time:**
- **Critical bugs:** Within 24 hours
- **General questions:** Within 48 hours
- **Feature discussions:** Within week

---

## License and CLA

### MIT License

SIS Kernel is licensed under the MIT License. See [LICENSE](../LICENSE) for full text.

**What this means:**
- You can use, modify, and distribute the code
- You must include the original copyright notice
- The software is provided "as is" without warranty

### Contributor License Agreement (CLA)

**Currently:** No CLA required

**By submitting a PR, you agree:**
1. You have the right to contribute the code
2. Your contribution is licensed under MIT
3. You grant the project maintainers rights to use your contribution

**Future:** If project grows, we may introduce a formal CLA for legal clarity.

---

## Recognition

### Hall of Fame

Contributors are recognized in:
- `CONTRIBUTORS.md` - All contributors listed
- Release notes - Feature attributions
- Blog posts - Deep-dive articles featuring contributions

### Becoming a Maintainer

**Path to maintainer:**
1. Multiple quality contributions (5+ PRs merged)
2. Consistent involvement (3+ months)
3. Code review participation
4. Community engagement (helping others)
5. Invitation from existing maintainers

**Maintainer responsibilities:**
- Review PRs (within 48 hours)
- Triage issues
- Mentor new contributors
- Make architectural decisions
- Maintain code quality

---

## FAQ

**Q: I'm new to Rust. Can I still contribute?**
A: Yes! Start with documentation or good-first-issue labeled issues. We're happy to mentor new Rustaceans.

**Q: How do I get my PR reviewed faster?**
A: Keep PRs small (<500 lines), add tests, follow conventions, and provide clear description.

**Q: Can I work on an issue that's not assigned?**
A: Comment on the issue expressing interest. Wait for maintainer confirmation before starting to avoid duplicate work.

**Q: My PR was rejected. What now?**
A: Don't be discouraged! Ask for clarification, understand the reasoning, and consider alternative approaches.

**Q: Can I use SIS Kernel for my thesis/research?**
A: Absolutely! We'd love to hear about your research. Please cite the project and share your findings.

**Q: How do I report a security vulnerability?**
A: Email security@sis-kernel.dev (do NOT open public issue). We'll respond within 24 hours.

---

## Thank You!

Your contributions make SIS Kernel better for everyone. Whether you're fixing a typo, adding a feature, or just asking questions - you're helping build something great.

**Resources:**
- [Code Conventions](./CODE_CONVENTIONS.md)
- [Architecture Overview](../README.md#architecture-overview)
- [Testing Guide](./TESTING.md)
- [Performance Tuning](./PERFORMANCE.md)

**Questions?** Open an issue or join our Discord!

Happy hacking! 🚀
