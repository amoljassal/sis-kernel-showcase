# Shell Commands Reorganization Plan

**Date Created**: 2025-11-16
**Status**: Planning
**Priority**: High
**Estimated Time**: 4-6 hours

---

## Objectives

1. **Create comprehensive shell commands table in README**
   - Single unified table with all 191+ shell commands
   - Categorically organized for easy discovery
   - Proper formatting and alignment

2. **Enhance help command functionality**
   - Categorical organization with alphabetical category sorting
   - Commands within categories arranged by usage sequence
   - Detailed explanations with robotics analogies
   - Support `help <command>` for specific command lookup

---

## Part 1: README Shell Commands Table

### Requirements

**Table Structure:**
```markdown
| Sr. No. | Category | Command | Subcommands |
|---------|----------|---------|-------------|
| 1 | Agent Supervision (ASM) | agentsys | status, list, spawn, kill, restart, metrics, resources, limits, telemetry, compliance, risk, deps, depgraph, policy, policy-update, profile, profile-reset, dump, info, gwstatus |
| 2 | Agent Supervision (ASM) | asmstatus | - |
...
```

**Categories (organized by feature, not alphabetical):**
1. **Core System** - Basic OS commands (help, clear, uptime, version, reboot, shutdown, etc.)
2. **Filesystem** - File and directory operations (ls, cat, mkdir, rm, cp, mv, etc.)
3. **Process Management** - Process control (ps, kill, spawn, etc.)
4. **Network** - Network operations (ifconfig, ping, dhcp, curl, etc.)
5. **Memory Management** - Memory operations (memstat, heapinfo, pageinfo, etc.)
6. **Agent Supervision (ASM)** - Agent lifecycle and monitoring (agentsys + subcommands)
7. **AI Phase 1** - Neural, memory, autonomous agents (neuralctl, memctl, autoctl, etc.)
8. **AI Phase 2** - Multi-agent coordination (coordctl, deployctl, driftctl, etc.)
9. **AI Phase 7** - AI Operations (modelctl, tracectl, shadowctl)
10. **Chaos Engineering** - Chaos testing (chaos + subcommands)
11. **Observability** - Metrics and monitoring (metricsctl, pmustats, etc.)
12. **Security** - Security operations (users, groups, chmod, etc.)
13. **Graphics & UI** - Window management and display (windows, screenshot, etc.)
14. **Audio & Media** - Audio operations (audio, voice, etc.)
15. **Demos** - Demo commands (coorddemo, metademo, actorcriticdemo, etc.)
16. **Development** - Testing and debugging (test, stress, benchmark, etc.)

### Implementation Steps

1. **Inventory all commands** (estimated: 30 min)
   - Read `crates/kernel/src/shell.rs` for command dispatch table
   - List all top-level commands
   - Document all subcommands for each command
   - Count total: should match 73 top-level, 118 subcommands, 191 total

2. **Categorize commands** (estimated: 30 min)
   - Assign each command to appropriate category
   - Order commands within category by usage sequence
   - Verify no commands are missing

3. **Create table in README** (estimated: 30 min)
   - Location: After "Shell Command Reference" section
   - Format: Markdown table with 4 columns
   - Add table of contents link
   - Ensure proper alignment

4. **Verify table completeness** (estimated: 15 min)
   - Cross-reference with actual shell.rs implementation
   - Check all agentsys subcommands are listed
   - Check all chaos subcommands are listed
   - Verify modelctl, tracectl, shadowctl subcommands

---

## Part 2: Help Command Enhancement

### Current State

The help command currently prints a flat list of commands:
- No categorization
- Random order
- Minimal descriptions
- No subcommand documentation
- No `help <command>` support

### Target State

**Example Output:**

