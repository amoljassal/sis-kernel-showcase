//! Boot logging system that captures kernel output to memory
//! and writes it to a file on the boot USB drive for debugging

use core::sync::atomic::{AtomicUsize, Ordering};
use spin::Mutex;

/// Maximum boot log size (256KB should be plenty)
const BOOT_LOG_SIZE: usize = 256 * 1024;

/// Static buffer for boot log
static mut BOOT_LOG_BUFFER: [u8; BOOT_LOG_SIZE] = [0u8; BOOT_LOG_SIZE];

/// Current position in the boot log
static BOOT_LOG_POS: AtomicUsize = AtomicUsize::new(0);

/// Lock for boot log operations
static BOOT_LOG_LOCK: Mutex<()> = Mutex::new(());

/// Append data to the boot log buffer
pub fn append(data: &[u8]) {
    let _lock = BOOT_LOG_LOCK.lock();

    let pos = BOOT_LOG_POS.load(Ordering::Relaxed);
    let remaining = BOOT_LOG_SIZE.saturating_sub(pos);

    if remaining == 0 {
        return; // Buffer full
    }

    let to_write = data.len().min(remaining);
    unsafe {
        let dest = BOOT_LOG_BUFFER.as_mut_ptr().add(pos);
        core::ptr::copy_nonoverlapping(data.as_ptr(), dest, to_write);
    }

    BOOT_LOG_POS.store(pos + to_write, Ordering::Relaxed);
}

/// Append a string to the boot log
pub fn append_str(s: &str) {
    append(s.as_bytes());
}

/// Get the current boot log contents
pub fn get_log() -> &'static [u8] {
    let pos = BOOT_LOG_POS.load(Ordering::Relaxed);
    unsafe {
        &BOOT_LOG_BUFFER[..pos]
    }
}

/// Get the current boot log size
pub fn size() -> usize {
    BOOT_LOG_POS.load(Ordering::Relaxed)
}

/// Try to write the boot log to VFS if available
pub fn try_save_to_vfs() {
    use crate::vfs::OpenFlags;

    let log_data = get_log();
    if log_data.is_empty() {
        return;
    }

    // Try to create /boot.log using VFS
    match crate::vfs::create("/boot.log", 0o644, OpenFlags::O_WRONLY | OpenFlags::O_CREAT | OpenFlags::O_TRUNC) {
        Ok(file) => {
            match file.write(log_data) {
                Ok(written) => {
                    append_str(&alloc::format!("\n[BOOTLOG] Wrote {} bytes to /boot.log\n", written));
                }
                Err(_) => {
                    append_str("\n[BOOTLOG] Failed to write to /boot.log\n");
                }
            }
            // File is automatically closed when dropped
        }
        Err(_) => {
            // VFS not ready yet or file system not mounted
            append_str("\n[BOOTLOG] VFS not ready, log kept in memory\n");
        }
    }
}

/// Print the boot log to UART (for shell command)
pub fn print_to_uart() {
    let log_data = get_log();
    unsafe {
        crate::uart::write_bytes(b"\n=== BOOT LOG START ===\n");
        crate::uart::write_bytes(log_data);
        crate::uart::write_bytes(b"\n=== BOOT LOG END ===\n");
    }
}
