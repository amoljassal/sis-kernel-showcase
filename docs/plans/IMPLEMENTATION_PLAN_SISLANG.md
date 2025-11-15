# Implementation Plan: SIS Native Scripting Language

**Status:** Planning
**Target Platform:** SIS Kernel (QEMU + RPi 5)
**Language Name:** SISLang
**File Extensions:** `.sis` (source), `.sib` (bytecode), `.sip` (package)
**Timeline:** 8 weeks (56 days)

---

## Executive Summary

SISLang is a kernel-native, Python-like scripting language designed to run directly on the SIS kernel without requiring Linux ABI compatibility. It integrates deeply with kernel subsystems (AgentSys, VFS, metrics, OpenTelemetry) while providing secure, capability-based execution with deterministic resource management.

**Key Differentiators:**
- **Kernel-Native:** No POSIX layer required; direct kernel API access
- **Capability-Secure:** All operations gated through AgentSys tokens
- **Deterministic:** Resource budgets, pacing control, bounded execution
- **Observable:** Built-in metrics, OpenTelemetry spans, audit logging
- **Lightweight:** <500KB runtime, <10ms startup, mark-sweep GC

---

## 1. Architecture Overview

### 1.1 System Components

```
┌─────────────────────────────────────────────────────────────┐
│                     User Scripts (.sis)                      │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                    SISLang Compiler (siscc)                  │
│  Lexer → Parser → AST → Optimizer → Bytecode Emitter        │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                    Bytecode Files (.sib)                     │
│         Header | Constants | Code | Debug Info | Sig         │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                      SISLang Runtime VM                      │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │   VM Core    │  │   Scheduler  │  │      GC      │     │
│  │ ∙ Dispatch   │  │ ∙ Fibers     │  │ ∙ Mark-Sweep │     │
│  │ ∙ Stack      │  │ ∙ Budgets    │  │ ∙ Ref Count  │     │
│  │ ∙ Frames     │  │ ∙ Pacing     │  │ ∙ Gen GC     │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
├─────────────────────────────────────────────────────────────┤
│                      Standard Library                        │
│  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐   │
│  │  fs  │ │ net  │ │ time │ │ json │ │metrics│ │agent │   │
│  └──────┘ └──────┘ └──────┘ └──────┘ └──────┘ └──────┘   │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                    Kernel Integration Layer                  │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │   AgentSys   │  │     VFS      │  │   Metrics    │     │
│  │ Capabilities │  │  File I/O    │  │   & OTel     │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │   Network    │  │  Scheduler   │  │    Memory    │     │
│  │   (smoltcp)  │  │  (Process)   │  │  Management  │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 Data Flow

```
Source Code (.sis)
    ↓
[Compilation Phase]
    Tokenization → Parsing → AST Generation
    ↓
    Semantic Analysis → Type Inference
    ↓
    Optimization → Bytecode Generation
    ↓
Bytecode (.sib)
    ↓
[Runtime Phase]
    Loading → Verification → Linking
    ↓
    Execution (VM dispatch loop)
    ↓
    Host Calls → Capability Checks → Kernel APIs
    ↓
Results/Side Effects
```

---

## 2. Language Specification

### 2.1 Type System

```rust
// Value representation in VM
#[repr(C)]
pub enum Value {
    Nil,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(StringId),      // Interned string
    List(Arc<RefCell<Vec<Value>>>),
    Dict(Arc<RefCell<HashMap<StringId, Value>>>),
    Function(FunctionRef),
    NativeFunc(NativeFuncId),
    Object(ObjectRef),     // User-defined objects
    Fiber(FiberHandle),
    Error(ErrorValue),
}

// String interning for performance
pub struct StringPool {
    strings: HashMap<String, StringId>,
    pool: Vec<Arc<String>>,
    next_id: u32,
}

// Object representation
pub struct Object {
    class: ClassId,
    fields: HashMap<StringId, Value>,
    gc_mark: Cell<bool>,
}
```

### 2.2 Syntax Grammar (EBNF)

```ebnf
program     = statement* EOF ;

statement   = expr_stmt
            | if_stmt
            | while_stmt
            | for_stmt
            | func_def
            | class_def
            | return_stmt
            | break_stmt
            | continue_stmt
            | try_stmt
            | import_stmt ;

if_stmt     = "if" expression ":" block
              ("elif" expression ":" block)*
              ("else" ":" block)? ;

while_stmt  = "while" expression ":" block ;

for_stmt    = "for" IDENTIFIER "in" expression ":" block ;

func_def    = "def" IDENTIFIER "(" params? ")" ":" block ;

class_def   = "class" IDENTIFIER ("(" expression ")")? ":" class_body ;

try_stmt    = "try" ":" block
              ("except" (expression ("as" IDENTIFIER)?)? ":" block)+
              ("finally" ":" block)? ;

import_stmt = "import" module_path ("as" IDENTIFIER)?
            | "from" module_path "import" import_list ;

expression  = assignment ;
assignment  = logical_or ("=" assignment)? ;
logical_or  = logical_and ("or" logical_and)* ;
logical_and = equality ("and" equality)* ;
equality    = comparison (("==" | "!=") comparison)* ;
comparison  = addition (("<" | ">" | "<=" | ">=") addition)* ;
addition    = multiplication (("+" | "-") multiplication)* ;
multiplication = unary (("*" | "/" | "%" | "//") unary)* ;
unary       = ("not" | "-" | "+") unary | power ;
power       = call ("**" unary)? ;
call        = primary ("(" arguments? ")" | "[" expression "]" | "." IDENTIFIER)* ;
primary     = NUMBER | STRING | "True" | "False" | "None"
            | IDENTIFIER | "(" expression ")" | list_literal | dict_literal ;

list_literal = "[" (expression ("," expression)*)? "]" ;
dict_literal = "{" (expression ":" expression ("," expression ":" expression)*)? "}" ;
```

### 2.3 Built-in Functions

```python
# Core functions (always available)
print(*args, sep=' ', end='\n')
len(obj) -> int
range(start=0, stop, step=1) -> iterator
iter(obj) -> iterator
next(iterator, default=None) -> value
type(obj) -> str
isinstance(obj, type_or_types) -> bool
repr(obj) -> str
str(obj) -> str
int(obj) -> int
float(obj) -> float
bool(obj) -> bool
list(iterable) -> list
dict(iterable) -> dict
set(iterable) -> set
tuple(iterable) -> tuple
abs(x) -> number
min(*args) -> value
max(*args) -> value
sum(iterable, start=0) -> number
round(number, ndigits=0) -> number
sorted(iterable, key=None, reverse=False) -> list
reversed(sequence) -> iterator
enumerate(iterable, start=0) -> iterator
zip(*iterables) -> iterator
map(func, *iterables) -> iterator
filter(func, iterable) -> iterator
all(iterable) -> bool
any(iterable) -> bool
```

---

## 3. Virtual Machine Design

### 3.1 Bytecode Instruction Set

```rust
// Opcode definitions
#[repr(u8)]
pub enum OpCode {
    // Stack manipulation
    NOP         = 0x00,
    POP         = 0x01,
    DUP         = 0x02,
    DUP2        = 0x03,
    SWAP        = 0x04,
    ROT3        = 0x05,

    // Constants
    LOAD_CONST  = 0x10,  // u16 const_idx
    LOAD_NIL    = 0x11,
    LOAD_TRUE   = 0x12,
    LOAD_FALSE  = 0x13,

    // Variables
    LOAD_LOCAL  = 0x20,  // u8 local_idx
    STORE_LOCAL = 0x21,  // u8 local_idx
    LOAD_GLOBAL = 0x22,  // u16 name_idx
    STORE_GLOBAL= 0x23,  // u16 name_idx
    LOAD_UPVAL  = 0x24,  // u8 upval_idx
    STORE_UPVAL = 0x25,  // u8 upval_idx

    // Arithmetic
    ADD         = 0x30,
    SUB         = 0x31,
    MUL         = 0x32,
    DIV         = 0x33,
    MOD         = 0x34,
    POW         = 0x35,
    NEG         = 0x36,

    // Comparison
    EQ          = 0x40,
    NE          = 0x41,
    LT          = 0x42,
    LE          = 0x43,
    GT          = 0x44,
    GE          = 0x45,

    // Logical
    AND         = 0x50,
    OR          = 0x51,
    NOT         = 0x52,

