//! Enhanced Help System for SIS Kernel Shell
//!
//! Provides categorized, comprehensive help for all shell commands with detailed
//! explanations, robotics analogies, and OS-level technical details.

use core::fmt;

/// Command categories for organization
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Category {
    AgentSupervision,
    AiPhase1,
    AiPhase2,
    AiPhase7,
    ChaosEngineering,
    Compliance,
    ControlPlane,
    CoreSystem,
    Demos,
    Deterministic,
    Development,
    Filesystem,
    Hardware,
    Llm,
    LoggingValidation,
    MemoryManagement,
    Network,
    Observability,
    StressTesting,
}

impl Category {
    /// Get category name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Category::AgentSupervision => "Agent Supervision",
            Category::AiPhase1 => "AI Phase 1",
            Category::AiPhase2 => "AI Phase 2",
            Category::AiPhase7 => "AI Phase 7",
            Category::ChaosEngineering => "Chaos Engineering",
            Category::Compliance => "Compliance",
            Category::ControlPlane => "Control Plane",
            Category::CoreSystem => "Core System",
            Category::Demos => "Demos",
            Category::Deterministic => "Deterministic",
            Category::Development => "Development",
            Category::Filesystem => "Filesystem",
            Category::Hardware => "Hardware",
            Category::Llm => "LLM",
            Category::LoggingValidation => "Logging & Validation",
            Category::MemoryManagement => "Memory Management",
            Category::Network => "Network",
            Category::Observability => "Observability",
            Category::StressTesting => "Stress Testing",
        }
    }

    /// Get category description
    pub fn description(&self) -> &'static str {
        match self {
            Category::AgentSupervision => "Oversees agent lifecycles, telemetry, compliance, and resource management",
            Category::AiPhase1 => "Neural agents for autonomous system control, prediction, and learning",
            Category::AiPhase2 => "Coordination, deployment, drift detection, and advanced ML orchestration",
            Category::AiPhase7 => "Model lifecycle management, decision tracing, and shadow deployments",
            Category::ChaosEngineering => "Fault injection and resilience testing for system hardening",
            Category::Compliance => "EU AI Act compliance reporting and validation",
            Category::ControlPlane => "Secure control plane with token-based authentication and admin tools",
            Category::CoreSystem => "Essential kernel operations, system info, and verification",
            Category::Demos => "Interactive demonstrations of AI, deterministic scheduling, and ML features",
            Category::Deterministic => "Deterministic scheduling and control graph pipeline management",
            Category::Development => "Testing, benchmarking, and validation tools for development",
            Category::Filesystem => "Virtual filesystem operations and block device management",
            Category::Hardware => "PCIe/RP1, SPI, I2C, sensors, PWM, GPIO, and firmware mailbox",
            Category::Llm => "Large language model inference, streaming, and model management",
            Category::LoggingValidation => "System logging, validation suites, and driver self-tests",
            Category::MemoryManagement => "AI-driven memory allocation prediction and optimization",
            Category::Network => "Network stack management and web server control",
            Category::Observability => "Performance monitoring, metrics collection, and profiling",
            Category::StressTesting => "Advanced stress testing and system benchmarks",
        }
    }

    /// Get robotics analogy for category
    pub fn robotics_analogy(&self) -> &'static str {
        match self {
            Category::AgentSupervision => "Robot swarm control center - monitors all agents, enforces policies",
            Category::AiPhase1 => "Autonomous navigation system - learns from environment, predicts obstacles",
            Category::AiPhase2 => "Multi-robot coordination - orchestrates fleet deployments and task allocation",
            Category::AiPhase7 => "Model versioning for robot behaviors - A/B testing and safe rollouts",
            Category::ChaosEngineering => "Fault simulation for robotic systems - tests resilience to actuator failures",
            Category::Compliance => "Safety certification logs - documents adherence to ISO/IEC standards",
            Category::ControlPlane => "Secure robot command center - authenticated control and admin access",
            Category::CoreSystem => "Robot operating system core - power management, status monitoring",
            Category::Demos => "Interactive robot demos - showcases capabilities to stakeholders",
            Category::Deterministic => "Real-time motion control - guarantees deterministic actuator timing",
            Category::Development => "Robot testing harness - validates behaviors before deployment",
            Category::Filesystem => "Robot data storage - logs sensor data, mission recordings",
            Category::Hardware => "Robot peripheral control - expansion buses, sensors, actuators, communication interfaces",
            Category::Llm => "Natural language interface for robots - voice commands and contextual understanding",
            Category::LoggingValidation => "Diagnostic logging - validates sensor calibration and system health",
            Category::MemoryManagement => "Predictive resource allocation - optimizes memory for sensor processing",
            Category::Network => "Robot fleet networking - communication between robots and base station",
            Category::Observability => "Performance telemetry - monitors CPU, memory, and actuator metrics",
            Category::StressTesting => "Endurance testing - validates robot performance under extreme loads",
        }
    }

    /// Get all categories in alphabetical order
    pub fn all() -> &'static [Category] {
        &[
            Category::AgentSupervision,
            Category::AiPhase1,
            Category::AiPhase2,
            Category::AiPhase7,
            Category::ChaosEngineering,
            Category::Compliance,
            Category::ControlPlane,
            Category::CoreSystem,
            Category::Demos,
            Category::Deterministic,
            Category::Development,
            Category::Filesystem,
            Category::Hardware,
            Category::Llm,
            Category::LoggingValidation,
            Category::MemoryManagement,
            Category::Network,
            Category::Observability,
            Category::StressTesting,
        ]
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Help information for a command parameter
#[derive(Debug, Clone)]
pub struct ParameterHelp {
    /// Parameter name (e.g., "path", "id", "value")
    pub name: &'static str,
    /// Parameter type description
    pub param_type: &'static str,
    /// Whether parameter is required
    pub required: bool,
    /// Parameter description
    pub description: &'static str,
}

