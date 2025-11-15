// User space memory access helpers
// Phase A0 - Basic stubs, full implementation in Phase A1 with address space

use crate::lib::error::{Errno, Result};
use core::ptr;

/// Kernel address space starts here (upper half)
const KERNEL_BASE: usize = 0xFFFF_0000_0000_0000;

/// Copy data from user space to kernel space
///
/// Phase A0: Basic validation only
/// Phase A1: Will add full address space validation and page fault handling
pub fn copy_from_user<T>(user_ptr: *const T, count: usize) -> Result<alloc::vec::Vec<T>>
where
    T: Copy,
{
    // Validate pointer is in user address space
    if (user_ptr as usize) >= KERNEL_BASE {
        return Err(Errno::EFAULT);
    }

    // Check for overflow
    let total_size = count
        .checked_mul(core::mem::size_of::<T>())
        .ok_or(Errno::EINVAL)?;

    if total_size == 0 {
        return Ok(alloc::vec::Vec::new());
    }

    // Phase A0: Simple copy (no page fault handling yet)
    // Phase A1: Add address space validation and fault handling
    let mut buf = alloc::vec::Vec::with_capacity(count);

    unsafe {
        ptr::copy_nonoverlapping(user_ptr, buf.as_mut_ptr(), count);
        buf.set_len(count);
    }

    Ok(buf)
}

/// Copy data from kernel space to user space
///
/// Phase A0: Basic validation only
/// Phase A1: Will add full address space validation
pub fn copy_to_user<T>(user_ptr: *mut T, data: &[T]) -> Result<()>
where
    T: Copy,
{
    if (user_ptr as usize) >= KERNEL_BASE {
        return Err(Errno::EFAULT);
    }

    if data.is_empty() {
        return Ok(());
    }

    // Phase A0: Simple copy
    unsafe {
        ptr::copy_nonoverlapping(data.as_ptr(), user_ptr, data.len());
    }

    Ok(())
}

/// Copy a string from user space
///
/// Reads until NULL terminator or max_len bytes
pub fn copy_string_from_user(user_ptr: *const u8, max_len: usize) -> Result<alloc::string::String> {
    if (user_ptr as usize) >= KERNEL_BASE {
        return Err(Errno::EFAULT);
    }

    let mut bytes = alloc::vec::Vec::new();

    unsafe {
        for i in 0..max_len {
            let byte = ptr::read(user_ptr.add(i));
            if byte == 0 {
                break;
            }
            bytes.push(byte);
        }
    }

    alloc::string::String::from_utf8(bytes).map_err(|_| Errno::EINVAL)
}

/// Validate that a user pointer is readable
///
/// Phase A0: Basic range check only
/// Phase A1: Will check page tables and permissions
pub fn validate_user_read(ptr: *const u8, len: usize) -> Result<()> {
    let addr = ptr as usize;

    if addr >= KERNEL_BASE {
        return Err(Errno::EFAULT);
    }

    let end = addr.checked_add(len).ok_or(Errno::EFAULT)?;

    if end >= KERNEL_BASE {
        return Err(Errno::EFAULT);
    }

    Ok(())
}

/// Validate that a user pointer is writable
///
/// Phase A0: Basic range check only
/// Phase A1: Will check page tables and permissions
pub fn validate_user_write(ptr: *mut u8, len: usize) -> Result<()> {
    validate_user_read(ptr as *const u8, len)
}