    // Control flow
    JUMP        = 0x60,  // i16 offset
    JUMP_IF     = 0x61,  // i16 offset
    JUMP_IF_NOT = 0x62,  // i16 offset
    LOOP        = 0x63,  // i16 offset (backwards)

    // Functions
    CALL        = 0x70,  // u8 arg_count
    RETURN      = 0x71,
    CLOSURE     = 0x72,  // u16 func_idx

    // Collections
    BUILD_LIST  = 0x80,  // u16 item_count
    BUILD_DICT  = 0x81,  // u16 pair_count
    BUILD_SET   = 0x82,  // u16 item_count
    GET_ITEM    = 0x83,
    SET_ITEM    = 0x84,
    APPEND      = 0x85,

    // Objects
    GET_ATTR    = 0x90,  // u16 name_idx
    SET_ATTR    = 0x91,  // u16 name_idx
    BUILD_CLASS = 0x92,  // u16 name_idx

    // Iteration
    GET_ITER    = 0xA0,
    FOR_ITER    = 0xA1,  // i16 break_offset

    // Exceptions
    RAISE       = 0xB0,
    RERAISE     = 0xB1,
    TRY_BEGIN   = 0xB2,  // i16 handler_offset
    TRY_END     = 0xB3,
    EXCEPT      = 0xB4,  // u16 type_idx
    FINALLY     = 0xB5,

    // Imports
    IMPORT      = 0xC0,  // u16 module_idx
    IMPORT_FROM = 0xC1,  // u16 name_idx

    // Fibers
    YIELD       = 0xD0,
    YIELD_FROM  = 0xD1,
    AWAIT       = 0xD2,

    // Debug
    DEBUG_LINE  = 0xF0,  // u16 line_number
    DEBUG_ENTER = 0xF1,  // u16 scope_id
    DEBUG_LEAVE = 0xF2,

    // Termination
    HALT        = 0xFF,
}
```

### 3.2 Bytecode File Format (.sib)

```rust
// SIB (SIS Intermediate Bytecode) format
pub struct SibFile {
    header: SibHeader,
    constants: ConstantPool,
    functions: Vec<Function>,
    classes: Vec<Class>,
    debug_info: DebugInfo,
    signature: Option<Ed25519Signature>,
}

pub struct SibHeader {
    magic: [u8; 4],           // "SIB\0"
    version_major: u16,        // 1
    version_minor: u16,        // 0
    min_vm_version: u32,       // Minimum VM version required
    flags: u32,                // SIGNED, COMPRESSED, DEBUG_INFO
    checksum: [u8; 32],        // SHA256 of content
    timestamp: u64,            // Compilation timestamp
    module_name: String,       // Module identifier
}

pub struct ConstantPool {
    integers: Vec<i64>,
    floats: Vec<f64>,
    strings: Vec<String>,
    bytearrays: Vec<Vec<u8>>,
}

pub struct Function {
    name: StringId,
    arity: u8,
    max_locals: u8,
    max_stack: u16,
    code: Vec<u8>,
    constants: Vec<u16>,      // Indices into constant pool
    upvalues: Vec<UpvalueDesc>,
    exception_table: Vec<ExceptionHandler>,
}

pub struct ExceptionHandler {
    try_start: u16,
    try_end: u16,
    handler: u16,
    exception_type: Option<StringId>,
    var_name: Option<StringId>,
}
```

### 3.3 VM Execution Model

```rust
pub struct VM {
    // Execution state
    frames: Vec<CallFrame>,
    stack: Vec<Value>,
    globals: HashMap<StringId, Value>,

    // Memory management
    heap: Heap,
    gc: GarbageCollector,
    string_pool: StringPool,

    // Resource management
    budget: ResourceBudget,
    pacing: PacingControl,

    // Security
    agent_token: AgentSysToken,
    capabilities: CapabilitySet,
    audit_log: AuditLogger,

    // Scheduling
    fibers: FiberScheduler,
    current_fiber: Option<FiberHandle>,

    // Modules
    loaded_modules: HashMap<String, Module>,
    native_modules: HashMap<String, NativeModule>,

    // Metrics
    metrics: VmMetrics,
}

pub struct CallFrame {
    function: FunctionRef,
    ip: usize,                // Instruction pointer
    fp: usize,                // Frame pointer (stack base)
    locals: Vec<Value>,
    upvalues: Vec<UpvalueRef>,
}

pub struct ResourceBudget {
    max_memory: usize,         // Heap size limit
    max_stack_depth: usize,    // Call stack depth
    max_instructions: u64,     // Instruction count limit
    max_fibers: usize,         // Concurrent fiber limit
    time_budget_ns: u64,       // Wall clock time limit
    cpu_budget_cycles: u64,    // CPU cycles limit
}

pub struct PacingControl {
    enabled: bool,
    scale_percent: u32,        // 50-500%
    auto_adjust: bool,
    target_iops: u64,          // Instructions per second
    deadline_ns: u64,
    misses: u64,
}
```

### 3.4 VM Dispatch Loop

```rust
impl VM {
    pub fn execute(&mut self) -> Result<Value, VmError> {
        loop {
            // Check resource budgets
            if self.check_budgets()? == BudgetExceeded {
                self.yield_to_scheduler()?;
                continue;
            }

            // Get current instruction
            let frame = self.current_frame_mut();
            let opcode = frame.read_byte();

            // Update metrics
            self.metrics.instructions += 1;
            OPCODE_COUNTS[opcode as usize].fetch_add(1, Ordering::Relaxed);

            // Dispatch
            let start_cycles = read_cycle_counter();
            let result = match OpCode::from_u8(opcode) {
                OpCode::LOAD_CONST => {
                    let idx = frame.read_u16();
                    let value = self.get_constant(idx)?;
                    self.push(value);
                }

                OpCode::ADD => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(self.add_values(a, b)?);
                }

                OpCode::CALL => {
                    let arg_count = frame.read_u8();
                    self.call_function(arg_count)?;
                }

                OpCode::JUMP_IF_NOT => {
                    let offset = frame.read_i16();
                    let condition = self.pop()?;
                    if !self.is_truthy(condition) {
                        frame.ip = (frame.ip as i32 + offset as i32) as usize;
                    }
                }

                OpCode::BUILD_LIST => {
                    let count = frame.read_u16();
                    let items = self.pop_n(count as usize)?;
                    self.push(Value::List(Arc::new(RefCell::new(items))));
                }

                OpCode::GET_ITEM => {
                    let index = self.pop()?;
                    let container = self.pop()?;
                    self.push(self.get_item(container, index)?);
                }

                OpCode::YIELD => {
                    self.yield_fiber()?;
                }

                OpCode::HALT => {
                    break;
                }

                _ => return Err(VmError::InvalidOpcode(opcode)),
            };

            let elapsed_cycles = read_cycle_counter() - start_cycles;
            OPCODE_CYCLES[opcode as usize].fetch_add(elapsed_cycles, Ordering::Relaxed);

            // Handle errors
            if let Err(e) = result {
                if !self.handle_exception(e)? {
                    return Err(e);
                }
            }

            // Check for GC
            if self.should_gc() {
                self.collect_garbage()?;
            }
        }

        // Return top of stack or Nil
        Ok(self.pop().unwrap_or(Value::Nil))
    }

    fn check_budgets(&mut self) -> Result<BudgetStatus, VmError> {
        // Memory budget
        if self.heap.used() > self.budget.max_memory {
            return Err(VmError::OutOfMemory);
        }

        // Stack depth
        if self.frames.len() > self.budget.max_stack_depth {
            return Err(VmError::StackOverflow);
        }

        // Instruction budget
        if self.metrics.instructions > self.budget.max_instructions {
            return Ok(BudgetExceeded);
        }

        // Time budget with pacing
        let elapsed_ns = timestamp_ns() - self.metrics.start_time;
        if self.pacing.enabled {
            let target_ns = (self.metrics.instructions * 1_000_000_000) / self.pacing.target_iops;
            let scaled_target = (target_ns * self.pacing.scale_percent as u64) / 100;

            if elapsed_ns > scaled_target + self.pacing.deadline_ns {
                self.pacing.misses += 1;
                crate::info!("[SCRIPT][DEADLINE] miss actual={} expected={} misses={}",
                            elapsed_ns, scaled_target, self.pacing.misses);

                if self.pacing.auto_adjust {
                    self.pacing.scale_percent = (self.pacing.scale_percent * 110).min(500);
                }

                return Ok(BudgetExceeded);
            }
        }

        Ok(BudgetOk)
    }
}
```

---

## 4. Garbage Collection

### 4.1 Hybrid GC Strategy

```rust
pub struct GarbageCollector {
    strategy: GcStrategy,
    heap: Heap,
    roots: Vec<RootSet>,