/// Help information for a subcommand
#[derive(Debug, Clone)]
pub struct SubcommandHelp {
    /// Subcommand name
    pub name: &'static str,
    /// Brief description
    pub brief: &'static str,
    /// Usage pattern
    pub usage: &'static str,
    /// Parameters for this subcommand
    pub parameters: &'static [ParameterHelp],
    /// Example usage
    pub example: &'static str,
}

/// Comprehensive help information for a command
#[derive(Debug, Clone)]
pub struct CommandHelp {
    /// Command name
    pub name: &'static str,

    /// Category this command belongs to
    pub category: Category,

    /// Brief one-line description
    pub brief: &'static str,

    /// What the command does (technical explanation)
    pub what_it_does: &'static str,

    /// Use case (when to use this command)
    pub use_case: &'static str,

    /// Functionality (what operations it performs)
    pub functionality: &'static str,

    /// Robotics analogy (how this relates to robotics systems)
    pub robotics_analogy: &'static str,

    /// OS-level explanation (kernel/systems perspective)
    pub os_level_explanation: &'static str,

    /// Subcommands (if any)
    pub subcommands: &'static [SubcommandHelp],

    /// Usage examples
    pub examples: &'static [&'static str],

    /// Related commands
    pub see_also: &'static [&'static str],

    /// Feature requirements (None if always available)
    pub feature: Option<&'static str>,
}

impl CommandHelp {
    /// Check if command is available based on feature flags
    pub fn is_available(&self) -> bool {
        match self.feature {
            None => true,
            #[cfg(feature = "agentsys")]
            Some("agentsys") => true,
            #[cfg(feature = "llm")]
            Some("llm") => true,
            #[cfg(feature = "chaos")]
            Some("chaos") => true,
            #[cfg(feature = "model-lifecycle")]
            Some("model-lifecycle") => true,
            #[cfg(feature = "decision-traces")]
            Some("decision-traces") => true,
            #[cfg(feature = "shadow-mode")]
            Some("shadow-mode") => true,
            #[cfg(feature = "virtio-console")]
            Some("virtio-console") => true,
            #[cfg(feature = "profiling")]
            Some("profiling") => true,
            #[cfg(feature = "graph-demo")]
            Some("graph-demo") => true,
            #[cfg(feature = "deterministic")]
            Some("deterministic") => true,
            Some(_) => false,
        }
    }
}

/// Get help for a specific command by name
pub fn get_command_help(name: &str) -> Option<&'static CommandHelp> {
    super::help_content::ALL_COMMANDS
        .iter()
        .find(|cmd| cmd.name == name)
}

