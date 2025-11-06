//! QEMU supervisor - manages QEMU process lifecycle
//!
//! Launches QEMU via scripts/uefi_run.sh, captures stdout/stderr,
//! and provides process control (run, stop, status).

pub mod replay;
pub mod shell;
pub mod shell_executor;
pub mod supervisor;
pub mod types;

pub use replay::{ReplayManager, ReplaySpeed, ReplayState, ReplayStatus, ReplayTransport};
pub use shell::{SelfCheckResponse, ShellCommandRequest, ShellCommandResponse, TestResultEntry};
pub use shell_executor::ShellExecutor;
pub use supervisor::{QemuEvent, QemuSupervisor};
pub use types::{QemuConfig, QemuState, QemuStatus};