    // Reference counting
    ref_counts: HashMap<ObjectId, u32>,

    // Mark-sweep
    mark_stack: Vec<ObjectId>,
    marked: BitSet,

    // Generational (optional)
    young_gen: Generation,
    old_gen: Generation,

    // Statistics
    stats: GcStats,
}

pub enum GcStrategy {
    RefCount,           // Simple reference counting
    MarkSweep,          // Stop-the-world mark-sweep
    Incremental,        // Incremental mark-sweep
    Generational,       // Young/old generations
}

impl GarbageCollector {
    pub fn collect(&mut self, vm: &VM) -> Result<GcStats, VmError> {
        let start = read_cycle_counter();

        match self.strategy {
            GcStrategy::RefCount => self.collect_refcount(vm),
            GcStrategy::MarkSweep => self.collect_marksweep(vm),
            GcStrategy::Incremental => self.collect_incremental(vm),
            GcStrategy::Generational => self.collect_generational(vm),
        }?;

        let elapsed = read_cycle_counter() - start;
        self.stats.total_cycles += elapsed;
        self.stats.collections += 1;

        // Emit metrics
        crate::metrics::METRIC.emit("vm_gc_cycles", elapsed);
        crate::metrics::METRIC.emit("vm_gc_freed_bytes", self.stats.last_freed);

        Ok(self.stats.clone())
    }

    fn collect_marksweep(&mut self, vm: &VM) -> Result<(), VmError> {
        // Phase 1: Mark
        self.marked.clear();
        self.mark_roots(vm)?;

        while let Some(obj_id) = self.mark_stack.pop() {
            if !self.marked.insert(obj_id) {
                continue;
            }

            let obj = self.heap.get(obj_id)?;
            self.mark_object(obj)?;
        }

        // Phase 2: Sweep
        let mut freed = 0;
        for obj_id in self.heap.all_objects() {
            if !self.marked.contains(obj_id) {
                freed += self.heap.free(obj_id)?;
            }
        }

        self.stats.last_freed = freed;
        Ok(())
    }

    fn mark_roots(&mut self, vm: &VM) -> Result<(), VmError> {
        // Mark from stack
        for value in &vm.stack {
            self.mark_value(value)?;
        }

        // Mark from call frames
        for frame in &vm.frames {
            for local in &frame.locals {
                self.mark_value(local)?;
            }
        }

        // Mark from globals
        for value in vm.globals.values() {
            self.mark_value(value)?;
        }

        // Mark from fibers
        for fiber in vm.fibers.all_fibers() {
            self.mark_fiber(fiber)?;
        }

        Ok(())
    }
}
```

### 4.2 Memory Management

```rust
pub struct Heap {
    chunks: Vec<MemoryChunk>,
    free_list: FreeList,
    total_size: usize,
    used_size: usize,
    high_water_mark: usize,
}

pub struct MemoryChunk {
    data: Box<[u8; CHUNK_SIZE]>,
    bitmap: BitSet,
    next_free: usize,
}

const CHUNK_SIZE: usize = 64 * 1024; // 64KB chunks

impl Heap {
    pub fn allocate(&mut self, size: usize) -> Result<*mut u8, VmError> {
        // Check heap limit
        if self.used_size + size > self.total_size {
            return Err(VmError::OutOfMemory);
        }

        // Try free list first
        if let Some(ptr) = self.free_list.allocate(size) {
            self.used_size += size;
            return Ok(ptr);
        }

        // Allocate from current chunk or get new chunk
        let ptr = self.allocate_from_chunk(size)?;
        self.used_size += size;

        // Update high water mark
        if self.used_size > self.high_water_mark {
            self.high_water_mark = self.used_size;
            crate::metrics::METRIC.emit("vm_heap_hwm", self.high_water_mark as u64);
        }

        Ok(ptr)
    }

    pub fn free(&mut self, ptr: *mut u8, size: usize) -> usize {
        self.free_list.free(ptr, size);
        self.used_size -= size;
        size
    }
}
```

---

## 5. Standard Library Modules

### 5.1 File System Module

```rust
// crates/langstd/src/fs.rs

pub struct FsModule;

impl NativeModule for FsModule {
    fn name(&self) -> &str { "fs" }

    fn functions(&self) -> Vec<NativeFunction> {
        vec![
            native_func!("read", fs_read, 1),
            native_func!("write", fs_write, 2),
            native_func!("exists", fs_exists, 1),
            native_func!("listdir", fs_listdir, 1),
            native_func!("stat", fs_stat, 1),
            native_func!("mkdir", fs_mkdir, 1..2),
            native_func!("remove", fs_remove, 1),
        ]
    }
}

fn fs_read(vm: &mut VM, args: &[Value]) -> Result<Value, VmError> {
    let path = args[0].as_string()?;

    // Capability check via AgentSys
    let token = vm.agent_token;
    if !crate::agent_sys::check_capability(token, Capability::FsRead, path)? {
        vm.audit_log.log(AuditEvent::FsRead(path, false));
        return Err(VmError::PermissionDenied);
    }

    vm.audit_log.log(AuditEvent::FsRead(path, true));

    // Perform read via VFS
    let data = crate::vfs::read(path)
        .map_err(|e| VmError::IoError(e.description()))?;

    Ok(Value::String(vm.intern_string(data)?))
}

fn fs_write(vm: &mut VM, args: &[Value]) -> Result<Value, VmError> {
    let path = args[0].as_string()?;
    let data = args[1].as_string()?;

    // Capability check
    let token = vm.agent_token;
    if !crate::agent_sys::check_capability(token, Capability::FsWrite, path)? {
        vm.audit_log.log(AuditEvent::FsWrite(path, data.len(), false));
        return Err(VmError::PermissionDenied);
    }

    vm.audit_log.log(AuditEvent::FsWrite(path, data.len(), true));

    // Size limit check
    if data.len() > MAX_WRITE_SIZE {
        return Err(VmError::ResourceLimit("write size exceeded"));
    }

    // Perform write via VFS
    crate::vfs::write(path, data.as_bytes())
        .map_err(|e| VmError::IoError(e.description()))?;

    Ok(Value::Nil)
}

fn fs_listdir(vm: &mut VM, args: &[Value]) -> Result<Value, VmError> {
    let path = args[0].as_string()?;

    // Capability check
    let token = vm.agent_token;
    if !crate::agent_sys::check_capability(token, Capability::FsList, path)? {
        vm.audit_log.log(AuditEvent::FsList(path, false));
        return Err(VmError::PermissionDenied);
    }

    vm.audit_log.log(AuditEvent::FsList(path, true));

    // List directory via VFS
    let entries = crate::vfs::listdir(path)
        .map_err(|e| VmError::IoError(e.description()))?;

    // Convert to Value::List
    let values: Result<Vec<Value>, _> = entries
        .iter()
        .map(|e| vm.intern_string(e.clone()).map(Value::String))
        .collect();

    Ok(Value::List(Arc::new(RefCell::new(values?))))
}
```

### 5.2 Network Module

```rust
// crates/langstd/src/net.rs

pub struct NetModule;

impl NativeModule for NetModule {
    fn name(&self) -> &str { "net" }

    fn functions(&self) -> Vec<NativeFunction> {
        vec![
            native_func!("tcp_connect", tcp_connect, 2),
            native_func!("tcp_send", tcp_send, 2),
            native_func!("tcp_recv", tcp_recv, 1..2),
            native_func!("tcp_close", tcp_close, 1),
            native_func!("http_get", http_get, 1..2),
            native_func!("http_post", http_post, 2..3),
            native_func!("dns_resolve", dns_resolve, 1),
        ]
    }
}

fn tcp_connect(vm: &mut VM, args: &[Value]) -> Result<Value, VmError> {
    let host = args[0].as_string()?;
    let port = args[1].as_int()? as u16;

    // Capability check
    let token = vm.agent_token;
    if !crate::agent_sys::check_capability(token, Capability::NetConnect, &host)? {
        vm.audit_log.log(AuditEvent::NetConnect(host, port, false));
        return Err(VmError::PermissionDenied);
    }

    vm.audit_log.log(AuditEvent::NetConnect(host.clone(), port, true));

    // Create TCP connection via smoltcp
    let socket = crate::net::tcp_connect(&host, port)
        .map_err(|e| VmError::NetworkError(e.to_string()))?;

    // Wrap in native object
    Ok(Value::NativeObject(NativeObject::TcpSocket(socket)))
}

