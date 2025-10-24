//! Lightweight tracing and METRIC emission utilities.
//! Designed for no_std, low-overhead serial output.

#[inline(always)]
pub fn metric_kv(name: &str, value: usize) {
    unsafe {
        crate::uart_print(b"METRIC ");
        print_str(name);
        crate::uart_print(b"=");
        print_usize(value);
        crate::uart_print(b"\n");
    }
    // Note: demo metric rings are disabled during bring-up to avoid early-boot contention.
}

#[inline(always)]
pub fn trace(tag: &str) {
    unsafe {
        crate::uart_print(b"[TRACE] ");
        print_str(tag);
        crate::uart_print(b"\n");
    }
}

#[inline(always)]
pub fn op_start(op_id: u32) {
    unsafe {
        crate::uart_print(b"[TRACE] op_start id=");
        print_usize(op_id as usize);
        crate::uart_print(b"\n");
    }
}

#[inline(always)]
pub fn op_queued(op_id: u32) {
    unsafe {
        crate::uart_print(b"[TRACE] op_queued id=");
        print_usize(op_id as usize);
        crate::uart_print(b"\n");
    }
}

#[inline(always)]
pub fn op_end_ns(op_id: u32, ns: u64) {
    unsafe {
        crate::uart_print(b"[TRACE] op_end id=");
        print_usize(op_id as usize);
        crate::uart_print(b" ns=");
        print_usize(ns as usize);
        crate::uart_print(b"\n");
    }
}

#[inline(always)]
pub fn ch_depth(ch_id: usize, depth: usize) {
    unsafe {
        crate::uart_print(b"[TRACE] ch_depth id=");
        print_usize(ch_id);
        crate::uart_print(b" depth=");
        print_usize(depth);
        crate::uart_print(b"\n");
    }
}

#[inline(always)]
pub unsafe fn print_str(s: &str) {
    crate::uart_print(s.as_bytes());
}

#[inline(always)]
pub unsafe fn print_usize(mut num: usize) {
    if num == 0 {
        crate::uart_print(b"0");
        return;
    }
    let mut buf = [0u8; 20];
    let mut i = 0;
    while num > 0 {
        buf[i] = b'0' + (num % 10) as u8;
        num /= 10;
        i += 1;
    }
    while i > 0 {
        i -= 1;
        crate::uart_print(&[buf[i]]);
    }
}

// Demo metric rings disabled: provide stubs for snapshotters
#[inline(always)]
pub fn metrics_snapshot_ctx_switch(_out: &mut [usize]) -> usize { 0 }
#[inline(always)]
pub fn metrics_snapshot_memory_alloc(_out: &mut [usize]) -> usize { 0 }
#[inline(always)]
pub fn metrics_snapshot_real_ctx(_out: &mut [usize]) -> usize { 0 }