```
sis> help

SIS Kernel Shell - Command Reference
Commands are arranged categorically and in sequential testing order.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
1. AGENT SUPERVISION (ASM)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  agentsys status
    What it does: Displays the status of all 8 ASM subsystems
    Use case: System health check and initialization verification
    Functionality: Shows Agent Supervisor, Telemetry Aggregator, Fault
                   Detector, Policy Controller, Compliance Tracker,
                   Resource Monitor, Dependency Graph, System Profiler
    Robotics: Like checking if all robot control systems are online
    OS-level: Queries ASM supervisor lock, reads subsystem states

  agentsys list
    What it does: Lists all currently active agents in the system
    Use case: See which agents are running and their basic info
    Functionality: Displays agent ID, name, status, and uptime
    Robotics: Like listing all active robot subsystems (vision, planning, etc.)
    OS-level: Iterates through agent registry, reads process table

  agentsys spawn <id> <name> <caps>
    What it does: Creates a new agent with specified capabilities
    Use case: Test agent lifecycle or spawn custom agents
    Functionality: Registers agent with policy controller, assigns capabilities
    Parameters:
      <id>   - Unique agent ID (e.g., 100)
      <name> - Agent name (e.g., "test_agent")
      <caps> - Comma-separated capabilities (e.g., FsBasic,AudioControl)
    Robotics: Like spawning a new robot task with specific permissions
    OS-level: Allocates agent token, updates policy registry, creates process

...

Type 'help <command>' for detailed help on a specific command.
```

**Example of `help <command>`:**

```
sis> help agentsys

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
AGENTSYS - Agent Supervision Module
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

The Agent Supervision Module (ASM) provides comprehensive lifecycle
management, monitoring, and compliance tracking for LLM-driven agents.

Subcommands (arranged in testing order):

  status              Show ASM system status (8 subsystems)
  list                List all active agents
  spawn <id> <n> <c>  Spawn test agent with capabilities
  kill <id>           Terminate agent by ID
  restart <id>        Restart crashed or stopped agent
  metrics <id>        Show agent-specific telemetry metrics
  resources <id>      Show CPU/memory resource usage
  limits <id>         Display resource limits (quota, memory, syscall rate)
  telemetry           System-wide telemetry aggregation
  compliance          EU AI Act compliance report
  risk <id>           Show risk classification (Unacceptable, High, Limited, Minimal)
  deps <id>           Show agent dependencies
  depgraph            Visualize full dependency graph
  policy <id>         Show agent policy and capabilities
  policy-update <id>  Hot-patch agent policy without restart
  profile <id>        Show performance profiling data
  profile-reset [id]  Reset profiling counters
  dump                Debug dump (combines status, list, compliance, gateway)
  info <id>           Detailed agent information
  gwstatus            Cloud gateway status and routing info

Examples:
  agentsys status
  agentsys spawn 100 test_agent FsBasic,AudioControl
  agentsys metrics 100
  agentsys compliance

See also: asmstatus, asmlist, asminfo, asmpolicy (legacy commands)
```

### Implementation Steps

#### Step 1: Design Help System Architecture (30 min)

**File Structure:**
```
crates/kernel/src/shell/
  ├── help.rs              # New: Help system implementation
  ├── help_content.rs      # New: Help text content database
  └── mod.rs               # Modified: Import help module
```

**Data Structures:**
```rust
// help_content.rs

pub struct CommandHelp {
    pub name: &'static str,
    pub category: Category,
    pub what_it_does: &'static str,
    pub use_case: &'static str,
    pub functionality: &'static str,
    pub robotics_analogy: &'static str,
    pub os_level_explanation: &'static str,
    pub subcommands: &'static [SubcommandHelp],
    pub examples: &'static [&'static str],
    pub see_also: &'static [&'static str],
}

pub struct SubcommandHelp {
    pub name: &'static str,
    pub description: &'static str,
    pub parameters: &'static [ParameterHelp],
}

pub struct ParameterHelp {
    pub name: &'static str,
    pub description: &'static str,
    pub example: &'static str,
}

pub enum Category {
    CoreSystem,
    Filesystem,
    ProcessManagement,
    Network,
    MemoryManagement,
    AgentSupervision,
    AIPhase1,
    AIPhase2,
    AIPhase7,
    ChaosEngineering,
    Observability,
    Security,
    GraphicsUI,
    AudioMedia,
    Demos,
    Development,
}

impl Category {
    fn name(&self) -> &'static str { ... }
    fn order(&self) -> usize { ... }  // For sorting
}
```

#### Step 2: Create Help Content Database (2-3 hours)