fn http_get(vm: &mut VM, args: &[Value]) -> Result<Value, VmError> {
    let url = args[0].as_string()?;
    let timeout_ms = args.get(1)
        .and_then(|v| v.as_int().ok())
        .unwrap_or(30000) as u64;

    // Parse URL
    let parsed_url = parse_url(&url)?;

    // Capability check
    let token = vm.agent_token;
    if !crate::agent_sys::check_capability(token, Capability::NetHttp, &parsed_url.host)? {
        vm.audit_log.log(AuditEvent::HttpRequest("GET", url, false));
        return Err(VmError::PermissionDenied);
    }

    vm.audit_log.log(AuditEvent::HttpRequest("GET", url.clone(), true));

    // Perform HTTP GET
    let response = with_timeout(timeout_ms, async {
        crate::net::http_get(&url).await
    }).await?;

    // Convert response to dict
    let mut dict = HashMap::new();
    dict.insert(vm.intern_string("status")?, Value::Int(response.status as i64));
    dict.insert(vm.intern_string("body")?, Value::String(vm.intern_string(response.body)?));
    dict.insert(vm.intern_string("headers")?, headers_to_dict(vm, response.headers)?);

    Ok(Value::Dict(Arc::new(RefCell::new(dict))))
}
```

### 5.3 Time Module

```rust
// crates/langstd/src/time.rs

pub struct TimeModule;

impl NativeModule for TimeModule {
    fn name(&self) -> &str { "time" }

    fn functions(&self) -> Vec<NativeFunction> {
        vec![
            native_func!("now", time_now, 0),
            native_func!("now_ns", time_now_ns, 0),
            native_func!("sleep", time_sleep, 1),
            native_func!("uptime", time_uptime, 0),
        ]
    }
}

fn time_now(vm: &mut VM, _args: &[Value]) -> Result<Value, VmError> {
    let timestamp = crate::time::timestamp_seconds();
    Ok(Value::Float(timestamp as f64))
}

fn time_now_ns(vm: &mut VM, _args: &[Value]) -> Result<Value, VmError> {
    let timestamp = crate::time::timestamp_ns();
    Ok(Value::Int(timestamp as i64))
}

fn time_sleep(vm: &mut VM, args: &[Value]) -> Result<Value, VmError> {
    let seconds = args[0].as_float()?;
    let ns = (seconds * 1_000_000_000.0) as u64;

    // Yield fiber and schedule wakeup
    vm.fibers.sleep_current(ns)?;

    Ok(Value::Nil)
}
```

### 5.4 Metrics & OpenTelemetry Module

```rust
// crates/langstd/src/metrics.rs

pub struct MetricsModule;

impl NativeModule for MetricsModule {
    fn name(&self) -> &str { "metrics" }

    fn functions(&self) -> Vec<NativeFunction> {
        vec![
            native_func!("emit", metrics_emit, 2..3),
            native_func!("gauge", metrics_gauge, 2),
            native_func!("histogram", metrics_histogram, 2),
        ]
    }
}

fn metrics_emit(vm: &mut VM, args: &[Value]) -> Result<Value, VmError> {
    let key = args[0].as_string()?;
    let value = args[1].as_int()? as u64;
    let labels = args.get(2).and_then(|v| v.as_dict().ok());

    // Emit metric
    crate::metrics::METRIC.emit(&key, value);

    // Log if verbose
    crate::info!("[METRIC] {}={}", key, value);

    Ok(Value::Nil)
}

// crates/langstd/src/otel.rs

pub struct OtelModule;

impl NativeModule for OtelModule {
    fn name(&self) -> &str { "otel" }

    fn functions(&self) -> Vec<NativeFunction> {
        vec![
            native_func!("span", otel_span, 1),
            native_func!("event", otel_event, 1..2),
            native_func!("flush", otel_flush, 0),
        ]
    }
}

fn otel_span(vm: &mut VM, args: &[Value]) -> Result<Value, VmError> {
    let attrs = args[0].as_dict()?;

    // Create span
    let mut span = crate::otel::Span::new();

    // Set attributes from dict
    for (key, value) in attrs.borrow().iter() {
        let key_str = vm.get_string(*key)?;
        span.set_attribute(key_str, value_to_otel(value)?);
    }

    // Record span
    crate::otel::record_span(span);

    Ok(Value::Nil)
}
```

### 5.5 AgentSys Integration Module

```rust
// crates/langstd/src/agent.rs

pub struct AgentModule;

impl NativeModule for AgentModule {
    fn name(&self) -> &str { "agent" }

    fn functions(&self) -> Vec<NativeFunction> {
        vec![
            native_func!("fs_open", agent_fs_open, 2),
            native_func!("fs_read", agent_fs_read, 2),
            native_func!("fs_write", agent_fs_write, 3),
            native_func!("io_screenshot", agent_io_screenshot, 0..1),
            native_func!("io_record", agent_io_record, 1),
            native_func!("audio_play", agent_audio_play, 1),
            native_func!("llm_infer", agent_llm_infer, 1..2),
        ]
    }
}

fn agent_fs_read(vm: &mut VM, args: &[Value]) -> Result<Value, VmError> {
    let fd = args[0].as_int()? as u32;
    let size = args[1].as_int()? as usize;

    // Build AgentSys request
    let mut request = Vec::new();
    request.push(0x33); // FS_READ opcode
    request.extend_from_slice(&fd.to_le_bytes());
    request.extend_from_slice(&(size as u32).to_le_bytes());

    // Send via AgentSys
    let response = crate::agent_sys::handle_request(vm.agent_token, &request)?;

    // Parse response
    if response[0] != 0x00 {
        return Err(VmError::AgentError(response[0]));
    }

    let data = String::from_utf8_lossy(&response[1..]).to_string();
    Ok(Value::String(vm.intern_string(data)?))
}

fn agent_llm_infer(vm: &mut VM, args: &[Value]) -> Result<Value, VmError> {
    let prompt = args[0].as_string()?;
    let options = args.get(1).and_then(|v| v.as_dict().ok());

    // Check LLM capability
    let token = vm.agent_token;
    if !crate::agent_sys::check_capability(token, Capability::LlmInfer, "")? {
        return Err(VmError::PermissionDenied);
    }

    // Build inference request
    let mut config = crate::llm::LlmConfig::default();
    if let Some(opts) = options {
        // Parse options from dict
        if let Some(max_tokens) = opts.borrow().get(&vm.intern_string("max_tokens")?) {
            config.max_tokens = max_tokens.as_int()? as u32;
        }
    }

    // Perform inference
    let result = crate::llm::infer(&prompt, &config)?;

    Ok(Value::String(vm.intern_string(result)?))
}
```

---

## 6. Concurrency Model

### 6.1 Fiber Implementation

```rust
pub struct Fiber {
    id: FiberId,
    state: FiberState,
    stack: Vec<Value>,
    frames: Vec<CallFrame>,
    wake_time: Option<u64>,
    waiting_on: Option<WaitTarget>,
    priority: u8,
}

pub enum FiberState {
    Ready,
    Running,
    Sleeping,
    Waiting,
    Completed,
    Failed(VmError),
}

pub enum WaitTarget {
    Io(IoHandle),
    Fiber(FiberId),
    Channel(ChannelId),
    Timer(u64),
}

pub struct FiberScheduler {
    fibers: HashMap<FiberId, Fiber>,
    ready_queue: VecDeque<FiberId>,
    sleep_queue: BinaryHeap<(u64, FiberId)>,
    current: Option<FiberId>,
    next_id: u32,
}

impl FiberScheduler {
    pub fn spawn(&mut self, function: Value, args: Vec<Value>) -> Result<FiberId, VmError> {
        let id = FiberId(self.next_id);
        self.next_id += 1;

        let fiber = Fiber {
            id,
            state: FiberState::Ready,
            stack: args,
            frames: vec![/* initial frame */],
            wake_time: None,
            waiting_on: None,
            priority: 0,
        };

        self.fibers.insert(id, fiber);
        self.ready_queue.push_back(id);

        Ok(id)
    }

