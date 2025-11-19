// Early boot byte-ring logger exposed via /proc/bootlog
// Keeps a small circular buffer of raw bytes for very early messages

use crate::lib::ringbuf::RingBuffer;
use spin::Mutex;

const BOOTLOG_CAP: usize = 64 * 1024; // 64 KiB

struct ByteRing {
    inner: RingBuffer<u8, BOOTLOG_CAP>,
}

impl ByteRing {
    pub const fn new() -> Self { Self { inner: RingBuffer::new() } }
    pub fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            let _ = self.inner.push(b);
        }
    }
    pub fn drain_into(&mut self, out: &mut [u8]) -> usize {
        let mut n = 0;
        while n < out.len() {
            if let Some(b) = self.inner.pop() {
                out[n] = b;
                n += 1;
            } else { break; }
        }
        n
    }
}

static BOOTLOG: Mutex<ByteRing> = Mutex::new(ByteRing::new());

/// Write raw bytes to the early boot log ring
pub fn write(bytes: &[u8]) {
    let mut ring = BOOTLOG.lock();
    ring.write(bytes);
}

/// Drain bytes from the boot log into the provided buffer
/// Returns number of bytes written; subsequent reads see remaining bytes
pub fn drain(buf: &mut [u8]) -> usize {
    let mut ring = BOOTLOG.lock();
    ring.drain_into(buf)
}