Create comprehensive help entries for all 191 commands. Priority order:
1. **Phase 9 ASM commands** (26 commands) - 45 min
2. **Core system commands** (15 commands) - 30 min
3. **AI commands** (Phase 1, 2, 7) - 45 min
4. **Filesystem commands** - 30 min
5. **Other categories** - 1 hour

**Example entries:**
```rust
// help_content.rs

pub const AGENTSYS_HELP: CommandHelp = CommandHelp {
    name: "agentsys",
    category: Category::AgentSupervision,
    what_it_does: "Provides comprehensive agent lifecycle management, monitoring, and compliance tracking",
    use_case: "Managing LLM-driven agents with EU AI Act compliance",
    functionality: "Controls all aspects of agent supervision including spawning, killing, metrics, telemetry, compliance, dependencies, and profiling",
    robotics_analogy: "Like a master control system for all robot subsystems, ensuring safe operation and regulatory compliance",
    os_level_explanation: "Interfaces with Agent Supervisor module, maintains agent registry, enforces policies via Policy Controller, tracks telemetry through Telemetry Aggregator",
    subcommands: &[
        SubcommandHelp {
            name: "status",
            description: "Show ASM system status (8 subsystems)",
            parameters: &[],
        },
        SubcommandHelp {
            name: "spawn",
            description: "Spawn test agent with capabilities",
            parameters: &[
                ParameterHelp {
                    name: "id",
                    description: "Unique agent ID",
                    example: "100",
                },
                ParameterHelp {
                    name: "name",
                    description: "Agent name",
                    example: "test_agent",
                },
                ParameterHelp {
                    name: "caps",
                    description: "Comma-separated capabilities",
                    example: "FsBasic,AudioControl",
                },
            ],
        },
        // ... more subcommands
    ],
    examples: &[
        "agentsys status",
        "agentsys spawn 100 test_agent FsBasic",
        "agentsys metrics 100",
    ],
    see_also: &["asmstatus", "asmlist", "compliance"],
};
```

#### Step 3: Implement Help Command Logic (1 hour)

**Modify `shell.rs`:**
```rust
// shell.rs

pub fn cmd_help(&self, args: &[&str]) {
    if args.is_empty() {
        // Print categorized help
        help::print_all_help();
    } else {
        // Print specific command help
        let cmd_name = args[0];
        help::print_command_help(cmd_name);
    }
}
```

**Implement `help.rs`:**
```rust
// help.rs

use crate::shell::help_content::*;
use alloc::vec::Vec;

pub fn print_all_help() {
    println!("\nSIS Kernel Shell - Command Reference");
    println!("Commands are arranged categorically and in sequential testing order.\n");

    // Get all categories, sorted alphabetically
    let mut categories: Vec<Category> = vec![
        Category::AgentSupervision,
        Category::AIPhase1,
        Category::AIPhase2,
        // ... all categories
    ];
    categories.sort_by_key(|c| c.order());

    // Print each category
    for (idx, category) in categories.iter().enumerate() {
        print_category_help(idx + 1, category);
    }

    println!("\nType 'help <command>' for detailed help on a specific command.");
}

pub fn print_command_help(cmd_name: &str) {
    // Find command in help database
    if let Some(help_entry) = find_command_help(cmd_name) {
        print_detailed_help(help_entry);
    } else {
        println!("Unknown command: {}", cmd_name);
        println!("Type 'help' to see all available commands.");
    }
}

fn print_category_help(num: usize, category: &Category) {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("{}. {}", num, category.name().to_uppercase());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Get commands for this category
    let commands = get_commands_for_category(category);

    // Print each command
    for cmd_help in commands {
        print_command_brief(cmd_help);
    }
}

fn print_command_brief(help: &CommandHelp) {
    println!("  {}", help.name);
    println!("    What it does: {}", help.what_it_does);
    println!("    Use case: {}", help.use_case);
    println!("    Functionality: {}", help.functionality);
    println!("    Robotics: {}", help.robotics_analogy);
    println!("    OS-level: {}", help.os_level_explanation);

    if !help.subcommands.is_empty() {
        println!("    Subcommands:");
        for subcmd in help.subcommands {
            println!("      {} - {}", subcmd.name, subcmd.description);
        }
    }
    println!();
}

fn print_detailed_help(help: &CommandHelp) {
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("{} - {}", help.name.to_uppercase(), help.category.name());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    println!("{}\n", help.functionality);

    if !help.subcommands.is_empty() {
        println!("Subcommands (arranged in testing order):\n");
        for subcmd in help.subcommands {
            print!("  {:20}", subcmd.name);
            println!("{}", subcmd.description);

            if !subcmd.parameters.is_empty() {
                for param in subcmd.parameters {
                    println!("    <{}> - {} (example: {})",
                             param.name, param.description, param.example);
                }
            }
        }
    }

    if !help.examples.is_empty() {
        println!("\nExamples:");
        for example in help.examples {
            println!("  {}", example);
        }
    }

    if !help.see_also.is_empty() {
        println!("\nSee also: {}", help.see_also.join(", "));
    }

    println!();
}
```