    pub fn schedule_next(&mut self) -> Option<FiberId> {
        // Check sleeping fibers
        let now = crate::time::timestamp_ns();
        while let Some((wake_time, fiber_id)) = self.sleep_queue.peek() {
            if *wake_time <= now {
                let (_, id) = self.sleep_queue.pop().unwrap();
                if let Some(fiber) = self.fibers.get_mut(&id) {
                    fiber.state = FiberState::Ready;
                    self.ready_queue.push_back(id);
                }
            } else {
                break;
            }
        }

        // Get next ready fiber
        self.ready_queue.pop_front()
    }

    pub fn yield_current(&mut self) -> Result<(), VmError> {
        if let Some(current) = self.current {
            if let Some(fiber) = self.fibers.get_mut(&current) {
                if matches!(fiber.state, FiberState::Running) {
                    fiber.state = FiberState::Ready;
                    self.ready_queue.push_back(current);
                }
            }
        }
        Ok(())
    }

    pub fn sleep_current(&mut self, ns: u64) -> Result<(), VmError> {
        if let Some(current) = self.current {
            if let Some(fiber) = self.fibers.get_mut(&current) {
                fiber.state = FiberState::Sleeping;
                fiber.wake_time = Some(crate::time::timestamp_ns() + ns);
                self.sleep_queue.push((fiber.wake_time.unwrap(), current));
            }
        }
        Ok(())
    }
}
```

### 6.2 Cooperative Scheduling Points

```rust
// Automatic yield points inserted by compiler
// 1. Before function calls
// 2. At loop backedges
// 3. Before I/O operations
// 4. After N instructions (configurable)

impl VM {
    const YIELD_CHECK_INTERVAL: u64 = 1000; // Check every 1000 instructions

    fn should_yield(&self) -> bool {
        // Check instruction count
        if self.metrics.instructions % Self::YIELD_CHECK_INTERVAL == 0 {
            return true;
        }

        // Check time slice
        let elapsed = crate::time::timestamp_ns() - self.fiber_start_time;
        if elapsed > FIBER_TIME_SLICE_NS {
            return true;
        }

        // Check if higher priority fiber is ready
        if self.fibers.has_higher_priority_ready(self.current_fiber) {
            return true;
        }

        false
    }
}
```

---

## 7. Security & Sandboxing

### 7.1 Capability Model

```rust
pub struct SecurityContext {
    token: AgentSysToken,
    capabilities: CapabilitySet,
    resource_limits: ResourceLimits,
    audit_enabled: bool,
}

pub struct CapabilitySet {
    fs_read: PathMatcher,
    fs_write: PathMatcher,
    net_connect: HostMatcher,
    llm_access: bool,
    metrics_emit: bool,
    max_memory: usize,
    max_cpu_ms: u64,
}

pub struct PathMatcher {
    allowed_prefixes: Vec<String>,
    denied_prefixes: Vec<String>,
    max_size: usize,
}

impl PathMatcher {
    pub fn check(&self, path: &str, size: usize) -> bool {
        // Check denied prefixes first
        for prefix in &self.denied_prefixes {
            if path.starts_with(prefix) {
                return false;
            }
        }

        // Check allowed prefixes
        let allowed = self.allowed_prefixes.is_empty() ||
                     self.allowed_prefixes.iter().any(|p| path.starts_with(p));

        // Check size limit
        allowed && size <= self.max_size
    }
}
```

### 7.2 Audit Logging

```rust
pub struct AuditLogger {
    enabled: bool,
    buffer: Vec<AuditEvent>,
    file: Option<File>,
}

#[derive(Debug, Serialize)]
pub enum AuditEvent {
    ScriptStart { script: String, token: u32 },
    ScriptEnd { script: String, result: String },
    FsRead(String, bool),
    FsWrite(String, usize, bool),
    FsList(String, bool),
    NetConnect(String, u16, bool),
    HttpRequest(&'static str, String, bool),
    LlmInfer(String, bool),
    ResourceLimit { type: String, limit: u64, used: u64 },
}

impl AuditLogger {
    pub fn log(&mut self, event: AuditEvent) {
        if !self.enabled {
            return;
        }

        let timestamp = crate::time::timestamp_ns();
        let serialized = serde_json::to_string(&event).unwrap();

        crate::info!("[AUDIT] {} {}", timestamp, serialized);

        // Buffer for later retrieval
        self.buffer.push(event);

        // Write to file if configured
        if let Some(file) = &mut self.file {
            writeln!(file, "{} {}", timestamp, serialized);
        }
    }
}
```

---

## 8. Package System

### 8.1 Package Format (.sip)

```rust
// SIP (SIS Package) format - ZIP archive containing:
// - manifest.json
// - *.sib files (bytecode modules)
// - signature.json (Ed25519 signature)
// - assets/ (optional data files)

#[derive(Serialize, Deserialize)]
pub struct PackageManifest {
    name: String,
    version: String,           // Semver
    description: String,
    author: String,
    license: String,
    min_vm_version: String,
    entrypoint: String,         // Main module
    modules: Vec<String>,       // List of .sib files
    dependencies: Vec<Dependency>,
    capabilities: RequestedCapabilities,
}

#[derive(Serialize, Deserialize)]
pub struct RequestedCapabilities {
    fs_read: Vec<String>,       // Path prefixes
    fs_write: Vec<String>,
    net_hosts: Vec<String>,     // Allowed hosts
    llm: bool,
    metrics: bool,
    max_memory_mb: u32,
    max_cpu_seconds: u32,
}

#[derive(Serialize, Deserialize)]
pub struct PackageSignature {
    algorithm: String,          // "ed25519"
    public_key: String,         // Base64 encoded
    signature: String,          // Base64 encoded
    timestamp: u64,
    files: Vec<FileHash>,       // Hash of each file
}
```

### 8.2 Package Manager (sispm)

```rust
// tools/sispm/src/main.rs

pub struct PackageManager {
    registry: PackageRegistry,
    installed: HashMap<String, InstalledPackage>,
    key_store: KeyStore,
}

impl PackageManager {
    pub fn build(&self, src_dir: &Path, output: &Path) -> Result<(), PmError> {
        // 1. Load manifest
        let manifest = load_manifest(src_dir)?;

        // 2. Compile all .sis files to .sib
        for module in find_modules(src_dir)? {
            compile_module(&module)?;
        }

        // 3. Create ZIP archive
        let mut zip = ZipWriter::new(File::create(output)?);

        // Add manifest
        zip.start_file("manifest.json", FileOptions::default())?;
        zip.write_all(&serde_json::to_vec(&manifest)?)?;

        // Add bytecode modules
        for sib_file in find_sib_files(src_dir)? {
            let name = sib_file.file_name().unwrap();
            zip.start_file(name, FileOptions::default())?;
            zip.write_all(&fs::read(&sib_file)?)?;
        }

        zip.finish()?;
        Ok(())
    }

    pub fn sign(&self, package: &Path, key_path: &Path) -> Result<(), PmError> {
        // Load private key
        let private_key = load_private_key(key_path)?;

        // Create signature
        let mut hasher = Sha256::new();
        let files = list_package_files(package)?;

        for file in &files {
            hasher.update(&file.hash);
        }

        let signature = private_key.sign(&hasher.finalize());

        // Add signature to package
        add_signature_to_package(package, signature)?;

        Ok(())
    }

    pub fn verify(&self, package: &Path) -> Result<bool, PmError> {
        // Extract signature
        let sig = extract_signature(package)?;

        // Verify with public key
        let public_key = PublicKey::from_bytes(&base64::decode(&sig.public_key)?)?;

        // Hash all files
        let mut hasher = Sha256::new();
        for file_hash in &sig.files {
            hasher.update(&file_hash.hash);
        }

        // Verify signature
        Ok(public_key.verify(&hasher.finalize(), &sig.signature))
    }

