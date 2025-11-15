//! Userspace programs for SIS kernel testing
//!
//! Contains simple test programs to validate syscall interface and
//! process management functionality.

pub mod hello;

/// Embedded userspace program data (placeholder for now)
pub static HELLO_PROGRAM: &[u8] = &[];

/// Load and execute the hello world test program
pub fn load_hello_program() -> Result<(), &'static str> {
    use crate::process::{scheduler, ElfLoader};
    use alloc::string::ToString;
    
    let scheduler = scheduler();
    
    // For now, create a simple process manually since we don't have full ELF loading
    let hello_pid = scheduler.create_process(
        "hello".to_string(),
        HELLO_PROGRAM,
        Some(0), // Parent is kernel (PID 0)
    )?;
    
    crate::kernel::serial::write_str("[USERSPACE] Created hello program, PID ");
    crate::kernel::serial::write_dec(hello_pid as u64);
    crate::kernel::serial::write_str("\n");
    
    Ok(())
}

/// Initialize userspace subsystem
pub fn init_userspace() {
    crate::kernel::serial::write_str("[USERSPACE] Initializing userspace subsystem...\n");
    
    // Load test programs
    match load_hello_program() {
        Ok(_) => crate::kernel::serial::write_str("[USERSPACE] Hello program loaded successfully\n"),
        Err(e) => {
            crate::kernel::serial::write_str("[USERSPACE] Failed to load hello program: ");
            crate::kernel::serial::write_str(e);
            crate::kernel::serial::write_str("\n");
        }
    }
}