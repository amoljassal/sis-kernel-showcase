//! QEMU supervisor - manages QEMU process lifecycle
//!
//! Launches QEMU via scripts/uefi_run.sh, captures stdout/stderr,
//! and provides process control (run, stop, status).

pub mod supervisor;
pub mod types;

pub use supervisor::QemuSupervisor;
pub use types::{QemuConfig, QemuState, QemuStatus};