#### Step 4: Testing (30 min)

1. **Test `help` command** - Verify categorical output
2. **Test `help <command>`** - Test with:
   - `help agentsys`
   - `help ls`
   - `help chaos`
   - `help modelctl`
   - `help invalid` (error case)
3. **Verify completeness** - All 73 top-level commands have help entries
4. **Check formatting** - Alignment, indentation, line breaks

---

## Implementation Timeline

### Phase 1: README Table (2 hours)
- [ ] Inventory all commands (30 min)
- [ ] Categorize commands (30 min)
- [ ] Create table in README (30 min)
- [ ] Verify completeness (30 min)

### Phase 2: Help System Foundation (1.5 hours)
- [ ] Design data structures (30 min)
- [ ] Create help.rs and help_content.rs (30 min)
- [ ] Implement basic help logic (30 min)

### Phase 3: Help Content Creation (3 hours)
- [ ] ASM commands (45 min)
- [ ] Core system commands (30 min)
- [ ] AI commands (45 min)
- [ ] Filesystem commands (30 min)
- [ ] Remaining categories (30 min)

### Phase 4: Testing & Refinement (30 min)
- [ ] Test all help output
- [ ] Fix formatting issues
- [ ] Add missing commands
- [ ] Update documentation

**Total Estimated Time: 6-7 hours**

---

## Success Criteria

### README Table
- ✅ Single comprehensive table with all commands
- ✅ 4 columns: Sr. No., Category, Command, Subcommands
- ✅ Categorically organized
- ✅ Commands ordered by usage sequence within categories
- ✅ Properly formatted and aligned
- ✅ All 73 top-level commands included
- ✅ All 118 subcommands documented

### Help Command
- ✅ `help` prints categorized list
- ✅ Categories in alphabetical order
- ✅ Commands in usage sequence within categories
- ✅ Detailed explanations with 5 components:
  - What it does
  - Use case
  - Functionality
  - Robotics analogy
  - OS-level explanation
- ✅ `help <command>` works for all commands
- ✅ Proper indentation and formatting
- ✅ Subcommand documentation included
- ✅ Examples and "See also" sections

---

## Files to Modify

### New Files
1. `docs/plans/SHELL_COMMANDS_REORGANIZATION_PLAN.md` - This plan
2. `crates/kernel/src/shell/help.rs` - Help system logic
3. `crates/kernel/src/shell/help_content.rs` - Help content database

### Modified Files
1. `README.md` - Add comprehensive shell commands table
2. `crates/kernel/src/shell.rs` - Update help command handler
3. `crates/kernel/src/shell/mod.rs` - Import help module

---

## Risk Assessment

### Low Risk
- README table creation (documentation only)
- Help content creation (no functional changes)

### Medium Risk
- Help command logic changes (could break existing help)
- Mitigation: Keep old help as fallback if new help fails

### High Risk
- None identified

---

## Next Steps After Completion

1. Update WEEK2_TROUBLESHOOTING.md with completion status
2. Run ASM integration tests to verify no regressions
3. Update README with help command usage examples
4. Consider adding shell command autocomplete in future

---

**Last Updated**: 2025-11-16
**Status**: Ready for implementation