    pub fn install(&self, package: &Path, target_dir: &Path) -> Result<(), PmError> {
        // Verify signature if crypto-real enabled
        #[cfg(feature = "crypto-real")]
        {
            if !self.verify(package)? {
                return Err(PmError::InvalidSignature);
            }
        }

        // Extract package
        extract_package(package, target_dir)?;

        // Register installation
        let manifest = load_manifest_from_package(package)?;
        self.register_installation(&manifest, target_dir)?;

        Ok(())
    }
}
```

---

## 9. Shell Integration

### 9.1 Shell Commands

```rust
// crates/apps/siscript/src/shell.rs

pub fn register_script_commands(shell: &mut Shell) {
    shell.register_command("script", script_command);
    shell.register_command("scriptctl", scriptctl_command);
}

fn script_command(shell: &mut Shell, args: &[&str]) -> Result<(), ShellError> {
    match args.get(0).map(|s| *s) {
        Some("run") => {
            let path = args.get(1).ok_or("Usage: script run <path>")?;
            let token = parse_token(args)?;
            let timeout = parse_timeout(args)?;
            let mem_limit = parse_mem_limit(args)?;

            run_script(path, token, timeout, mem_limit)?;
        }

        Some("repl") => {
            start_repl()?;
        }

        Some("compile") => {
            let src = args.get(1).ok_or("Usage: script compile <src> <out>")?;
            let out = args.get(2).ok_or("Usage: script compile <src> <out>")?;
            compile_script(src, out)?;
        }

        _ => {
            shell.print("Usage: script <run|repl|compile> ...");
        }
    }
    Ok(())
}

fn scriptctl_command(shell: &mut Shell, args: &[&str]) -> Result<(), ShellError> {
    match args.get(0).map(|s| *s) {
        Some("list") => {
            let scripts = list_running_scripts()?;
            for (id, info) in scripts {
                shell.print(&format!("{}: {} (state={:?})", id, info.name, info.state));
            }
        }

        Some("kill") => {
            let id = args.get(1).ok_or("Usage: scriptctl kill <id>")?;
            kill_script(id.parse()?)?;
        }

        Some("pace") => {
            if let Some(subcmd) = args.get(1) {
                match *subcmd {
                    "--scale" => {
                        let scale = args.get(2)
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(100);
                        set_script_pace_scale(scale)?;
                    }
                    "auto" => {
                        let enabled = args.get(2).map(|s| *s == "on").unwrap_or(false);
                        set_script_pace_auto(enabled)?;
                    }
                    _ => {}
                }
            }

            let status = get_pace_status()?;
            shell.print(&format!("Script pacing: scale={}% auto={} misses={}",
                               status.scale, status.auto, status.misses));
        }

        Some("audit") => {
            let count = args.get(1)
                .and_then(|s| s.parse().ok())
                .unwrap_or(10);

            let events = get_audit_events(count)?;
            for event in events {
                shell.print(&format!("[AUDIT] {}", event));
            }
        }

        _ => {
            shell.print("Usage: scriptctl <list|kill|pace|audit> ...");
        }
    }
    Ok(())
}
```

### 9.2 REPL Implementation

```rust
pub struct Repl {
    vm: VM,
    history: Vec<String>,
    line_buffer: String,
}

impl Repl {
    pub fn run(&mut self) -> Result<(), ReplError> {
        crate::uart_print(b"SISLang REPL v1.0\n");
        crate::uart_print(b"Type 'exit()' to quit\n\n");

        loop {
            crate::uart_print(b">>> ");

            let line = self.read_line()?;
            if line.trim() == "exit()" {
                break;
            }

            self.history.push(line.clone());

            // Try to compile and execute
            match self.execute_line(&line) {
                Ok(value) => {
                    if !matches!(value, Value::Nil) {
                        crate::uart_print(format!("{}\n", self.format_value(&value)).as_bytes());
                    }
                }
                Err(e) => {
                    crate::uart_print(format!("Error: {}\n", e).as_bytes());
                }
            }
        }

        Ok(())
    }

    fn execute_line(&mut self, line: &str) -> Result<Value, VmError> {
        // Parse line
        let ast = parse_expression(line)?;

        // Compile to bytecode
        let bytecode = compile_expression(&ast)?;

        // Execute in VM
        self.vm.execute_bytecode(&bytecode)
    }
}
```

---

## 10. Compiler Implementation

### 10.1 Lexer

```rust
// crates/langcompiler/src/lexer.rs

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    If, Elif, Else, While, For, In, Def, Class, Return, Break, Continue,
    Try, Except, Finally, Raise, Import, From, As, Pass, Lambda,
    True, False, None, And, Or, Not,

    // Identifiers and literals
    Identifier(String),
    Integer(i64),
    Float(f64),
    String(String),

    // Operators
    Plus, Minus, Star, Slash, Percent, DoubleSlash, DoubleStar,
    Equal, NotEqual, Less, Greater, LessEqual, GreaterEqual,
    Assign, PlusAssign, MinusAssign, StarAssign, SlashAssign,

    // Delimiters
    LeftParen, RightParen, LeftBracket, RightBracket, LeftBrace, RightBrace,
    Comma, Colon, Semicolon, Dot, Arrow,

    // Special
    Newline, Indent, Dedent, Eof,
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    current_char: Option<char>,
    indent_stack: Vec<usize>,
}

impl Lexer {
    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();

        while let Some(token) = self.next_token()? {
            tokens.push(token);
        }

        // Add remaining dedents
        while self.indent_stack.len() > 1 {
            self.indent_stack.pop();
            tokens.push(Token::Dedent);
        }

