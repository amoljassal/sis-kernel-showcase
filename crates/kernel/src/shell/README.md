# Shell Subsystem

## What Lives Here

The shell provides an interactive command-line interface for operating and debugging the SIS Kernel. All shell commands are organized into modular helper files that route to subsystem APIs.

**Main File:**
- `../shell.rs` - Shell core with dispatch loop (2000+ lines)

**Helper Modules (20+ files):**
- `agentctl_helpers.rs` - Agent network management commands
- `autoctl_helpers.rs` - Autonomous control commands
- `graphctl_helpers.rs` - Dataflow graph control commands
- `llmctl_helpers.rs` - LLM service control commands (feature-gated: `llm`)
- `neuralctl_helpers.rs` - Neural network control commands
- `schedctl_helpers.rs` - Scheduling AI control commands
- `shell_metricsctl.rs` - Metrics control commands
- `demos/mod.rs` - Demo commands (feature-gated: `demos`)
- ... (13 more helper modules)

**Related:**
- `../control.rs` - Control plane integration
- `../graph.rs` - Dataflow graph backend
- `../llm.rs` - LLM service backend
- `../deterministic.rs` - Scheduling backend

## How Shell Commands Are Routed

### Architecture Pattern: Thin Dispatch + Fat Helpers

The shell follows a **thin dispatch, fat helpers** pattern:

```
User Input → shell.rs dispatch → helper module → subsystem API → kernel module
```

**Example Flow:**
```
"graphctl status" → shell.rs → graphctl_helpers.rs → graph.rs::get_status() → GRAPH global
```

### Command Routing Pattern

```rust
// In shell.rs (thin dispatch)
fn handle_command(line: &str) {
    let parts: Vec<&str> = line.split_whitespace().collect();
    match parts.get(0) {
        Some(&"graphctl") => graphctl_helpers::handle_graphctl(&parts[1..]),
        Some(&"llmctl") => llmctl_helpers::handle_llmctl(&parts[1..]),
        Some(&"autoctl") => autoctl_helpers::handle_autoctl(&parts[1..]),
        // ... other commands
        _ => println!("Unknown command: {}", line),
    }
}

// In graphctl_helpers.rs (fat helper)
pub fn handle_graphctl(args: &[&str]) {
    match args.get(0) {
        Some(&"status") => {
            let stats = crate::graph::get_statistics();
            println!("Graph status: {} channels, {} operators", stats.channels, stats.operators);
        }
        Some(&"add-channel") => {
            // Parse args, validate, call graph::add_channel()
        }
        // ... other subcommands
    }
}
```

### Why This Pattern?

1. **Maintainability** - Each subsystem's commands in one file
2. **Feature gating** - Easy to conditionally compile groups of commands
3. **Testing** - Can test helper modules independently
4. **Minimal shell.rs** - Dispatch logic stays simple

## Integration Points and Boundaries

### **Incoming:** Who Calls the Shell
- **Main kernel loop** (`../main.rs`) - Shell runs after boot
- **UART input** (`../uart.rs`) - Character input from serial console
- **VirtIO console** (`../virtio_console.rs`) - Optional console input (feature: `virtio-console`)

### **Outgoing:** What Shell Commands Call
- **Module APIs** - Each helper calls specific kernel module functions
  - `graph.rs` - Dataflow graph operations
  - `llm.rs` - LLM service control
  - `deterministic.rs` - Scheduler control
  - `autonomy.rs` - Autonomous control
  - `neural.rs` - Neural network operations
  - `vfs::*` - File operations (cat, ls, mkdir)

### **Boundary Rules**
1. **Shell NEVER modifies kernel state directly** - Always through module APIs
2. **All output through printk** - No direct UART writes from helpers
3. **Parse and validate in helpers** - Shell.rs only dispatches
4. **Feature flags control command groups** - Use `#[cfg(feature = "...")]`
5. **No panics in shell code** - Always return errors to dispatch loop

## Adding a New Shell Command

### Step-by-Step Guide

1. **Create helper module** (if needed) or add to existing helper

