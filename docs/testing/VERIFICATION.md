# Formal Verification Guide

This document describes the formal verification setup for the SIS Kernel project using Kani and Prusti.

## Overview

The SIS Kernel uses two complementary formal verification tools:

1. **Kani** - Model checker based on CBMC for verifying Rust code
2. **Prusti** - Verification tool based on Viper for checking functional correctness

## Prerequisites

### Installing Kani

```bash
# Install Kani verifier
cargo install --locked kani-verifier
cargo kani setup

# Verify installation
cargo kani --version
```

**Documentation**: https://model-checking.github.io/kani/

### Installing Prusti

```bash
# Install Prusti (requires Java)
cargo install prusti-cli

# Verify installation
cargo prusti --version
```

**Documentation**: https://viperproject.github.io/prusti-dev/

## Running Verification

### Kani Model Checking

Kani can verify properties like:
- Absence of panics, overflows, and undefined behavior
- Memory safety properties
- Custom assertions and invariants

```bash
# Run Kani on a specific function
cargo kani --harness my_test_harness

# Run all Kani proofs in testing crate
cd crates/testing
cargo kani

# Run with specific features
cargo kani --features formal-verification
```

#### Example Kani Proof

```rust
#[cfg(kani)]
#[kani::proof]
fn verify_tokenizer_roundtrip() {
    let input: Vec<u8> = kani::any();
    kani::assume(input.len() < 1000); // Bounded model checking

    let tokenizer = BpeTokenizer::new(vocab_size);
    let tokens = tokenizer.encode(&input);
    let output = tokenizer.decode(&tokens);

    // Property: encode-decode should be identity for valid inputs
    assert_eq!(input, output);
}
```

### Prusti Verification

Prusti can verify properties like:
- Pre/post-conditions
- Loop invariants
- Absence of panics
- Functional correctness

```bash
# Run Prusti on testing crate
cd crates/testing
cargo prusti

# Run with specific features
cargo prusti --features formal-verification
```

#### Example Prusti Contracts

```rust
use prusti_contracts::*;

#[requires(tokens.len() > 0)]
#[ensures(result.len() > 0)]
fn decode_tokens(tokens: &[u32]) -> Vec<u8> {
    // Implementation with guaranteed non-empty output
    tokens.iter()
        .flat_map(|&token| token_to_bytes(token))
        .collect()
}

#[pure]
#[ensures(result >= 0)]
fn token_priority(token_pair: (u32, u32)) -> usize {
    // Pure function for priority lookup
    self.merge_priority.get(&token_pair).copied().unwrap_or(usize::MAX)
}
```

## Verification Targets

### Hardware Backend Verification

The `HardwareBackend` trait and implementations are good candidates for verification:

**Properties to verify**:
- `initialize()` followed by `is_alive()` returns true
- `send_command()` after `shutdown()` returns error
- No data races in async operations
- Timeout bounds are respected

**Example verification harness**:

```rust
#[cfg(kani)]
#[kani::proof]
fn verify_backend_lifecycle() {
    let config = BackendConfig::default();
    let mut backend = QemuBackend::new(0, config, manager);

    // Verify initialization works
    let init_result = backend.initialize().await;
    assert!(init_result.is_ok());
    assert!(backend.is_alive().await);

    // Verify shutdown works
    let shutdown_result = backend.shutdown().await;
    assert!(shutdown_result.is_ok());
    assert!(!backend.is_alive().await);
}
```

### Tokenizer Verification

The BPE tokenizer has well-defined mathematical properties:

**Properties to verify**:
- Roundtrip property: `decode(encode(x)) == x`
- Merge priority ordering is respected
- No buffer overflows in token processing
- Token vocabulary bounds are maintained

### Memory Safety

Verify memory safety properties across all test infrastructure:

**Properties to verify**:
- No use-after-free
- No double-free
- No buffer overflows
- No null pointer dereferences
- No data races

## Continuous Integration

### GitHub Actions Workflow

Add verification to CI/CD pipeline:

```yaml
name: Formal Verification

on: [push, pull_request]

jobs:
  kani:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable

      - name: Install Kani
        run: |
          cargo install --locked kani-verifier
          cargo kani setup

      - name: Run Kani verification
        run: |
          cd crates/testing
          cargo kani --features formal-verification

  prusti:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@nightly

      - name: Install Java (Prusti dependency)
        uses: actions/setup-java@v3
        with:
          distribution: 'temurin'
          java-version: '11'

      - name: Install Prusti
        run: cargo install prusti-cli

      - name: Run Prusti verification
        run: |
          cd crates/testing
          cargo prusti --features formal-verification
```

## Best Practices

### 1. Start Small

Begin with simple properties and gradually increase complexity:

1. **Level 1**: No panics, no overflows
2. **Level 2**: Memory safety, no undefined behavior
3. **Level 3**: Functional correctness (roundtrip, invariants)
4. **Level 4**: Complex protocols (state machines, async)

### 2. Bounded Model Checking

Kani uses bounded model checking, so limit input sizes:

```rust
#[kani::proof]
fn bounded_property() {
    let input_size: usize = kani::any();
    kani::assume(input_size <= 100); // Reasonable bound

    let input = vec![0u8; input_size];
    // ... verify property on bounded input
}
```

### 3. Separate Verification Code

Keep verification harnesses separate from production code:

```
crates/testing/
├── src/
│   ├── lib.rs
│   ├── backends/
│   └── ...
└── verification/
    ├── kani/
    │   ├── backend_proofs.rs
    │   └── tokenizer_proofs.rs
    └── prusti/
        ├── backend_contracts.rs
        └── tokenizer_contracts.rs
```

### 4. Document Assumptions

Always document what you're assuming and verifying:

```rust
/// # Verification Properties
///
/// - **Memory Safety**: No buffer overflows, use-after-free
/// - **Functional Correctness**: Roundtrip property holds
/// - **Assumptions**: Input size bounded to 1000 bytes
#[kani::proof]
fn verify_encode_decode_roundtrip() {
    // ...
}
```

## Limitations

### Kani Limitations

- **Bounded model checking**: Must bound loops and recursion
- **Async verification**: Limited support for async/await
- **Performance**: Large state spaces can be slow
- **External dependencies**: Cannot verify system calls

### Prusti Limitations

- **Requires nightly Rust**: Depends on compiler internals
- **Limited trait support**: Some trait features not fully supported
- **Learning curve**: Requires understanding separation logic
- **Specification overhead**: Writing contracts takes time

## Troubleshooting

### Kani Times Out

```bash
# Increase timeout
cargo kani --timeout 600 --harness my_test

# Reduce input bounds
kani::assume(input.len() <= 50); // Smaller bound
```

### Prusti Fails to Verify

```bash
# Enable debug output
PRUSTI_LOG=debug cargo prusti

# Check specific function
cargo prusti --function my_function_name
```

### Java Not Found (Prusti)

```bash
# Install OpenJDK 11 or later
sudo apt-get install openjdk-11-jdk

# Or use package manager
brew install openjdk@11
```

## Resources

- **Kani Book**: https://model-checking.github.io/kani/
- **Prusti User Guide**: https://viperproject.github.io/prusti-dev/user-guide/
- **Rust Verification Tools**: https://rust-formal-methods.github.io/
- **CBMC Documentation**: https://www.cprover.org/cbmc/

## Examples from SIS Kernel

See `crates/testing/src/bin/formal_verification_runner.rs` for example verification harnesses.

Run verification tests:

```bash
# Run formal verification binary
cargo run -p sis-testing --bin formal-verification-runner --features formal-verification

# Run with specific backend
cargo run -p sis-testing --bin formal-verification-runner --features formal-verification -- --backend kani
```

## Contributing

When adding new code that should be verified:

1. Add verification harnesses in `verification/kani/` or `verification/prusti/`
2. Document properties being verified
3. Ensure CI passes verification checks
4. Update this document if adding new verification patterns
