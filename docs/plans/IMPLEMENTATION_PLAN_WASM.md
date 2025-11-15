# WebAssembly Runtime Integration Plan

**Status**: Planning
**Priority**: P1 - Replaces custom SISLang effort
**Timeline**: 6-8 weeks
**Dependencies**: AgentSys (Phase 9), VFS, Capability System

---

## Executive Summary

This plan describes the integration of WebAssembly (WASM) into the SIS Kernel as the primary scripting/extension language, replacing the previously planned custom SISLang. WASM provides:

- **Industry-standard security**: Sandboxed execution with proven isolation
- **Multi-language support**: Rust, C/C++, AssemblyScript, TinyGo, and more
- **Existing ecosystem**: Toolchains, libraries, and community support
- **Deterministic execution**: Ideal for resource budgeting and reproducibility
- **WASI integration**: Standard system interface for capabilities

The WASM runtime will integrate deeply with:
- **AgentSys**: Agents as WASM modules with capability-based security
- **LLM subsystem**: LoRA adapters and inference as WASM modules
- **VFS**: File access through WASI
- **Resource management**: CPU/memory budgets, pacing, quotas

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Runtime Selection](#runtime-selection)
3. [Milestone 0: Runtime Integration](#milestone-0-runtime-integration)
4. [Milestone 1: WASI Implementation](#milestone-1-wasi-implementation)
5. [Milestone 2: Capability System Integration](#milestone-2-capability-system-integration)
6. [Milestone 3: AgentSys WASM Bindings](#milestone-3-agentsys-wasm-bindings)
7. [Milestone 4: Resource Management](#milestone-4-resource-management)
8. [Milestone 5: LLM Integration](#milestone-5-llm-integration)
9. [Milestone 6: Package System](#milestone-6-package-system)
10. [Milestone 7: Testing & Validation](#milestone-7-testing--validation)
11. [Timeline](#timeline)
12. [Migration from SISLang](#migration-from-sislang)
13. [References](#references)

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    User Applications                        │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
│  │ Agent 1  │  │ Agent 2  │  │ LLM Mod  │  │ User App │  │
│  │ (WASM)   │  │ (WASM)   │  │ (WASM)   │  │ (WASM)   │  │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘  │
└─────────────────────────────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────┐
│              WASM Runtime (wasmi/wasm3)                     │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  Module Loader  │  JIT/Interp  │  Memory Manager    │  │
│  └──────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  Resource Metering  │  Fuel System  │  Gas Limits    │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────┐
│                  WASI Implementation                        │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────────┐  │
│  │ wasi_fd │  │wasi_path│  │wasi_env │  │ wasi_random │  │
│  └─────────┘  └─────────┘  └─────────┘  └─────────────┘  │
└─────────────────────────────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────┐
│              Capability Bridge Layer                        │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  WASI → Kernel Capability Mapping                    │  │
│  │  - fd_read    → CAP_FILE_READ                        │  │
│  │  - fd_write   → CAP_FILE_WRITE                       │  │
│  │  - path_open  → CAP_FILE_OPEN                        │  │
│  │  - sock_send  → CAP_NET_SEND                         │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────┐
│                   Kernel Services                           │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
│  │ VFS      │  │ Network  │  │ AgentSys │  │   LLM    │  │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### Key Design Principles

1. **No Trust Boundary Violations**: WASM modules cannot escape sandbox
2. **Capability-Based Security**: All host access through explicit capabilities
3. **Deterministic Execution**: Gas metering ensures predictable resource usage
4. **Multi-Tenancy**: Multiple WASM instances isolated from each other
5. **Zero-Copy Where Possible**: Linear memory access for performance

---

## Runtime Selection

### Evaluation Criteria

| Runtime     | Size   | no_std | JIT | Interp | Fuel | WASI | Status       |
|-------------|--------|--------|-----|--------|------|------|--------------|
| **wasmi**   | ~150KB | ✅     | ❌  | ✅     | ✅   | ✅   | Recommended  |
| wasm3       | ~100KB | ✅     | ❌  | ✅     | ✅   | ⚠️   | Alternative  |
| wasmtime    | ~5MB   | ❌     | ✅  | ✅     | ✅   | ✅   | Too large    |

### Decision: wasmi

**Rationale**:
- **no_std compatible**: Works in kernel environment
- **Small footprint**: ~150KB suitable for embedded
- **Fuel metering**: Built-in gas/resource tracking
- **WASI support**: Can implement standard WASI interface
- **Pure Rust**: Memory-safe, integrates well with kernel
- **Active maintenance**: Regular updates, security patches

### Alternative: wasm3

If wasmi proves too heavy:
- **Smaller**: ~100KB binary size
- **Faster interpretation**: Optimized bytecode interpreter
- **C-based**: May require more unsafe code
- **WASI**: Limited support, would need custom implementation

---

## Milestone 0: Runtime Integration

**Duration**: 1 week
**Goal**: Get wasmi running in the kernel

### M0.1: Add wasmi Dependency

```toml
# crates/kernel/Cargo.toml
[dependencies]
wasmi = { version = "0.31", default-features = false, features = ["no_std"] }
wasmi-core = { version = "0.13", default-features = false }

# For WASM binary parsing
wasmparser = { version = "0.121", default-features = false }
```

### M0.2: Create WASM Subsystem

```rust
// crates/kernel/src/wasm/mod.rs
pub mod runtime;
pub mod module;
pub mod instance;
pub mod error;

use wasmi::{Engine, Store, Module, Instance};
use alloc::vec::Vec;
use alloc::collections::BTreeMap;

/// Global WASM engine (shared across all modules)
static mut WASM_ENGINE: Option<Engine> = None;

/// Initialize WASM subsystem
pub unsafe fn init() -> Result<(), WasmError> {
    info!("WASM: Initializing WebAssembly runtime");

    // Create wasmi engine with default config
    let engine = Engine::default();
    WASM_ENGINE = Some(engine);

    info!("WASM: Runtime initialized");
    Ok(())
}

/// Get reference to global WASM engine
pub fn engine() -> Result<&'static Engine, WasmError> {
    unsafe {
        WASM_ENGINE.as_ref().ok_or(WasmError::NotInitialized)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmError {
    NotInitialized,
    InvalidModule,
    InstantiationFailed,
    ExecutionFailed,
    OutOfMemory,
    OutOfFuel,
    TrapOccurred,
    CapabilityDenied,
}
```

### M0.3: Module Loading

```rust
// crates/kernel/src/wasm/module.rs
use wasmi::{Engine, Module};
use alloc::vec::Vec;

/// WASM module wrapper
pub struct WasmModule {
    module: Module,
    bytecode_hash: [u8; 32],  // SHA-256 of original bytecode
    name: Option<&'static str>,
}

impl WasmModule {
    /// Load WASM module from bytecode
    pub fn from_bytecode(bytecode: &[u8], name: Option<&'static str>) -> Result<Self, WasmError> {
        // Validate WASM bytecode
        wasmparser::validate(bytecode)
            .map_err(|_| WasmError::InvalidModule)?;

        // Compute hash for integrity
        let bytecode_hash = crate::crypto::sha256(bytecode);

        // Parse with wasmi
        let engine = crate::wasm::engine()?;
        let module = Module::new(engine, bytecode)
            .map_err(|_| WasmError::InvalidModule)?;

        Ok(Self {
            module,
            bytecode_hash,
            name,
        })
    }

    /// Get module name
    pub fn name(&self) -> Option<&str> {
        self.name
    }

    /// Get bytecode hash (for verification)
    pub fn hash(&self) -> &[u8; 32] {
        &self.bytecode_hash
    }
}
```

### M0.4: Instance Creation

```rust
// crates/kernel/src/wasm/instance.rs
use wasmi::{Store, Instance, Linker, Caller};
use alloc::collections::BTreeMap;

/// WASM instance with isolated state
pub struct WasmInstance {
    store: Store<InstanceContext>,
    instance: Instance,
    fuel_limit: u64,
}

/// Per-instance context (host state)
pub struct InstanceContext {
    /// Instance ID
    id: u32,

    /// Capabilities granted to this instance
    capabilities: CapabilitySet,

    /// Open file descriptors
    fds: BTreeMap<u32, FileHandle>,

    /// Environment variables
    env: BTreeMap<&'static str, &'static str>,

    /// Resource usage statistics
    stats: InstanceStats,
}

#[derive(Default)]
pub struct InstanceStats {
    pub fuel_consumed: u64,
    pub memory_used: usize,
    pub syscalls: u64,
    pub execution_time_ns: u64,
}

impl WasmInstance {
    /// Create new instance from module
    pub fn new(
        module: &WasmModule,
        capabilities: CapabilitySet,
        fuel_limit: u64,
    ) -> Result<Self, WasmError> {
        // Create store with fuel metering
        let engine = crate::wasm::engine()?;
        let mut store = Store::new(engine, InstanceContext {
            id: crate::wasm::next_instance_id(),
            capabilities,
            fds: BTreeMap::new(),
            env: BTreeMap::new(),
            stats: InstanceStats::default(),
        });

        // Set fuel limit
        store.set_fuel(fuel_limit).map_err(|_| WasmError::OutOfFuel)?;

        // Create linker for host functions
        let mut linker = Linker::new(engine);

        // Register WASI imports (will be implemented in M1)
        register_wasi_imports(&mut linker)?;

        // Instantiate module
        let instance = linker
            .instantiate(&mut store, module.module())
            .map_err(|_| WasmError::InstantiationFailed)?
            .start(&mut store)
            .map_err(|_| WasmError::ExecutionFailed)?;

        Ok(Self {
            store,
            instance,
            fuel_limit,
        })
    }

    /// Call exported function
    pub fn call(&mut self, func_name: &str, params: &[wasmi::Value]) -> Result<Option<wasmi::Value>, WasmError> {
        // Get exported function
        let func = self.instance
            .get_export(&self.store, func_name)
            .and_then(|e| e.into_func())
            .ok_or(WasmError::ExecutionFailed)?;

        // Prepare result buffer
        let mut results = [wasmi::Value::I32(0)];

        // Call function with fuel tracking
        let start_fuel = self.store.get_fuel().unwrap_or(0);
        let start_time = crate::time::current_time_ns();

        func.call(&mut self.store, params, &mut results)
            .map_err(|_| WasmError::TrapOccurred)?;

        // Update statistics
        let end_fuel = self.store.get_fuel().unwrap_or(0);
        let end_time = crate::time::current_time_ns();

        self.store.data_mut().stats.fuel_consumed += start_fuel - end_fuel;
        self.store.data_mut().stats.execution_time_ns += end_time - start_time;

        Ok(Some(results[0]))
    }

    /// Get remaining fuel
    pub fn remaining_fuel(&self) -> u64 {
        self.store.get_fuel().unwrap_or(0)
    }

    /// Get instance statistics
    pub fn stats(&self) -> &InstanceStats {
        &self.store.data().stats
    }
}

/// Stub for WASI import registration (implemented in M1)
fn register_wasi_imports(linker: &mut Linker<InstanceContext>) -> Result<(), WasmError> {
    // Will be implemented in Milestone 1
    Ok(())
}
```

### M0.5: Basic Test

```rust
// crates/kernel/src/wasm/tests.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_simple_module() {
        // Simple WASM module: (module (func (export "add") (param i32 i32) (result i32) local.get 0 local.get 1 i32.add))
        let bytecode = &[
            0x00, 0x61, 0x73, 0x6d, // Magic
            0x01, 0x00, 0x00, 0x00, // Version
            // ... rest of bytecode
        ];

        let module = WasmModule::from_bytecode(bytecode, Some("test_add")).unwrap();
        assert_eq!(module.name(), Some("test_add"));
    }

    #[test]
    fn test_fuel_metering() {
        // Load test module
        let module = load_test_module();

        // Create instance with 10000 fuel
        let mut instance = WasmInstance::new(&module, CapabilitySet::empty(), 10000).unwrap();

        // Call function
        let result = instance.call("compute", &[]).unwrap();

        // Check fuel was consumed
        assert!(instance.stats().fuel_consumed > 0);
        assert!(instance.remaining_fuel() < 10000);
    }
}
```

### M0.6: Shell Commands

```rust
// crates/kernel/src/shell/wasm_helpers.rs
impl Shell {
    /// Load and run WASM module
    pub(crate) fn wasm_run_cmd(&self, args: &[u8]) {
        // Parse: wasm-run <path> <function> [args...]
        let parts: Vec<&[u8]> = args.split(|&b| b == b' ').collect();

        if parts.len() < 2 {
            uart_print(b"Usage: wasm-run <path> <function> [args...]\n");
            return;
        }

        let path = parts[0];
        let func_name = parts[1];

        // Load WASM bytecode from VFS
        let bytecode = match self.load_file(path) {
            Ok(data) => data,
            Err(_) => {
                uart_print(b"Error: Failed to load module\n");
                return;
            }
        };

        // Create module
        let module = match WasmModule::from_bytecode(&bytecode, None) {
            Ok(m) => m,
            Err(_) => {
                uart_print(b"Error: Invalid WASM module\n");
                return;
            }
        };

        // Create instance with default capabilities
        let caps = CapabilitySet::default();
        let fuel_limit = 1_000_000;

        let mut instance = match WasmInstance::new(&module, caps, fuel_limit) {
            Ok(i) => i,
            Err(_) => {
                uart_print(b"Error: Failed to instantiate module\n");
                return;
            }
        };

        // Call function
        let func_name_str = core::str::from_utf8(func_name).unwrap_or("main");
        match instance.call(func_name_str, &[]) {
            Ok(Some(result)) => {
                uart_print(b"Result: ");
                self.print_wasm_value(result);
                uart_print(b"\n");
            }
            Ok(None) => {
                uart_print(b"Function executed (no return value)\n");
            }
            Err(e) => {
                uart_print(b"Error: Execution failed\n");
            }
        }

        // Print statistics
        let stats = instance.stats();
        uart_print(b"Fuel consumed: ");
        self.print_number(stats.fuel_consumed);
        uart_print(b"\nExecution time: ");
        self.print_number(stats.execution_time_ns / 1000);
        uart_print(b" us\n");
    }

    fn print_wasm_value(&self, value: wasmi::Value) {
        match value {
            wasmi::Value::I32(v) => self.print_number_signed(v as i64),
            wasmi::Value::I64(v) => self.print_number_signed(v),
            wasmi::Value::F32(v) => uart_print(b"<f32>"),
            wasmi::Value::F64(v) => uart_print(b"<f64>"),
        }
    }
}
```

### M0 Deliverables

- [ ] wasmi integrated into kernel build
- [ ] Module loading from bytecode
- [ ] Instance creation with fuel metering
- [ ] Basic function calls work
- [ ] Shell command `wasm-run` to execute modules
- [ ] Unit tests for module loading and execution

---

## Milestone 1: WASI Implementation

**Duration**: 2 weeks
**Goal**: Implement WASI preview1 for file/network/environment access

### M1.1: WASI Preview1 Specification

WASI (WebAssembly System Interface) provides a standard API for WASM modules to interact with the host system. We'll implement WASI preview1, focusing on:

- **File I/O**: `fd_read`, `fd_write`, `fd_close`, `path_open`
- **File metadata**: `fd_stat_get`, `path_filestat_get`
- **Environment**: `environ_get`, `environ_sizes_get`
- **Random**: `random_get`
- **Clock**: `clock_time_get`
- **Process**: `proc_exit`

### M1.2: WASI Types

```rust
// crates/kernel/src/wasm/wasi/types.rs

/// WASI file descriptor
pub type Fd = u32;

/// WASI errno codes
#[repr(u16)]
#[derive(Debug, Clone, Copy)]
pub enum Errno {
    Success = 0,
    TooBig = 1,
    Access = 2,
    AddrInUse = 3,
    AddrNotAvail = 4,
    // ... (full WASI errno set)
    NotCapable = 76,
}

impl Errno {
    pub fn as_u16(self) -> u16 {
        self as u16
    }
}

/// File descriptor flags
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct FdFlags(u16);

impl FdFlags {
    pub const APPEND: Self = Self(1 << 0);
    pub const DSYNC: Self = Self(1 << 1);
    pub const NONBLOCK: Self = Self(1 << 2);
    pub const RSYNC: Self = Self(1 << 3);
    pub const SYNC: Self = Self(1 << 4);
}

/// Open flags
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct OFlags(u16);

impl OFlags {
    pub const CREAT: Self = Self(1 << 0);
    pub const DIRECTORY: Self = Self(1 << 1);
    pub const EXCL: Self = Self(1 << 2);
    pub const TRUNC: Self = Self(1 << 3);
}

/// Rights (capabilities) for file descriptors
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Rights(u64);

impl Rights {
    pub const FD_READ: Self = Self(1 << 1);
    pub const FD_WRITE: Self = Self(1 << 6);
    pub const PATH_OPEN: Self = Self(1 << 13);
    pub const PATH_CREATE_FILE: Self = Self(1 << 15);
    // ... (full rights set)
}

/// File metadata
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FileStat {
    pub dev: u64,
    pub ino: u64,
    pub filetype: FileType,
    pub nlink: u64,
    pub size: u64,
    pub atim: u64,
    pub mtim: u64,
    pub ctim: u64,
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum FileType {
    Unknown = 0,
    BlockDevice = 1,
    CharacterDevice = 2,
    Directory = 3,
    RegularFile = 4,
    SocketDGram = 5,
    SocketStream = 6,
    SymbolicLink = 7,
}
```

### M1.3: File Descriptor Table

```rust
// crates/kernel/src/wasm/wasi/fd_table.rs
use alloc::collections::BTreeMap;
use crate::vfs::FileHandle;

/// File descriptor table for WASM instance
pub struct FdTable {
    /// Next FD to allocate
    next_fd: Fd,

    /// Open file descriptors
    fds: BTreeMap<Fd, FdEntry>,
}

/// File descriptor entry
pub struct FdEntry {
    /// Underlying kernel file handle
    handle: FileHandle,

    /// Rights for this FD
    rights: Rights,

    /// Flags (append, sync, etc.)
    flags: FdFlags,

    /// Current file position
    offset: u64,
}

impl FdTable {
    pub fn new() -> Self {
        let mut table = Self {
            next_fd: 3, // 0=stdin, 1=stdout, 2=stderr
            fds: BTreeMap::new(),
        };

        // Pre-open standard streams
        table.insert_stdio();
        table
    }

    fn insert_stdio(&mut self) {
        // stdin (fd=0)
        self.fds.insert(0, FdEntry {
            handle: FileHandle::stdin(),
            rights: Rights::FD_READ,
            flags: FdFlags(0),
            offset: 0,
        });

        // stdout (fd=1)
        self.fds.insert(1, FdEntry {
            handle: FileHandle::stdout(),
            rights: Rights::FD_WRITE,
            flags: FdFlags(0),
            offset: 0,
        });

        // stderr (fd=2)
        self.fds.insert(2, FdEntry {
            handle: FileHandle::stderr(),
            rights: Rights::FD_WRITE,
            flags: FdFlags(0),
            offset: 0,
        });
    }

    pub fn insert(&mut self, handle: FileHandle, rights: Rights, flags: FdFlags) -> Fd {
        let fd = self.next_fd;
        self.next_fd += 1;

        self.fds.insert(fd, FdEntry {
            handle,
            rights,
            flags,
            offset: 0,
        });

        fd
    }

    pub fn get(&self, fd: Fd) -> Option<&FdEntry> {
        self.fds.get(&fd)
    }

    pub fn get_mut(&mut self, fd: Fd) -> Option<&mut FdEntry> {
        self.fds.get_mut(&fd)
    }

    pub fn close(&mut self, fd: Fd) -> Result<(), Errno> {
        self.fds.remove(&fd).ok_or(Errno::BadF)?;
        Ok(())
    }
}
```

### M1.4: WASI Host Functions

```rust
// crates/kernel/src/wasm/wasi/mod.rs
use wasmi::{Caller, Linker};
use super::instance::InstanceContext;

/// Register all WASI functions with linker
pub fn register_wasi(linker: &mut Linker<InstanceContext>) -> Result<(), WasmError> {
    // File I/O
    linker.func_wrap("wasi_snapshot_preview1", "fd_read", wasi_fd_read)?;
    linker.func_wrap("wasi_snapshot_preview1", "fd_write", wasi_fd_write)?;
    linker.func_wrap("wasi_snapshot_preview1", "fd_close", wasi_fd_close)?;
    linker.func_wrap("wasi_snapshot_preview1", "path_open", wasi_path_open)?;

    // File metadata
    linker.func_wrap("wasi_snapshot_preview1", "fd_fdstat_get", wasi_fd_fdstat_get)?;
    linker.func_wrap("wasi_snapshot_preview1", "path_filestat_get", wasi_path_filestat_get)?;

    // Environment
    linker.func_wrap("wasi_snapshot_preview1", "environ_get", wasi_environ_get)?;
    linker.func_wrap("wasi_snapshot_preview1", "environ_sizes_get", wasi_environ_sizes_get)?;

    // Random
    linker.func_wrap("wasi_snapshot_preview1", "random_get", wasi_random_get)?;

    // Clock
    linker.func_wrap("wasi_snapshot_preview1", "clock_time_get", wasi_clock_time_get)?;

    // Process
    linker.func_wrap("wasi_snapshot_preview1", "proc_exit", wasi_proc_exit)?;

    Ok(())
}

/// fd_read: Read from file descriptor
fn wasi_fd_read(
    mut caller: Caller<'_, InstanceContext>,
    fd: u32,
    iovs_ptr: u32,
    iovs_len: u32,
    nread_ptr: u32,
) -> u32 {
    // Get memory
    let memory = match caller.get_export("memory").and_then(|e| e.into_memory()) {
        Some(m) => m,
        None => return Errno::Fault as u32,
    };

    // Check capability
    if !caller.data().capabilities.has(Capability::FileRead) {
        return Errno::NotCapable as u32;
    }

    // Get FD entry
    let fd_entry = match caller.data_mut().fds.get_mut(&fd) {
        Some(e) => e,
        None => return Errno::BadF as u32,
    };

    // Check FD has read rights
    if !fd_entry.rights.contains(Rights::FD_READ) {
        return Errno::NotCapable as u32;
    }

    // Read iovecs from WASM memory
    let mut total_read = 0u64;
    for i in 0..iovs_len {
        let iov_ptr = iovs_ptr + (i * 8); // Each iovec is 8 bytes (ptr + len)

        // Read iovec.buf_ptr and iovec.buf_len from memory
        let buf_ptr = read_u32(&memory, &caller, iov_ptr);
        let buf_len = read_u32(&memory, &caller, iov_ptr + 4);

        // Allocate temp buffer
        let mut buffer = alloc::vec![0u8; buf_len as usize];

        // Read from file
        let nread = match fd_entry.handle.read(&mut buffer) {
            Ok(n) => n,
            Err(_) => return Errno::Io as u32,
        };

        // Copy to WASM memory
        write_bytes(&memory, &mut caller, buf_ptr, &buffer[..nread]);

        total_read += nread as u64;
        fd_entry.offset += nread as u64;
    }

    // Write total bytes read
    write_u32(&memory, &mut caller, nread_ptr, total_read as u32);

    // Update stats
    caller.data_mut().stats.syscalls += 1;

    Errno::Success as u32
}

/// fd_write: Write to file descriptor
fn wasi_fd_write(
    mut caller: Caller<'_, InstanceContext>,
    fd: u32,
    iovs_ptr: u32,
    iovs_len: u32,
    nwritten_ptr: u32,
) -> u32 {
    // Get memory
    let memory = match caller.get_export("memory").and_then(|e| e.into_memory()) {
        Some(m) => m,
        None => return Errno::Fault as u32,
    };

    // Check capability
    if !caller.data().capabilities.has(Capability::FileWrite) {
        return Errno::NotCapable as u32;
    }

    // Get FD entry
    let fd_entry = match caller.data_mut().fds.get_mut(&fd) {
        Some(e) => e,
        None => return Errno::BadF as u32,
    };

    // Check FD has write rights
    if !fd_entry.rights.contains(Rights::FD_WRITE) {
        return Errno::NotCapable as u32;
    }

    // Write iovecs
    let mut total_written = 0u64;
    for i in 0..iovs_len {
        let iov_ptr = iovs_ptr + (i * 8);

        let buf_ptr = read_u32(&memory, &caller, iov_ptr);
        let buf_len = read_u32(&memory, &caller, iov_ptr + 4);

        // Read from WASM memory
        let buffer = read_bytes(&memory, &caller, buf_ptr, buf_len as usize);

        // Write to file
        let nwritten = match fd_entry.handle.write(&buffer) {
            Ok(n) => n,
            Err(_) => return Errno::Io as u32,
        };

        total_written += nwritten as u64;
        fd_entry.offset += nwritten as u64;
    }

    // Write total bytes written
    write_u32(&memory, &mut caller, nwritten_ptr, total_written as u32);

    caller.data_mut().stats.syscalls += 1;
    Errno::Success as u32
}

/// path_open: Open file or directory
fn wasi_path_open(
    mut caller: Caller<'_, InstanceContext>,
    dirfd: u32,
    dirflags: u32,
    path_ptr: u32,
    path_len: u32,
    oflags: u16,
    fs_rights_base: u64,
    fs_rights_inheriting: u64,
    fdflags: u16,
    fd_ptr: u32,
) -> u32 {
    // Check capability
    if !caller.data().capabilities.has(Capability::FileOpen) {
        return Errno::NotCapable as u32;
    }

    // Read path from memory
    let memory = match caller.get_export("memory").and_then(|e| e.into_memory()) {
        Some(m) => m,
        None => return Errno::Fault as u32,
    };

    let path_bytes = read_bytes(&memory, &caller, path_ptr, path_len as usize);
    let path_str = match core::str::from_utf8(&path_bytes) {
        Ok(s) => s,
        Err(_) => return Errno::Inval as u32,
    };

    // Convert oflags to VFS open flags
    let vfs_flags = oflags_to_vfs_flags(oflags);

    // Open file through VFS
    let handle = match crate::vfs::open(path_str, vfs_flags) {
        Ok(h) => h,
        Err(_) => return Errno::NoEnt as u32,
    };

    // Insert into FD table
    let fd = caller.data_mut().fds.insert(
        handle,
        Rights(fs_rights_base),
        FdFlags(fdflags),
    );

    // Write FD to memory
    write_u32(&memory, &mut caller, fd_ptr, fd);

    caller.data_mut().stats.syscalls += 1;
    Errno::Success as u32
}

/// random_get: Get cryptographically secure random bytes
fn wasi_random_get(
    mut caller: Caller<'_, InstanceContext>,
    buf_ptr: u32,
    buf_len: u32,
) -> u32 {
    let memory = match caller.get_export("memory").and_then(|e| e.into_memory()) {
        Some(m) => m,
        None => return Errno::Fault as u32,
    };

    // Generate random bytes
    let mut buffer = alloc::vec![0u8; buf_len as usize];
    crate::crypto::random_bytes(&mut buffer);

    // Write to WASM memory
    write_bytes(&memory, &mut caller, buf_ptr, &buffer);

    Errno::Success as u32
}

/// clock_time_get: Get current time
fn wasi_clock_time_get(
    mut caller: Caller<'_, InstanceContext>,
    clock_id: u32,
    precision: u64,
    time_ptr: u32,
) -> u32 {
    let memory = match caller.get_export("memory").and_then(|e| e.into_memory()) {
        Some(m) => m,
        None => return Errno::Fault as u32,
    };

    // Get time in nanoseconds
    let time_ns = match clock_id {
        0 => crate::time::current_time_ns(), // REALTIME
        1 => crate::time::monotonic_ns(),    // MONOTONIC
        _ => return Errno::Inval as u32,
    };

    // Write to memory
    write_u64(&memory, &mut caller, time_ptr, time_ns);

    Errno::Success as u32
}

// Helper functions for memory access
fn read_u32(memory: &wasmi::Memory, caller: &Caller<InstanceContext>, addr: u32) -> u32 {
    let mut bytes = [0u8; 4];
    memory.read(caller, addr as usize, &mut bytes).unwrap();
    u32::from_le_bytes(bytes)
}

fn write_u32(memory: &wasmi::Memory, caller: &mut Caller<InstanceContext>, addr: u32, value: u32) {
    let bytes = value.to_le_bytes();
    memory.write(caller, addr as usize, &bytes).unwrap();
}

fn read_bytes(memory: &wasmi::Memory, caller: &Caller<InstanceContext>, addr: u32, len: usize) -> Vec<u8> {
    let mut bytes = alloc::vec![0u8; len];
    memory.read(caller, addr as usize, &mut bytes).unwrap();
    bytes
}

fn write_bytes(memory: &wasmi::Memory, caller: &mut Caller<InstanceContext>, addr: u32, data: &[u8]) {
    memory.write(caller, addr as usize, data).unwrap();
}
```

### M1.5: WASI Tests

```rust
// Test WASI file I/O
#[test]
fn test_wasi_file_io() {
    // Compile simple WASM module that writes to file
    // (use wat2wasm for this test)
    let wasm = r#"
    (module
      (import "wasi_snapshot_preview1" "fd_write"
        (func $fd_write (param i32 i32 i32 i32) (result i32)))
      (memory (export "memory") 1)
      (data (i32.const 8) "Hello, WASI!\n")
      (func (export "_start")
        ;; iovec: [ptr=8, len=13]
        (i32.store (i32.const 0) (i32.const 8))
        (i32.store (i32.const 4) (i32.const 13))
        ;; fd_write(stdout=1, iovs=0, iovs_len=1, nwritten=16)
        (call $fd_write
          (i32.const 1)
          (i32.const 0)
          (i32.const 1)
          (i32.const 16))
        drop
      )
    )
    "#;

    let bytecode = wat::parse_str(wasm).unwrap();
    let module = WasmModule::from_bytecode(&bytecode, Some("test_wasi")).unwrap();

    let caps = CapabilitySet::new()
        .with(Capability::FileWrite);

    let mut instance = WasmInstance::new(&module, caps, 100000).unwrap();
    instance.call("_start", &[]).unwrap();

    // Verify output was written to stdout
    // (Would need to capture stdout for proper testing)
}
```

### M1 Deliverables

- [ ] WASI types defined
- [ ] FD table for file descriptor management
- [ ] `fd_read`, `fd_write`, `fd_close` implemented
- [ ] `path_open` for file opening
- [ ] `random_get` for random bytes
- [ ] `clock_time_get` for timestamps
- [ ] Integration tests with simple WASI programs
- [ ] Documentation for WASI usage

---

## Milestone 2: Capability System Integration

**Duration**: 1 week
**Goal**: Map WASI operations to kernel capability checks

### M2.1: Capability Mapping

```rust
// crates/kernel/src/wasm/capabilities.rs
use crate::capability::{Capability, CapabilitySet};

/// Map WASI rights to kernel capabilities
pub fn wasi_rights_to_capabilities(rights: Rights) -> CapabilitySet {
    let mut caps = CapabilitySet::empty();

    if rights.contains(Rights::FD_READ) {
        caps.grant(Capability::FileRead);
    }
    if rights.contains(Rights::FD_WRITE) {
        caps.grant(Capability::FileWrite);
    }
    if rights.contains(Rights::PATH_OPEN) {
        caps.grant(Capability::FileOpen);
    }
    if rights.contains(Rights::PATH_CREATE_FILE) {
        caps.grant(Capability::FileCreate);
    }
    if rights.contains(Rights::SOCK_CONNECT) {
        caps.grant(Capability::NetConnect);
    }
    if rights.contains(Rights::SOCK_SEND) {
        caps.grant(Capability::NetSend);
    }
    if rights.contains(Rights::SOCK_RECV) {
        caps.grant(Capability::NetRecv);
    }

    caps
}

/// Check if instance has required capability
pub fn check_capability(ctx: &InstanceContext, cap: Capability) -> Result<(), Errno> {
    if ctx.capabilities.has(cap) {
        Ok(())
    } else {
        Err(Errno::NotCapable)
    }
}
```

### M2.2: Capability Manifest

```rust
// WASM modules declare required capabilities in manifest
#[derive(Debug, Deserialize)]
pub struct WasmManifest {
    pub name: String,
    pub version: String,
    pub capabilities: Vec<String>,
    pub memory_limit: Option<usize>,
    pub fuel_limit: Option<u64>,
}

impl WasmManifest {
    pub fn parse(toml_str: &str) -> Result<Self, ManifestError> {
        toml::from_str(toml_str).map_err(|_| ManifestError::InvalidFormat)
    }

    pub fn to_capability_set(&self) -> Result<CapabilitySet, ManifestError> {
        let mut caps = CapabilitySet::empty();

        for cap_str in &self.capabilities {
            let cap = match cap_str.as_str() {
                "file:read" => Capability::FileRead,
                "file:write" => Capability::FileWrite,
                "file:create" => Capability::FileCreate,
                "net:connect" => Capability::NetConnect,
                "net:send" => Capability::NetSend,
                "net:recv" => Capability::NetRecv,
                "agent:spawn" => Capability::AgentSpawn,
                "llm:infer" => Capability::LlmInfer,
                _ => return Err(ManifestError::UnknownCapability(cap_str.clone())),
            };

            caps.grant(cap);
        }

        Ok(caps)
    }
}
```

### M2.3: Example Manifest

```toml
# agent_example.toml
name = "example-agent"
version = "0.1.0"

# Required capabilities
capabilities = [
    "file:read",
    "file:write",
    "agent:spawn",
]

# Resource limits
memory_limit = 16777216  # 16 MB
fuel_limit = 10000000    # 10M instructions
```

### M2 Deliverables

- [ ] WASI → Capability mapping
- [ ] Manifest format for declaring capabilities
- [ ] Runtime capability checking in WASI functions
- [ ] Tests for capability denial
- [ ] Documentation on capability system

---

## Milestone 3: AgentSys WASM Bindings

**Duration**: 2 weeks
**Goal**: Allow agents to be WASM modules

### M3.1: Agent Host Functions

```rust
// crates/kernel/src/wasm/agent_bindings.rs

/// Register AgentSys host functions
pub fn register_agent_functions(linker: &mut Linker<InstanceContext>) -> Result<(), WasmError> {
    linker.func_wrap("agent", "spawn", agent_spawn)?;
    linker.func_wrap("agent", "send_message", agent_send_message)?;
    linker.func_wrap("agent", "receive_message", agent_receive_message)?;
    linker.func_wrap("agent", "get_self_id", agent_get_self_id)?;
    linker.func_wrap("agent", "log", agent_log)?;

    Ok(())
}

/// Spawn new agent
fn agent_spawn(
    mut caller: Caller<'_, InstanceContext>,
    module_ptr: u32,
    module_len: u32,
    config_ptr: u32,
    config_len: u32,
) -> u32 {
    // Check capability
    if !caller.data().capabilities.has(Capability::AgentSpawn) {
        return 0; // Return 0 = failure
    }

    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    // Read module path
    let module_path = read_bytes(&memory, &caller, module_ptr, module_len as usize);
    let module_path_str = core::str::from_utf8(&module_path).unwrap();

    // Read config
    let config = read_bytes(&memory, &caller, config_ptr, config_len as usize);

    // Spawn agent through AgentSys
    match crate::agentsys::spawn_wasm_agent(module_path_str, &config) {
        Ok(agent_id) => agent_id,
        Err(_) => 0,
    }
}

/// Send message to another agent
fn agent_send_message(
    mut caller: Caller<'_, InstanceContext>,
    target_id: u32,
    msg_ptr: u32,
    msg_len: u32,
) -> u32 {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    // Read message
    let msg_bytes = read_bytes(&memory, &caller, msg_ptr, msg_len as usize);

    // Send through AgentSys
    match crate::agentsys::send_message(caller.data().id, target_id, msg_bytes) {
        Ok(()) => 1, // Success
        Err(_) => 0, // Failure
    }
}

/// Receive message (blocking)
fn agent_receive_message(
    mut caller: Caller<'_, InstanceContext>,
    buf_ptr: u32,
    buf_len: u32,
    sender_ptr: u32,
) -> u32 {
    // Receive message through AgentSys
    let (sender_id, msg) = match crate::agentsys::receive_message(caller.data().id) {
        Ok(m) => m,
        Err(_) => return 0, // No message
    };

    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    // Write sender ID
    write_u32(&memory, &mut caller, sender_ptr, sender_id);

    // Write message (truncate if too large)
    let copy_len = core::cmp::min(msg.len(), buf_len as usize);
    write_bytes(&memory, &mut caller, buf_ptr, &msg[..copy_len]);

    copy_len as u32
}
```

### M3.2: WASM Agent Example

```rust
// Example agent written in Rust, compiled to WASM

#[no_mangle]
pub extern "C" fn agent_init() -> u32 {
    // Called when agent starts
    agent_log(b"Agent initializing\n");
    1 // Success
}

#[no_mangle]
pub extern "C" fn agent_run() -> u32 {
    // Main agent loop
    loop {
        let mut buffer = [0u8; 1024];
        let mut sender = 0u32;

        // Receive message
        let len = agent_receive_message(buffer.as_mut_ptr(), buffer.len() as u32, &mut sender as *mut u32);

        if len > 0 {
            // Process message
            agent_log(b"Received message\n");

            // Send response
            let response = b"ACK";
            agent_send_message(sender, response.as_ptr(), response.len() as u32);
        }
    }
}

// Imported host functions
extern "C" {
    fn agent_send_message(target: u32, msg_ptr: *const u8, msg_len: u32) -> u32;
    fn agent_receive_message(buf_ptr: *mut u8, buf_len: u32, sender_ptr: *mut u32) -> u32;
    fn agent_log(msg_ptr: *const u8);
}
```

### M3 Deliverables

- [ ] Agent host functions for spawn/send/receive
- [ ] WASM agent lifecycle management
- [ ] Example agents in Rust → WASM
- [ ] Integration with existing AgentSys
- [ ] Tests for inter-agent communication

---

## Milestone 4: Resource Management

**Duration**: 1 week
**Goal**: Comprehensive resource tracking and limits

### M4.1: Resource Types

```rust
// crates/kernel/src/wasm/resources.rs

#[derive(Debug, Clone)]
pub struct ResourceLimits {
    /// Maximum linear memory (bytes)
    pub memory_limit: usize,

    /// Maximum fuel (instructions)
    pub fuel_limit: u64,

    /// Maximum execution time (nanoseconds)
    pub time_limit: u64,

    /// Maximum number of open file descriptors
    pub fd_limit: u32,

    /// Maximum number of spawned child agents
    pub spawn_limit: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            memory_limit: 16 * 1024 * 1024, // 16 MB
            fuel_limit: 10_000_000,          // 10M instructions
            time_limit: 1_000_000_000,       // 1 second
            fd_limit: 64,
            spawn_limit: 10,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ResourceUsage {
    pub memory_used: usize,
    pub fuel_consumed: u64,
    pub time_elapsed: u64,
    pub fds_open: u32,
    pub agents_spawned: u32,
}

impl ResourceUsage {
    pub fn check_limits(&self, limits: &ResourceLimits) -> Result<(), ResourceError> {
        if self.memory_used > limits.memory_limit {
            return Err(ResourceError::MemoryExceeded);
        }
        if self.fuel_consumed > limits.fuel_limit {
            return Err(ResourceError::FuelExceeded);
        }
        if self.time_elapsed > limits.time_limit {
            return Err(ResourceError::TimeExceeded);
        }
        if self.fds_open > limits.fd_limit {
            return Err(ResourceError::TooManyFds);
        }
        if self.agents_spawned > limits.spawn_limit {
            return Err(ResourceError::TooManyAgents);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ResourceError {
    MemoryExceeded,
    FuelExceeded,
    TimeExceeded,
    TooManyFds,
    TooManyAgents,
}
```

### M4.2: Fuel-Based Pacing

```rust
// Integrate with existing LLM pacing system
impl WasmInstance {
    /// Refuel instance using pacing budget
    pub fn refuel_with_pacing(&mut self, budget: &mut LlmBudget) -> Result<(), ResourceError> {
        // Calculate fuel needed
        let fuel_needed = self.fuel_limit - self.remaining_fuel();

        // Convert fuel to compute credits
        let credits_needed = fuel_needed / 1000; // 1 credit = 1000 fuel

        // Request from budget
        if budget.try_consume(credits_needed as usize) {
            // Grant fuel
            self.store.set_fuel(self.fuel_limit).unwrap();
            Ok(())
        } else {
            Err(ResourceError::FuelExceeded)
        }
    }
}
```

### M4.3: Memory Limits

```rust
// Set memory limits when creating instance
impl WasmInstance {
    pub fn new_with_limits(
        module: &WasmModule,
        capabilities: CapabilitySet,
        limits: ResourceLimits,
    ) -> Result<Self, WasmError> {
        let engine = crate::wasm::engine()?;

        // Configure engine with memory limits
        let mut config = wasmi::Config::default();
        config.set_max_memory_size(limits.memory_limit);

        let engine_limited = Engine::new(&config);

        // ... rest of instantiation
    }
}
```

### M4 Deliverables

- [ ] Resource limit configuration
- [ ] Fuel-based execution limiting
- [ ] Memory limit enforcement
- [ ] FD limit tracking
- [ ] Integration with LLM pacing
- [ ] Tests for resource exhaustion

---

## Milestone 5: LLM Integration

**Duration**: 1.5 weeks
**Goal**: LLM inference and LoRA as WASM modules

### M5.1: LLM Host Functions

```rust
// crates/kernel/src/wasm/llm_bindings.rs

pub fn register_llm_functions(linker: &mut Linker<InstanceContext>) -> Result<(), WasmError> {
    linker.func_wrap("llm", "infer", llm_infer)?;
    linker.func_wrap("llm", "load_lora", llm_load_lora)?;
    linker.func_wrap("llm", "unload_lora", llm_unload_lora)?;

    Ok(())
}

/// Run LLM inference
fn llm_infer(
    mut caller: Caller<'_, InstanceContext>,
    prompt_ptr: u32,
    prompt_len: u32,
    response_ptr: u32,
    response_max_len: u32,
) -> u32 {
    // Check capability
    if !caller.data().capabilities.has(Capability::LlmInfer) {
        return 0;
    }

    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    // Read prompt
    let prompt_bytes = read_bytes(&memory, &caller, prompt_ptr, prompt_len as usize);
    let prompt = core::str::from_utf8(&prompt_bytes).unwrap();

    // Run inference through LLM subsystem
    let response = match crate::llm::infer(prompt, response_max_len as usize) {
        Ok(r) => r,
        Err(_) => return 0,
    };

    // Write response to memory
    let response_bytes = response.as_bytes();
    let copy_len = core::cmp::min(response_bytes.len(), response_max_len as usize);
    write_bytes(&memory, &mut caller, response_ptr, &response_bytes[..copy_len]);

    copy_len as u32
}

/// Load LoRA adapter
fn llm_load_lora(
    mut caller: Caller<'_, InstanceContext>,
    path_ptr: u32,
    path_len: u32,
) -> u32 {
    if !caller.data().capabilities.has(Capability::LlmLoadAdapter) {
        return 0;
    }

    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    // Read adapter path
    let path_bytes = read_bytes(&memory, &caller, path_ptr, path_len as usize);
    let path = core::str::from_utf8(&path_bytes).unwrap();

    // Load adapter
    match crate::llm::load_lora_adapter(path) {
        Ok(adapter_id) => adapter_id,
        Err(_) => 0,
    }
}
```

### M5.2: LoRA as WASM Module

```rust
// Allow LoRA adapters to be implemented as WASM modules
// for custom transformation logic

pub struct WasmLoraAdapter {
    instance: WasmInstance,
}

impl WasmLoraAdapter {
    pub fn load(bytecode: &[u8]) -> Result<Self, WasmError> {
        let module = WasmModule::from_bytecode(bytecode, Some("lora_adapter"))?;

        let caps = CapabilitySet::new()
            .with(Capability::MemoryAccess);

        let instance = WasmInstance::new(&module, caps, 1_000_000)?;

        Ok(Self { instance })
    }

    /// Apply LoRA transformation to layer
    pub fn apply(&mut self, input: &[f32], output: &mut [f32]) -> Result<(), WasmError> {
        // Copy input to WASM memory
        // Call transform() function
        // Copy output from WASM memory

        todo!()
    }
}
```

### M5 Deliverables

- [ ] LLM host functions for inference
- [ ] LoRA loading/unloading
- [ ] WASM-based LoRA adapters (optional)
- [ ] Integration with existing LLM subsystem
- [ ] Performance benchmarks

---

## Milestone 6: Package System

**Duration**: 1 week
**Goal**: Package format for distributing WASM modules

### M6.1: Package Format

```
package_name.wap (WASM Application Package)
├── manifest.toml          # Metadata and capabilities
├── module.wasm            # Compiled WASM bytecode
├── signature.sig          # Ed25519 signature
└── assets/                # Optional assets
    ├── icon.png
    └── README.md
```

### M6.2: Package Structure

```rust
// crates/kernel/src/wasm/package.rs

pub struct WasmPackage {
    pub manifest: WasmManifest,
    pub bytecode: Vec<u8>,
    pub signature: Option<[u8; 64]>,
    pub assets: BTreeMap<String, Vec<u8>>,
}

impl WasmPackage {
    /// Load package from .wap archive
    pub fn load(archive_path: &str) -> Result<Self, PackageError> {
        // Read tar/zip archive
        let archive = crate::vfs::read(archive_path)?;

        // Extract manifest.toml
        let manifest_bytes = extract_file(&archive, "manifest.toml")?;
        let manifest = WasmManifest::parse(core::str::from_utf8(&manifest_bytes)?)?;

        // Extract module.wasm
        let bytecode = extract_file(&archive, "module.wasm")?;

        // Extract signature (if present)
        let signature = extract_file(&archive, "signature.sig").ok()
            .and_then(|s| <[u8; 64]>::try_from(s.as_slice()).ok());

        // Extract assets
        let assets = extract_assets(&archive)?;

        Ok(Self {
            manifest,
            bytecode,
            signature,
            assets,
        })
    }

    /// Verify package signature
    pub fn verify(&self, public_key: &[u8; 32]) -> Result<(), PackageError> {
        let sig = self.signature.ok_or(PackageError::NoSignature)?;

        // Hash manifest + bytecode
        let mut data = Vec::new();
        data.extend_from_slice(self.manifest.to_bytes());
        data.extend_from_slice(&self.bytecode);

        let hash = crate::crypto::sha256(&data);

        // Verify signature
        crate::crypto::ed25519_verify(&hash, &sig, public_key)
            .map_err(|_| PackageError::InvalidSignature)
    }

    /// Install package
    pub fn install(&self) -> Result<(), PackageError> {
        // Verify signature (if required)
        // Copy to /pkg/<name>/
        // Register with package manager

        todo!()
    }
}
```

### M6.3: Package Manager

```rust
// crates/kernel/src/wasm/package_manager.rs

pub struct PackageManager {
    installed: BTreeMap<String, InstalledPackage>,
}

pub struct InstalledPackage {
    pub manifest: WasmManifest,
    pub install_path: String,
    pub install_time: u64,
}

impl PackageManager {
    /// List installed packages
    pub fn list(&self) -> Vec<&InstalledPackage> {
        self.installed.values().collect()
    }

    /// Install package from file
    pub fn install(&mut self, package_path: &str) -> Result<(), PackageError> {
        let package = WasmPackage::load(package_path)?;

        // Check if already installed
        if self.installed.contains_key(&package.manifest.name) {
            return Err(PackageError::AlreadyInstalled);
        }

        // Copy to /pkg/<name>/
        let install_path = format!("/pkg/{}", package.manifest.name);
        crate::vfs::create_dir(&install_path)?;

        // Write module.wasm
        let module_path = format!("{}/module.wasm", install_path);
        crate::vfs::write(&module_path, &package.bytecode)?;

        // Write manifest.toml
        let manifest_path = format!("{}/manifest.toml", install_path);
        crate::vfs::write(&manifest_path, package.manifest.to_bytes())?;

        // Register
        self.installed.insert(package.manifest.name.clone(), InstalledPackage {
            manifest: package.manifest,
            install_path,
            install_time: crate::time::current_time_ns(),
        });

        Ok(())
    }

    /// Uninstall package
    pub fn uninstall(&mut self, name: &str) -> Result<(), PackageError> {
        let package = self.installed.remove(name)
            .ok_or(PackageError::NotInstalled)?;

        // Remove from filesystem
        crate::vfs::remove_dir_all(&package.install_path)?;

        Ok(())
    }

    /// Load installed module
    pub fn load(&self, name: &str) -> Result<WasmModule, PackageError> {
        let package = self.installed.get(name)
            .ok_or(PackageError::NotInstalled)?;

        let module_path = format!("{}/module.wasm", package.install_path);
        let bytecode = crate::vfs::read(&module_path)?;

        WasmModule::from_bytecode(&bytecode, Some(name))
            .map_err(|_| PackageError::InvalidModule)
    }
}
```

### M6.4: Shell Commands

```rust
impl Shell {
    /// List installed packages
    pub(crate) fn pkg_list_cmd(&self) {
        let pm = crate::wasm::package_manager();

        uart_print(b"Installed WASM packages:\n");
        for pkg in pm.list() {
            uart_print(b"  ");
            uart_print(pkg.manifest.name.as_bytes());
            uart_print(b" v");
            uart_print(pkg.manifest.version.as_bytes());
            uart_print(b"\n");
        }
    }

    /// Install package
    pub(crate) fn pkg_install_cmd(&self, args: &[u8]) {
        // Parse: pkg-install <path>
        let path = core::str::from_utf8(args).unwrap();

        match crate::wasm::package_manager().install(path) {
            Ok(()) => uart_print(b"Package installed successfully\n"),
            Err(e) => {
                uart_print(b"Error: Failed to install package\n");
            }
        }
    }
}
```

### M6 Deliverables

- [ ] Package format (.wap)
- [ ] Package manifest
- [ ] Signature verification
- [ ] Package manager for install/uninstall
- [ ] Shell commands: `pkg-list`, `pkg-install`, `pkg-uninstall`
- [ ] Example packages

---

## Milestone 7: Testing & Validation

**Duration**: 1 week
**Goal**: Comprehensive test suite

### M7.1: Unit Tests

```rust
#[cfg(test)]
mod tests {
    // Module loading
    #[test]
    fn test_load_valid_module() { }

    #[test]
    fn test_reject_invalid_module() { }

    // WASI
    #[test]
    fn test_wasi_fd_read() { }

    #[test]
    fn test_wasi_fd_write() { }

    #[test]
    fn test_wasi_path_open() { }

    // Capabilities
    #[test]
    fn test_capability_check() { }

    #[test]
    fn test_capability_denial() { }

    // Resources
    #[test]
    fn test_fuel_exhaustion() { }

    #[test]
    fn test_memory_limit() { }

    // AgentSys
    #[test]
    fn test_agent_spawn() { }

    #[test]
    fn test_agent_messaging() { }
}
```

### M7.2: Integration Tests

```rust
// Test full WASM agent lifecycle
#[test]
fn test_wasm_agent_lifecycle() {
    // 1. Create package
    let package = create_test_package();

    // 2. Install package
    let pm = package_manager();
    pm.install_package(&package).unwrap();

    // 3. Load and spawn agent
    let agent_id = agentsys::spawn_wasm_agent("test_agent", &[]).unwrap();

    // 4. Send message
    agentsys::send_message(0, agent_id, b"ping").unwrap();

    // 5. Receive response
    let (sender, msg) = agentsys::receive_message(0).unwrap();
    assert_eq!(msg, b"pong");

    // 6. Cleanup
    agentsys::stop_agent(agent_id).unwrap();
}
```

### M7.3: Fuzzing

```rust
// Fuzz test with arbitrary WASM bytecode
#[test]
fn fuzz_wasm_bytecode() {
    for _ in 0..1000 {
        let random_bytes = generate_random_bytes(1024);

        // Should not panic, just return error
        let _ = WasmModule::from_bytecode(&random_bytes, None);
    }
}
```

### M7 Deliverables

- [ ] 50+ unit tests
- [ ] 10+ integration tests
- [ ] Fuzzing tests for safety
- [ ] Performance benchmarks
- [ ] Test coverage report
- [ ] CI/CD integration

---

## Timeline

Total Duration: **6-8 weeks**

```
Week 1:   M0 - Runtime Integration
Week 2-3: M1 - WASI Implementation
Week 4:   M2 - Capability System
Week 5-6: M3 - AgentSys Bindings
Week 7:   M4 - Resource Management
          M5 - LLM Integration (start)
Week 8:   M5 - LLM Integration (finish)
          M6 - Package System
          M7 - Testing & Validation
```

### Parallel Tracks

- **Core Track** (M0-M2): Runtime, WASI, Capabilities
- **Integration Track** (M3-M5): AgentSys, Resources, LLM
- **Tooling Track** (M6-M7): Packages, Testing

---

## Migration from SISLang

Since we're replacing the custom SISLang plan, here's what to keep:

### Keep from SISLang Design

1. **Capability-based security** → Map to WASI capabilities
2. **Resource budgeting** → Use fuel metering
3. **Agent integration** → WASM host functions
4. **Package system** → .wap packages instead of .sib/.sip
5. **LLM integration** → WASM bindings to LLM subsystem

### Advantages Over SISLang

| Aspect | SISLang (Custom) | WASM |
|--------|-----------------|------|
| **Development Time** | 2-3 months | 6-8 weeks |
| **Tooling** | Build from scratch | Existing ecosystem |
| **Languages** | SISLang only | Rust, C/C++, Go, AS, ... |
| **Security** | Custom sandbox | Industry-proven sandbox |
| **Standards** | Proprietary | W3C standard |
| **Performance** | Interpreter | JIT possible |
| **Debugging** | Custom tools | Existing debuggers |
| **Libraries** | Start from zero | WASI ecosystem |

---

## References

### Specifications

- [WebAssembly Core Specification](https://webassembly.github.io/spec/core/)
- [WASI Preview1](https://github.com/WebAssembly/WASI/blob/main/legacy/preview1/docs.md)
- [wasmi Documentation](https://docs.rs/wasmi/)

### Example Projects

- [lunatic](https://github.com/lunatic-solutions/lunatic) - Actor-based WASM runtime
- [wasmtime](https://github.com/bytecodealliance/wasmtime) - Production WASM runtime
- [wasm3](https://github.com/wasm3/wasm3) - Fast WASM interpreter

### Toolchains

- **Rust**: `cargo build --target wasm32-wasi`
- **C/C++**: `clang --target=wasm32-wasi`
- **TinyGo**: `tinygo build -target=wasi`
- **AssemblyScript**: `asc file.ts -o file.wasm`

### Security

- [WASM Security Model](https://webassembly.org/docs/security/)
- [Capability-Based Security in WASM](https://bytecodealliance.org/articles/announcing-the-bytecode-alliance)

---

## Success Criteria

1. ✅ WASM modules can be loaded and executed
2. ✅ WASI provides file/network access
3. ✅ Capabilities properly restrict WASM modules
4. ✅ AgentSys agents can be WASM modules
5. ✅ Resource limits prevent abuse
6. ✅ LLM integration works from WASM
7. ✅ Package system for distribution
8. ✅ 90%+ test coverage
9. ✅ Performance: <10% overhead vs native
10. ✅ Documentation and examples

---

## Next Steps

1. Review this plan with team
2. Set up wasmi dependency
3. Start M0 implementation
4. Create example WASM modules for testing
5. Set up CI for WASM builds

---

*This plan replaces IMPLEMENTATION_PLAN_SISLANG.md as the primary scripting/extension mechanism for SIS Kernel.*