```rust
// In shell/myctl_helpers.rs
pub fn handle_myctl(args: &[&str]) {
    match args.get(0) {
        Some(&"status") => {
            // Call kernel module API
            let status = crate::my_module::get_status();
            println!("MyModule status: {:?}", status);
        }
        Some(&"config") => {
            if args.len() < 2 {
                println!("Usage: myctl config <value>");
                return;
            }
            let value = args[1].parse::<u32>().unwrap_or(0);
            crate::my_module::set_config(value);
        }
        _ => {
            println!("myctl: unknown subcommand");
            println!("Available: status, config");
        }
    }
}
```

2. **Add dispatch in shell.rs**

```rust
// In shell.rs
use crate::shell::myctl_helpers;

fn handle_command(line: &str) {
    // ... existing matches
    Some(&"myctl") => myctl_helpers::handle_myctl(&parts[1..]),
    // ...
}
```

3. **Add help entry**

```rust
// In shell.rs help command
"myctl <subcommand>         - Manage MyModule subsystem",
```

4. **Feature-gate if needed**

```rust
// In shell.rs
#[cfg(feature = "my-module")]
Some(&"myctl") => myctl_helpers::handle_myctl(&parts[1..]),
```

### Example: Full Helper Module

```rust
// shell/myctl_helpers.rs
use crate::my_module;

pub fn handle_myctl(args: &[&str]) {
    if args.is_empty() {
        print_help();
        return;
    }

    match args[0] {
        "status" => cmd_status(),
        "enable" => cmd_enable(),
        "disable" => cmd_disable(),
        "config" => {
            if args.len() < 2 {
                println!("Usage: myctl config <key> <value>");
                return;
            }
            cmd_config(args[1], args.get(2).copied());
        }
        _ => {
            println!("Unknown subcommand: {}", args[0]);
            print_help();
        }
    }
}

fn cmd_status() {
    let status = my_module::get_status();
    println!("MyModule Status:");
    println!("  Enabled: {}", status.enabled);
    println!("  Counter: {}", status.counter);
}

fn cmd_enable() {
    my_module::set_enabled(true);
    println!("MyModule enabled");
}

fn cmd_disable() {
    my_module::set_enabled(false);
    println!("MyModule disabled");
}

fn cmd_config(key: &str, value: Option<&str>) {
    match value {
        Some(v) => {
            my_module::set_config(key, v);
            println!("Config '{}' set to '{}'", key, v);
        }
        None => {
            let current = my_module::get_config(key);
            println!("Config '{}' = '{}'", key, current);
        }
    }
}

fn print_help() {
    println!("myctl - Manage MyModule subsystem");
    println!();
    println!("Commands:");
    println!("  status           - Show current status");
    println!("  enable           - Enable MyModule");
    println!("  disable          - Disable MyModule");
    println!("  config <k> [v]   - Get/set configuration");
}
```

## Common Command Patterns

### Pattern 1: Simple Status Query
```rust
Some(&"status") => {
    let stats = subsystem::get_stats();
    println!("{:?}", stats);
}
```

### Pattern 2: Enable/Disable Toggle
```rust
Some(&"on") => subsystem::enable(),
Some(&"off") => subsystem::disable(),
Some(&"status") => {
    let enabled = subsystem::is_enabled();
    println!("Status: {}", if enabled { "ON" } else { "OFF" });
}
```

### Pattern 3: Numeric Parameter
```rust
Some(&"set") => {
    if args.len() < 2 {
        println!("Usage: cmd set <value>");
        return;
    }
    match args[1].parse::<u32>() {
        Ok(val) => subsystem::set_value(val),
        Err(_) => println!("Invalid number: {}", args[1]),
    }
}
```

### Pattern 4: Multi-Argument Command
```rust
Some(&"add") => {
    if args.len() < 3 {
        println!("Usage: cmd add <name> <value>");
        return;
    }
    let name = args[1];
    let value = args[2].parse().unwrap_or(0);
    subsystem::add_entry(name, value);
}
```

## Feature-Gated Commands

### LLM Commands (feature: `llm`)
```rust
#[cfg(feature = "llm")]
pub mod llmctl_helpers;

#[cfg(feature = "llm")]
Some(&"llmctl") => llmctl_helpers::handle_llmctl(&parts[1..]),
#[cfg(feature = "llm")]
Some(&"llminfer") => llmctl_helpers::handle_llminfer(&parts[1..]),
#[cfg(feature = "llm")]
Some(&"llmstream") => llmctl_helpers::handle_llmstream(&parts[1..]),
```