        tokens.push(Token::Eof);
        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Option<Token>, LexError> {
        self.skip_whitespace();

        match self.current_char {
            None => Ok(None),

            Some('#') => {
                self.skip_comment();
                self.next_token()
            }

            Some('\n') => {
                self.advance();
                self.handle_indentation()
            }

            Some('"') | Some('\'') => self.read_string(),

            Some(c) if c.is_ascii_digit() => self.read_number(),

            Some(c) if c.is_ascii_alphabetic() || c == '_' => self.read_identifier(),

            Some('+') => self.make_token_advance(Token::Plus),
            Some('-') => self.make_token_advance(Token::Minus),
            Some('*') => {
                self.advance();
                if self.current_char == Some('*') {
                    self.advance();
                    Ok(Some(Token::DoubleStar))
                } else {
                    Ok(Some(Token::Star))
                }
            }
            // ... more operators ...

            _ => Err(LexError::UnexpectedCharacter(self.current_char.unwrap())),
        }
    }
}
```

### 10.2 Parser

```rust
// crates/langcompiler/src/parser.rs

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn parse(&mut self) -> Result<Module, ParseError> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            statements.push(self.statement()?);
        }

        Ok(Module { statements })
    }

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        match &self.peek() {
            Token::If => self.if_statement(),
            Token::While => self.while_statement(),
            Token::For => self.for_statement(),
            Token::Def => self.function_def(),
            Token::Class => self.class_def(),
            Token::Return => self.return_statement(),
            Token::Try => self.try_statement(),
            Token::Import => self.import_statement(),
            _ => self.expression_statement(),
        }
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, ParseError> {
        let expr = self.logical_or()?;

        if self.match_token(&Token::Assign) {
            let value = self.assignment()?;
            return Ok(Expr::Assign(Box::new(expr), Box::new(value)));
        }

        Ok(expr)
    }

    // Precedence climbing for binary operators
    fn logical_or(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.logical_and()?;

        while self.match_token(&Token::Or) {
            let op = BinaryOp::Or;
            let right = self.logical_and()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    // ... more parsing methods ...
}
```

### 10.3 Bytecode Compiler

```rust
// crates/langcompiler/src/compiler.rs

pub struct Compiler {
    constants: ConstantPool,
    functions: Vec<CompiledFunction>,
    current_function: usize,
    scope_depth: usize,
    locals: Vec<Local>,
}

impl Compiler {
    pub fn compile(&mut self, ast: &Module) -> Result<SibFile, CompileError> {
        // Compile main function
        let main = self.compile_function("__main__", &ast.statements)?;
        self.functions.push(main);

        // Create SIB file
        Ok(SibFile {
            header: self.create_header(),
            constants: self.constants.clone(),
            functions: self.functions.clone(),
            classes: vec![],
            debug_info: DebugInfo::default(),
            signature: None,
        })
    }

    fn compile_statement(&mut self, stmt: &Stmt) -> Result<(), CompileError> {
        match stmt {
            Stmt::Expression(expr) => {
                self.compile_expression(expr)?;
                self.emit(OpCode::POP);
            }

            Stmt::If { condition, then_branch, else_branch } => {
                self.compile_expression(condition)?;

                let jump_to_else = self.emit_jump(OpCode::JUMP_IF_NOT);

                self.compile_statement(then_branch)?;

                let jump_to_end = self.emit_jump(OpCode::JUMP);
                self.patch_jump(jump_to_else);

                if let Some(else_stmt) = else_branch {
                    self.compile_statement(else_stmt)?;
                }

                self.patch_jump(jump_to_end);
            }

            Stmt::While { condition, body } => {
                let loop_start = self.current_offset();

                self.compile_expression(condition)?;
                let jump_to_end = self.emit_jump(OpCode::JUMP_IF_NOT);

                self.compile_statement(body)?;
                self.emit_loop(loop_start);

                self.patch_jump(jump_to_end);
            }

            Stmt::Return(expr) => {
                if let Some(e) = expr {
                    self.compile_expression(e)?;
                } else {
                    self.emit(OpCode::LOAD_NIL);
                }
                self.emit(OpCode::RETURN);
            }

            // ... more statement types ...
        }

        Ok(())
    }

    fn compile_expression(&mut self, expr: &Expr) -> Result<(), CompileError> {
        match expr {
            Expr::Integer(n) => {
                let idx = self.constants.add_integer(*n);
                self.emit_constant(idx);
            }

            Expr::String(s) => {
                let idx = self.constants.add_string(s.clone());
                self.emit_constant(idx);
            }

            Expr::Identifier(name) => {
                if let Some(local) = self.resolve_local(name) {
                    self.emit_byte(OpCode::LOAD_LOCAL);
                    self.emit_byte(local.slot);
                } else {
                    let idx = self.constants.add_string(name.clone());
                    self.emit_byte(OpCode::LOAD_GLOBAL);
                    self.emit_u16(idx);
                }
            }

            Expr::Binary(left, op, right) => {
                self.compile_expression(left)?;
                self.compile_expression(right)?;

                match op {
                    BinaryOp::Add => self.emit(OpCode::ADD),
                    BinaryOp::Sub => self.emit(OpCode::SUB),
                    BinaryOp::Mul => self.emit(OpCode::MUL),
                    BinaryOp::Div => self.emit(OpCode::DIV),
                    BinaryOp::Eq => self.emit(OpCode::EQ),
                    BinaryOp::Lt => self.emit(OpCode::LT),
                    // ... more operators ...
                }
            }

            Expr::Call(func, args) => {
                self.compile_expression(func)?;

                for arg in args {
                    self.compile_expression(arg)?;
                }

                self.emit_byte(OpCode::CALL);
                self.emit_byte(args.len() as u8);
            }

            // ... more expression types ...
        }

        Ok(())
    }
}
```

---

## 11. Testing Strategy

### 11.1 Unit Tests

```rust
// tests/unit/value_test.rs
#[test]
fn test_value_equality() {
    assert_eq!(Value::Int(42), Value::Int(42));
    assert_ne!(Value::Int(42), Value::Float(42.0));
    assert_eq!(Value::String(StringId(1)), Value::String(StringId(1)));
}

// tests/unit/gc_test.rs
#[test]
fn test_mark_sweep() {
    let mut gc = GarbageCollector::new(GcStrategy::MarkSweep);
    let mut heap = Heap::new(1024 * 1024);

    // Allocate objects
    let obj1 = heap.allocate_object(Object::new());
    let obj2 = heap.allocate_object(Object::new());

    // Only obj1 is reachable
    gc.add_root(obj1);

    // Collect
    let stats = gc.collect(&heap).unwrap();

    assert_eq!(stats.freed_count, 1);
    assert!(heap.is_alive(obj1));
    assert!(!heap.is_alive(obj2));
}
```

### 11.2 Integration Tests

```rust
// tests/integration/fs_test.sis
import fs

# Test file operations
fs.write("/tmp/test.txt", "Hello, World!")
content = fs.read("/tmp/test.txt")
assert content == "Hello, World!"

# Test directory operations
fs.mkdir("/tmp/testdir")
assert fs.exists("/tmp/testdir")

files = fs.listdir("/tmp")
assert "test.txt" in files
assert "testdir" in files

# Cleanup
fs.remove("/tmp/test.txt")
fs.remove("/tmp/testdir")
```

### 11.3 Conformance Tests

```rust
// tests/conformance/control_flow.sis

# Test if/elif/else
x = 10
if x < 5:
    result = "less"
elif x < 15:
    result = "middle"
else:
    result = "more"
assert result == "middle"

# Test while loop
count = 0
while count < 5:
    count = count + 1
assert count == 5

# Test for loop
sum = 0
for i in range(10):
    sum = sum + i
assert sum == 45

# Test break/continue
result = []
for i in range(10):
    if i == 3:
        continue
    if i == 7:
        break
    result.append(i)
assert result == [0, 1, 2, 4, 5, 6]

# Test exceptions
try:
    x = 1 / 0
    assert False  # Should not reach here
except ZeroDivisionError as e:
    caught = True
assert caught == True

# Test finally
cleanup = False
try:
    raise ValueError("test")
except ValueError:
    pass
finally:
    cleanup = True
assert cleanup == True
```

### 11.4 Performance Benchmarks

```rust
// tests/perf/benchmark.rs

#[bench]
fn bench_arithmetic(b: &mut Bencher) {
    let script = r#"
        sum = 0
        for i in range(1000000):
            sum = sum + i
    "#;

    let bytecode = compile(script).unwrap();
    let mut vm = VM::new();

    b.iter(|| {
        vm.execute_bytecode(&bytecode).unwrap();
    });
}

#[bench]
fn bench_function_calls(b: &mut Bencher) {
    let script = r#"
        def fib(n):
            if n < 2:
                return n
            return fib(n-1) + fib(n-2)

        result = fib(20)
    "#;

    let bytecode = compile(script).unwrap();
    let mut vm = VM::new();

    b.iter(|| {
        vm.execute_bytecode(&bytecode).unwrap();
    });
}
```

### 11.5 Fuzz Testing

```rust
// tests/fuzz/fuzz_targets/parser.rs

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = parse_module(s);  // Should not panic
    }
});

// tests/fuzz/fuzz_targets/vm.rs

fuzz_target!(|data: &[u8]| {
    let mut vm = VM::new();
    vm.set_budget(ResourceBudget {
        max_memory: 1024 * 1024,
        max_instructions: 10000,
        time_budget_ns: 1_000_000_000,
        ..Default::default()
    });

    // Should not panic or hang
    let _ = vm.execute_bytecode(data);
});
```

---

## 12. Implementation Timeline

### Week 1-2: Core VM & Parser (M0)

**Deliverables:**
- Basic lexer and parser
- AST representation
- Tree-walking interpreter (no bytecode yet)
- Value types and basic operations
- REPL functional

**Files to create:**
```
crates/langvm/src/
  ├── value.rs        (300 LOC)
  ├── interpreter.rs  (500 LOC)
  └── repl.rs        (200 LOC)

crates/langcompiler/src/
  ├── lexer.rs       (400 LOC)
  ├── parser.rs      (600 LOC)
  └── ast.rs         (200 LOC)
```

**Acceptance tests:**
```python
# Can execute in REPL
>>> 1 + 2
3
>>> x = 10
>>> x * 2
20
>>> def add(a, b):
...     return a + b
>>> add(3, 4)
7
```

### Week 3: Bytecode & Stdlib (M1)

**Deliverables:**
- Bytecode compiler
- VM dispatch loop
- Stack-based execution
- File system module
- Time module
- JSON module

**Files to create:**
```
crates/langvm/src/
  ├── vm.rs          (800 LOC)
  ├── opcodes.rs     (200 LOC)
  └── dispatch.rs    (600 LOC)

crates/langcompiler/src/
  └── compiler.rs    (800 LOC)

crates/langstd/src/
  ├── fs.rs          (400 LOC)
  ├── time.rs        (200 LOC)
  └── json.rs        (300 LOC)
```

**Acceptance tests:**
```python
# File operations work
import fs
fs.write("/tmp/test.txt", "Hello")
assert fs.read("/tmp/test.txt") == "Hello"

# JSON works
import json
data = {"key": "value", "number": 42}
encoded = json.encode(data)
decoded = json.decode(encoded)
assert decoded["number"] == 42
```

### Week 4: Security & AgentSys (M2)

**Deliverables:**
- Capability checking
- Resource budgets
- AgentSys integration
- Audit logging

**Files to create:**
```
crates/langvm/src/
  ├── security.rs    (400 LOC)
  └── budget.rs      (300 LOC)

crates/langstd/src/
  └── agent.rs       (500 LOC)
```

**Acceptance tests:**
```python
# Capability denied
import fs
# Without FS_WRITE capability:
try:
    fs.write("/etc/passwd", "bad")