/// Get all available commands grouped by category
pub fn get_commands_by_category() -> alloc::vec::Vec<(Category, alloc::vec::Vec<&'static CommandHelp>)> {
    use alloc::vec::Vec;

    let mut categories: Vec<(Category, Vec<&'static CommandHelp>)> = Vec::new();

    // Group commands by category
    for category in Category::all() {
        let mut commands: Vec<&'static CommandHelp> = super::help_content::ALL_COMMANDS
            .iter()
            .filter(|cmd| cmd.category == *category && cmd.is_available())
            .collect();

        if !commands.is_empty() {
            // Sort commands by name (usage order is already in the array order)
            // Keep array order to maintain usage sequence
            categories.push((*category, commands));
        }
    }

    categories
}

/// Print comprehensive help (all commands, categorized)
pub fn print_help() {
    unsafe {
        crate::uart_print(b"\n");
        crate::uart_print(b"================================================================================\n");
        crate::uart_print(b"                    SIS KERNEL SHELL - COMMAND REFERENCE\n");
        crate::uart_print(b"================================================================================\n");
        crate::uart_print(b"\n");
        crate::uart_print(b"Commands are organized by category and listed in recommended usage order.\n");
        crate::uart_print(b"Use 'help <command>' for detailed information about a specific command.\n");
        crate::uart_print(b"\n");

        let commands_by_category = get_commands_by_category();

        for (idx, (category, commands)) in commands_by_category.iter().enumerate() {
            // Print category header
            crate::uart_print(b"[");
            print_number(idx + 1);
            crate::uart_print(b"] ");
            print_str(category.as_str());
            crate::uart_print(b"\n");
            crate::uart_print(b"    ");
            print_str(category.description());
            crate::uart_print(b"\n");
            crate::uart_print(b"    Robotics: ");
            print_str(category.robotics_analogy());
            crate::uart_print(b"\n");
            crate::uart_print(b"----------------------------------------\n");

            // Print commands in this category
            for cmd in commands {
                crate::uart_print(b"  ");
                print_str(cmd.name);

                // Pad command name to align descriptions
                let padding = if cmd.name.len() < 16 {
                    16 - cmd.name.len()
                } else {
                    1
                };
                for _ in 0..padding {
                    crate::uart_print(b" ");
                }

                crate::uart_print(b"- ");
                print_str(cmd.brief);
                crate::uart_print(b"\n");

                // Print subcommands if any
                if !cmd.subcommands.is_empty() {
                    for subcmd in cmd.subcommands {
                        crate::uart_print(b"    ");
                        print_str(subcmd.name);

                        let sub_padding = if subcmd.name.len() < 14 {
                            14 - subcmd.name.len()
                        } else {
                            1
                        };
                        for _ in 0..sub_padding {
                            crate::uart_print(b" ");
                        }

                        crate::uart_print(b"- ");
                        print_str(subcmd.brief);
                        crate::uart_print(b"\n");
                    }
                }
            }

            crate::uart_print(b"\n");
        }

        crate::uart_print(b"================================================================================\n");
        crate::uart_print(b"Total Commands: ");
        print_number(super::help_content::ALL_COMMANDS.len());
        crate::uart_print(b" | Categories: ");
        print_number(commands_by_category.len());
        crate::uart_print(b"\n");
        crate::uart_print(b"================================================================================\n");
        crate::uart_print(b"\n");
    }
}

/// Print detailed help for a specific command
pub fn print_command_help(name: &str) {
    match get_command_help(name) {
        Some(cmd) => {
            unsafe {
                crate::uart_print(b"\n");
                crate::uart_print(b"================================================================================\n");
                crate::uart_print(b"COMMAND: ");
                print_str(cmd.name);
                crate::uart_print(b"\n");
                crate::uart_print(b"================================================================================\n");
                crate::uart_print(b"\n");

                // Category
                crate::uart_print(b"Category: ");
                print_str(cmd.category.as_str());
                crate::uart_print(b"\n\n");

                // Brief description
                crate::uart_print(b"          DESCRIPTION:\n");
                crate::uart_print(b"            ");
                print_str(cmd.brief);
                crate::uart_print(b"\n\n");

                // What it does
                crate::uart_print(b"          WHAT IT DOES:\n");
                print_wrapped(cmd.what_it_does, 12);
                crate::uart_print(b"\n");

                // Use case
                crate::uart_print(b"          USE CASE:\n");
                print_wrapped(cmd.use_case, 12);
                crate::uart_print(b"\n");

                // Functionality
                crate::uart_print(b"          FUNCTIONALITY:\n");
                print_wrapped(cmd.functionality, 12);
                crate::uart_print(b"\n");

                // Robotics analogy
                crate::uart_print(b"          ROBOTICS ANALOGY:\n");
                print_wrapped(cmd.robotics_analogy, 12);
                crate::uart_print(b"\n");

                // OS-level explanation
                crate::uart_print(b"          OS-LEVEL EXPLANATION:\n");
                print_wrapped(cmd.os_level_explanation, 12);
                crate::uart_print(b"\n");

                // Subcommands
                if !cmd.subcommands.is_empty() {
                    crate::uart_print(b"          SUBCOMMANDS:\n");
                    for subcmd in cmd.subcommands {
                        crate::uart_print(b"            ");
                        print_str(subcmd.name);
                        crate::uart_print(b"\n");
                        crate::uart_print(b"              ");
                        print_str(subcmd.brief);
                        crate::uart_print(b"\n");
                        crate::uart_print(b"              Usage: ");
                        print_str(subcmd.usage);
                        crate::uart_print(b"\n");

                        if !subcmd.parameters.is_empty() {
                            crate::uart_print(b"              Parameters:\n");
                            for param in subcmd.parameters {
                                crate::uart_print(b"                ");
                                print_str(param.name);
                                crate::uart_print(b" (");
                                print_str(param.param_type);
                                crate::uart_print(b")");
                                if param.required {
                                    crate::uart_print(b" [required]");
                                } else {
                                    crate::uart_print(b" [optional]");
                                }
                                crate::uart_print(b"\n                  ");
                                print_str(param.description);
                                crate::uart_print(b"\n");
                            }
                        }

                        crate::uart_print(b"              Example: ");
                        print_str(subcmd.example);
                        crate::uart_print(b"\n\n");
                    }
                }

                // Examples
                if !cmd.examples.is_empty() {
                    crate::uart_print(b"          EXAMPLES:\n");
                    for example in cmd.examples {
                        crate::uart_print(b"            ");
                        print_str(example);
                        crate::uart_print(b"\n");
                    }
                    crate::uart_print(b"\n");
                }

                // See also
                if !cmd.see_also.is_empty() {
                    crate::uart_print(b"          SEE ALSO:\n");
                    crate::uart_print(b"            ");
                    for (i, related) in cmd.see_also.iter().enumerate() {
                        if i > 0 {
                            crate::uart_print(b", ");
                        }
                        print_str(related);
                    }
                    crate::uart_print(b"\n\n");
                }

                crate::uart_print(b"================================================================================\n");
                crate::uart_print(b"\n");
            }
        }
        None => {
            unsafe {
                crate::uart_print(b"Unknown command: ");
                print_str(name);
                crate::uart_print(b"\n");
                crate::uart_print(b"Type 'help' for a list of available commands.\n");
            }
        }
    }
}

/// Helper to print a string
fn print_str(s: &str) {
    unsafe {
        crate::uart_print(s.as_bytes());
    }
}

/// Helper to print a number
fn print_number(n: usize) {
    let mut buf = [0u8; 20];
    let mut idx = 0;
    let mut num = n;

    if num == 0 {
        unsafe { crate::uart_print(b"0"); }
        return;
    }

    while num > 0 {
        buf[idx] = b'0' + (num % 10) as u8;
        num /= 10;
        idx += 1;
    }

    unsafe {
        for i in (0..idx).rev() {
            crate::uart_print(&[buf[i]]);
        }
    }
}

/// Helper to print wrapped text with indentation
fn print_wrapped(text: &str, indent: usize) {
    for _ in 0..indent {
        unsafe { crate::uart_print(b" "); }
    }
    print_str(text);
    unsafe { crate::uart_print(b"\n"); }
}