### Demo Commands (feature: `demos`)
```rust
#[cfg(feature = "demos")]
pub mod demos;

#[cfg(feature = "demos")]
Some(&"aidemo") => demos::run_ai_demo(),
#[cfg(feature = "demos")]
Some(&"graphdemo") => demos::run_graph_demo(),
```

## Shell Helper Modules Reference

| Helper Module | Feature Flag | Purpose |
|---------------|--------------|---------|
| `agentctl_helpers.rs` | - | Agent network management |
| `autoctl_helpers.rs` | - | Autonomous control (auto on/off/status/interval) |
| `graphctl_helpers.rs` | - | Dataflow graph (add-channel, add-operator, status) |
| `llmctl_helpers.rs` | `llm` | LLM service control (load, infer, stream) |
| `neuralctl_helpers.rs` | - | Neural network control |
| `schedctl_helpers.rs` | - | Scheduling AI control |
| `memctl_helpers.rs` | - | Memory AI control |
| `netctl_helpers.rs` | - | Network AI control |
| `cmdctl_helpers.rs` | - | Command prediction control |
| `shell_metricsctl.rs` | - | Metrics control (on/off/status) |
| `benchmark_helpers.rs` | - | Comparative benchmarking |
| `compliance_helpers.rs` | - | EU AI Act compliance |
| `fullautodemo_helpers.rs` | - | 7-phase autonomous demo |
| `stresstest_helpers.rs` | - | Stress testing commands |
| `pmu_helpers.rs` | - | PMU (Performance Monitoring Unit) commands |
| `demos/mod.rs` | `demos` | Demo commands (feature-gated) |

## Testing Shell Commands

### Manual Testing
```bash
# Boot kernel and test at shell prompt
sis> myctl status
sis> myctl enable
sis> myctl config key value
```

### Automated Testing (Phase 4)
```bash
# Use QMP for automated shell testing
./scripts/automated_shell_tests.sh

# Script sends commands via QMP and validates output
```

### Unit Testing Helpers
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_myctl_parse() {
        // Test argument parsing
        handle_myctl(&["status"]);
        handle_myctl(&["config", "key", "value"]);
    }
}
```

## Common Patterns and Gotchas

### ✅ Correct: Parse and Validate Arguments
```rust
if args.len() < 2 {
    println!("Usage: cmd <arg1> <arg2>");
    return;
}
let value = args[1].parse::<u32>().unwrap_or_else(|_| {
    println!("Invalid number: {}", args[1]);
    0
});
```

### ❌ Incorrect: Panic on Bad Input
```rust
let value = args[1].parse::<u32>().unwrap();  // DON'T PANIC
```

### ✅ Correct: Feature-Gate Entire Helper
```rust
#[cfg(feature = "llm")]
pub mod llmctl_helpers;
```

### ❌ Incorrect: Feature-Gate Individual Functions
```rust
#[cfg(feature = "llm")]
pub fn handle_llmctl(args: &[&str]) { }  // Better to gate entire module
```

## Performance Considerations

1. **Avoid heavy computation in helpers** - Offload to kernel modules
2. **Minimize string allocations** - Use `&str` slices, not `String`
3. **Batch output** - Collect results, then print (reduces serial overhead)
4. **Use feature gates** - Don't compile unused command groups

## Related Documentation

- **Main README Shell section** - High-level command reference
- `../README.md` - Module documentation (once created)
- `../control.rs` - Control plane architecture
- `docs/guides/DEV_HANDOFF.md` - Shell command conventions

## Future Work / TODOs

- [ ] Command history and editing (readline-style)
- [ ] Tab completion for commands
- [ ] Command aliases
- [ ] Scripting support (batch commands)
- [ ] Pipe operators (cmd1 | cmd2)
- [ ] Output redirection (cmd > file)
- [ ] Background jobs (cmd &)
- [ ] Job control (fg, bg, jobs)

## Contact / Maintainers

Shell is the primary user interface. Keep commands consistent and well-documented.

For questions about adding new commands or shell architecture, refer to this README.