except PermissionError:
    pass  # Expected

# Audit logged
import agent
agent.fs_read(fd, 100)
# Check audit log shows: [AUDIT] agent=X op=FS_READ result=ALLOW
```

### Week 5: GC & Memory (M3)

**Deliverables:**
- Mark-sweep GC
- Reference counting option
- Heap management
- Memory limits

**Files to create:**
```
crates/langvm/src/
  ├── gc.rs          (600 LOC)
  ├── heap.rs        (400 LOC)
  └── memory.rs      (300 LOC)
```

**Acceptance tests:**
```python
# GC collects unreachable objects
x = [1, 2, 3]
x = None  # Original list should be collected

# Memory limit enforced
huge_list = []
try:
    for i in range(1000000000):
        huge_list.append(i)
except MemoryError:
    pass  # Expected when limit reached
```

### Week 6: Networking (M4)

**Deliverables:**
- TCP client
- HTTP client
- DNS resolution
- Timeouts

**Files to create:**
```
crates/langstd/src/
  └── net.rs         (600 LOC)
```

**Acceptance tests:**
```python
import net

# HTTP GET works
response = net.http_get("http://10.0.2.2:8080/test")
assert response["status"] == 200

# Timeout works
try:
    net.http_get("http://10.0.2.2:9999", timeout_ms=100)
except TimeoutError:
    pass  # Expected
```

### Week 7: Concurrency & Pacing (M5)

**Deliverables:**
- Fiber implementation
- Cooperative scheduling
- CPU pacing
- Deadline monitoring

**Files to create:**
```
crates/langvm/src/
  ├── fiber.rs       (500 LOC)
  ├── scheduler.rs   (400 LOC)
  └── pacing.rs      (300 LOC)
```

**Acceptance tests:**
```python
import task

# Spawn fiber
def worker():
    for i in range(10):
        print(i)
        task.yield()

handle = task.spawn(worker)
task.join(handle)

# Pacing works
# Check logs show: [SCRIPT][DEADLINE] ok actual=X expected=Y
```

### Week 8: Packaging & Polish (M6)

**Deliverables:**
- Package format (.sip)
- Signature verification
- siscc compiler tool
- sispm package manager
- Documentation

**Files to create:**
```
tools/siscc/src/
  └── main.rs        (400 LOC)

tools/sispm/src/
  └── main.rs        (600 LOC)

docs/lang/
  ├── spec.md
  ├── stdlib.md
  └── tutorial.md
```

**Acceptance tests:**
```bash
# Compile script
siscc hello.sis -o hello.sib

# Create package
sispm build myapp/ -o myapp.sip

# Sign package
sispm sign myapp.sip --key private.pem

# Install package
sispm install myapp.sip

# Run installed script
script run myapp
```

---

## 13. Performance Targets

### Execution Speed
- **Arithmetic ops:** 1M ops/sec minimum
- **Function calls:** 100K calls/sec minimum
- **String operations:** 50K ops/sec minimum

### Memory Usage
- **VM overhead:** <500KB base
- **Per-fiber overhead:** <4KB
- **String interning:** <2x string size

### Latency
- **GC pause:** <1ms for 10MB heap
- **Fiber switch:** <100μs
- **Script startup:** <10ms

### Resource Limits
- **Max heap:** 100MB default (configurable)
- **Max stack depth:** 1000 frames
- **Max fibers:** 1000 concurrent
- **Max instructions:** 1B per execution

---

## 14. Success Criteria

### Functional Requirements
- [ ] Python-like syntax parses correctly
- [ ] All control flow constructs work
- [ ] Functions and closures work
- [ ] Exceptions propagate correctly
- [ ] Modules import successfully
- [ ] All stdlib modules functional

### Security Requirements
- [ ] 100% of operations capability-checked
- [ ] No unauthorized file/network access
- [ ] Resource limits enforced
- [ ] Audit logging complete

### Performance Requirements
- [ ] Meets all performance targets
- [ ] No memory leaks detected
- [ ] GC collects all garbage
- [ ] Pacing maintains deadlines

### Integration Requirements
- [ ] Runs in QEMU without issues
- [ ] AgentSys integration works
- [ ] Metrics/OTel data flows
- [ ] Shell commands functional

---

## 15. Risk Analysis

### Technical Risks

**Risk 1: VM Performance**
- **Impact:** Scripts run too slowly
- **Mitigation:** Start with simple interpreter, optimize later
- **Fallback:** Consider JIT or WASM backend

**Risk 2: GC Complexity**
- **Impact:** Memory leaks or pauses
- **Mitigation:** Start with reference counting
- **Fallback:** Use arena allocator with script lifetime

**Risk 3: Security Vulnerabilities**
- **Impact:** Sandbox escape
- **Mitigation:** All operations through AgentSys
- **Fallback:** Additional sandboxing layer

### Schedule Risks

**Risk 1: Parser Complexity**
- **Impact:** 1-2 week delay
- **Mitigation:** Simplify syntax initially
- **Fallback:** Use parser generator

**Risk 2: Integration Issues**
- **Impact:** 1 week delay
- **Mitigation:** Test continuously in QEMU
- **Fallback:** Reduce stdlib scope

---

## 16. Documentation Requirements

### User Documentation
- Language tutorial (10 pages)
- Standard library reference (20 pages)
- Shell commands guide (5 pages)
- Example scripts (10+ examples)

### Developer Documentation
- VM internals guide (15 pages)
- Bytecode specification (10 pages)
- Module development guide (10 pages)
- Security model explanation (5 pages)

### API Documentation
- Rust API docs (rustdoc)
- Host binding API guide
- Package format specification

---

## Appendix A: Example Scripts

### A.1 System Monitor
```python
import fs
import time
import metrics

def monitor_system():
    while True:
        # Read memory usage
        meminfo = fs.read("/proc/meminfo")
        lines = meminfo.split("\n")
        for line in lines:
            if line.startswith("MemAvailable:"):
                parts = line.split()
                available_kb = int(parts[1])
                metrics.emit("memory_available_kb", available_kb)
                break

        # Sleep for 10 seconds
        time.sleep(10)

monitor_system()
```

### A.2 HTTP Server
```python
import net
import json

def handle_request(conn):
    # Read request
    request = net.tcp_recv(conn, 4096)

    # Parse request line
    lines = request.split("\r\n")
    request_line = lines[0].split(" ")
    method = request_line[0]
    path = request_line[1]

    # Build response
    if path == "/health":
        body = json.encode({"status": "ok"})
        response = "HTTP/1.1 200 OK\r\n"
        response += "Content-Type: application/json\r\n"
        response += f"Content-Length: {len(body)}\r\n"
        response += "\r\n"
        response += body
    else:
        response = "HTTP/1.1 404 Not Found\r\n\r\n"

    # Send response
    net.tcp_send(conn, response)
    net.tcp_close(conn)

# Listen on port 8080
server = net.tcp_listen(8080)
while True:
    conn = net.tcp_accept(server)
    handle_request(conn)
```

---

## Appendix B: Opcode Reference

| Opcode | Hex | Args | Stack | Description |
|--------|-----|------|-------|-------------|
| NOP | 0x00 | - | - | No operation |
| POP | 0x01 | - | -1 | Pop top of stack |
| DUP | 0x02 | - | +1 | Duplicate top of stack |
| LOAD_CONST | 0x10 | u16 | +1 | Load constant |
| LOAD_LOCAL | 0x20 | u8 | +1 | Load local variable |
| STORE_LOCAL | 0x21 | u8 | -1 | Store local variable |
| ADD | 0x30 | - | -1 | Add top two values |
| SUB | 0x31 | - | -1 | Subtract |
| MUL | 0x32 | - | -1 | Multiply |
| DIV | 0x33 | - | -1 | Divide |
| EQ | 0x40 | - | -1 | Equality test |
| LT | 0x42 | - | -1 | Less than |
| JUMP | 0x60 | i16 | - | Unconditional jump |
| JUMP_IF_NOT | 0x62 | i16 | -1 | Jump if false |
| CALL | 0x70 | u8 | -(n-1) | Call function |
| RETURN | 0x71 | - | - | Return from function |
| BUILD_LIST | 0x80 | u16 | -(n-1) | Create list |
| GET_ITEM | 0x83 | - | -1 | Index operation |
| YIELD | 0xD0 | - | - | Yield to scheduler |
| HALT | 0xFF | - | - | Stop execution |

---

**End of Implementation Plan**