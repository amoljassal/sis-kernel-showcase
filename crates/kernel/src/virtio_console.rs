//! VirtIO Console driver for SIS kernel
//!
//! Implements VirtIO console device driver for enhanced I/O performance
//! Provides character-based I/O through VirtIO virtqueues

use crate::driver::{DeviceId, DeviceInfo, Driver, DriverError, DriverInfo, DriverResult};
use core::sync::atomic::{AtomicUsize, Ordering};
use crate::virtio::{VirtIODeviceType, VirtIOMMIOOffset, VirtIOMMIOTransport};

// Simple page-aligned DMA pool (module scope) for virtqueue memory
const VCON_DMA_POOL_SIZE: usize = 256 * 1024; // 256 KiB
#[repr(align(4096))]
struct VconDmaPool([u8; VCON_DMA_POOL_SIZE]);
static mut VCON_DMA_POOL: VconDmaPool = VconDmaPool([0; VCON_DMA_POOL_SIZE]);
static VCON_DMA_OFF: AtomicUsize = AtomicUsize::new(0);

/// VirtIO Console feature bits
#[repr(u32)]
pub enum VirtIOConsoleFeatures {
    /// Console has multiple ports
    MultiPort = 1 << 1,
    /// Console supports emergency write
    EmergWrite = 1 << 2,
}

/// VirtIO Console configuration space
#[repr(C)]
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
struct VirtIOConsoleConfig {
    /// Number of columns
    cols: u16,
    /// Number of rows  
    rows: u16,
    /// Maximum number of ports
    max_nr_ports: u32,
    /// Emergency write character
    emerg_wr: u32,
}

/// VirtIO Console control message types
#[repr(u16)]
pub enum VirtIOConsoleControlType {
    DeviceReady = 0,
    DeviceAdd = 1,
    DeviceRemove = 2,
    PortReady = 3,
    ConsolePort = 4,
    Resize = 5,
    PortOpen = 6,
    PortName = 7,
}

/// VirtIO Console control message
#[repr(C)]
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
struct VirtIOConsoleControl {
    /// Port ID
    id: u32,
    /// Event type
    event: u16,
    /// Value
    value: u16,
}


/// VirtQueue descriptor
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct VirtQueueDesc {
    /// Buffer address
    addr: u64,
    /// Buffer length
    len: u32,
    /// Descriptor flags
    flags: u16,
    /// Next descriptor index
    next: u16,
}

/// VirtQueue available ring
#[repr(C)]
#[derive(Debug)]
struct VirtQueueAvail {
    /// Flags
    flags: u16,
    /// Index
    idx: u16,
    /// Ring of descriptor indices
    ring: [u16; 256],
    /// Used event (optional)
    used_event: u16,
}

/// VirtQueue used ring element
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct VirtQueueUsedElem {
    /// Descriptor index
    id: u32,
    /// Bytes written
    len: u32,
}

/// VirtQueue used ring
#[repr(C)]
#[derive(Debug)]
struct VirtQueueUsed {
    /// Flags
    flags: u16,
    /// Index
    idx: u16,
    /// Ring of used elements
    ring: [VirtQueueUsedElem; 256],
    /// Available event (optional)
    avail_event: u16,
}

/// VirtQueue implementation
struct VirtQueue {
    /// Queue index
    #[allow(dead_code)]
    index: u16,
    /// Queue size
    size: u16,
    /// Descriptor table
    desc_table: *mut VirtQueueDesc,
    /// Available ring
    avail_ring: *mut VirtQueueAvail,
    /// Used ring
    used_ring: *mut VirtQueueUsed,
    /// Next available descriptor
    next_desc: u16,
    /// Last seen used index
    last_used_idx: u16,
}

impl VirtQueue {
    /// Create new virtqueue (simplified for basic console)
    unsafe fn new(index: u16, size: u16, desc_addr: u64) -> Self {
        let desc_table = desc_addr as *mut VirtQueueDesc;
        let avail_ring = (desc_addr + (size as u64 * 16)) as *mut VirtQueueAvail;
        let used_ring =
            (desc_addr + (size as u64 * 16) + (size as u64 * 2) + 6) as *mut VirtQueueUsed;

        // Initialize descriptor table
        for i in 0..size {
            let desc = &mut *desc_table.add(i as usize);
            desc.addr = 0;
            desc.len = 0;
            desc.flags = 0;
            desc.next = (i + 1) % size;
        }

        // Initialize available ring
        (*avail_ring).flags = 0;
        (*avail_ring).idx = 0;

        // Initialize used ring
        (*used_ring).flags = 0;
        (*used_ring).idx = 0;

        VirtQueue {
            index,
            size,
            desc_table,
            avail_ring,
            used_ring,
            next_desc: 0,
            last_used_idx: 0,
        }
    }

    /// Add buffer to queue
    unsafe fn add_buffer(&mut self, addr: u64, len: u32, flags: u16) -> Result<(), DriverError> {
        if self.next_desc >= self.size {
            return Err(DriverError::ResourceError);
        }

        let desc_idx = self.next_desc;
        let desc = &mut *self.desc_table.add(desc_idx as usize);

        desc.addr = addr;
        desc.len = len;
        desc.flags = flags;

        // Add to available ring
        let avail_idx = (*self.avail_ring).idx as usize % self.size as usize;
        (*self.avail_ring).ring[avail_idx] = desc_idx;

        // Update available index
        (*self.avail_ring).idx = (*self.avail_ring).idx.wrapping_add(1);

        self.next_desc = (self.next_desc + 1) % self.size;

        Ok(())
    }

    /// Check for used buffers
    unsafe fn get_used_buffer(&mut self) -> Option<(u32, u32)> {
        if self.last_used_idx == (*self.used_ring).idx {
            return None;
        }

        let used_elem = (*self.used_ring).ring[self.last_used_idx as usize % self.size as usize];
        self.last_used_idx = self.last_used_idx.wrapping_add(1);

        Some((used_elem.id, used_elem.len))
    }
}

/// VirtIO Console driver
pub struct VirtIOConsoleDriver {
    transport: Option<VirtIOMMIOTransport>,
    receiveq: Option<VirtQueue>,
    transmitq: Option<VirtQueue>,
    ctrl_rxq: Option<VirtQueue>,
    ctrl_txq: Option<VirtQueue>,
    buffer: [u8; 4096],
    /// Dedicated RX buffer for receive queue
    rx_buffer: [u8; 4096],
    initialized: bool,
    multip: bool,
    selected_port: Option<u32>,
    ctl_buf: [u8; 1024],
    ctl_len: usize,
    ctrl_buf: [u8; 512],
    #[allow(dead_code)]
    bound_name: [u8; 32],
    #[allow(dead_code)]
    bound_len: usize,
    // Metrics counters
    ctl_frames_rx: usize,
    ctl_frames_tx: usize,
    ctl_errors: usize,
    ctl_backpressure_drops: usize,
}

impl VirtIOConsoleDriver {
    fn dma_alloc(size: usize, align: usize) -> u64 {
        // Very small bump allocator; single-thread use in early boot
        let off = VCON_DMA_OFF.load(Ordering::Relaxed);
        let base = unsafe { (&raw const VCON_DMA_POOL.0) as *const u8 as usize };
        let align_mask = align.saturating_sub(1);
        let aligned = (off + align_mask) & !align_mask;
        let end = aligned.saturating_add(size);
        let _ = VCON_DMA_OFF.store(end, Ordering::Relaxed);
        (base + aligned) as u64
    }
    /// Create new VirtIO console driver
    pub const fn new() -> Self {
        VirtIOConsoleDriver {
            transport: None,
            receiveq: None,
            transmitq: None,
            ctrl_rxq: None,
            ctrl_txq: None,
            buffer: [0; 4096],
            rx_buffer: [0; 4096],
            initialized: false,
            multip: false,
            selected_port: None,
            ctl_buf: [0; 1024],
            ctl_len: 0,
            ctrl_buf: [0; 512],
            bound_name: [0; 32],
            bound_len: 0,
            ctl_frames_rx: 0,
            ctl_frames_tx: 0,
            ctl_errors: 0,
            ctl_backpressure_drops: 0,
        }
    }

    /// Initialize virtqueues
    unsafe fn init_virtqueues(&mut self, _device: &DeviceInfo) -> DriverResult<()> {
        let transport = self.transport.as_ref().ok_or(DriverError::InitFailed)?;

        // Select queue 0 (receiveq)
        transport.write_reg(VirtIOMMIOOffset::QueueSel, 0);
        let queue0_size_hw = transport.read_reg(VirtIOMMIOOffset::QueueNumMax);
        // Clamp to a sane size to avoid huge allocations/writes if QEMU reports a large max
        let queue0_size = core::cmp::min(queue0_size_hw, 256);

        if queue0_size == 0 {
            return Err(DriverError::NotSupported);
        }

        // Allocate memory for queue 0
        let q0_bytes = (queue0_size as usize * 16) + (queue0_size as usize * 2) + 6 + (queue0_size as usize * 8) + 6;
        let queue0_addr = Self::dma_alloc(q0_bytes, 4096);
        self.receiveq = Some(VirtQueue::new(0, queue0_size as u16, queue0_addr));

        // Set queue 0 addresses
        transport.write_reg(
            VirtIOMMIOOffset::QueueDescLow,
            (queue0_addr & 0xFFFFFFFF) as u32,
        );
        transport.write_reg(VirtIOMMIOOffset::QueueDescHigh, (queue0_addr >> 32) as u32);

        let avail_addr = queue0_addr + (queue0_size as u64 * 16);
        transport.write_reg(
            VirtIOMMIOOffset::QueueAvailLow,
            (avail_addr & 0xFFFFFFFF) as u32,
        );
        transport.write_reg(VirtIOMMIOOffset::QueueAvailHigh, (avail_addr >> 32) as u32);

        let used_addr = avail_addr + (queue0_size as u64 * 2) + 6;
        transport.write_reg(
            VirtIOMMIOOffset::QueueUsedLow,
            (used_addr & 0xFFFFFFFF) as u32,
        );
        transport.write_reg(VirtIOMMIOOffset::QueueUsedHigh, (used_addr >> 32) as u32);

        // Set queue size and enable
        transport.write_reg(VirtIOMMIOOffset::QueueNum, queue0_size);
        transport.write_reg(VirtIOMMIOOffset::QueueReady, 1);

        // Prime receive queue with one RX buffer
        if let Some(recv) = self.receiveq.as_mut() {
            // VIRTQ_DESC_F_WRITE = 2 (device writes into buffer)
            const VIRTQ_DESC_F_WRITE: u16 = 1 << 1;
            let _ = recv.add_buffer(self.rx_buffer.as_ptr() as u64, self.rx_buffer.len() as u32, VIRTQ_DESC_F_WRITE);
            // Notify queue 0 that buffer is available
            transport.write_reg(VirtIOMMIOOffset::QueueNotify, 0);
        }

        // Select queue 1 (transmitq)
        transport.write_reg(VirtIOMMIOOffset::QueueSel, 1);
        let queue1_size_hw = transport.read_reg(VirtIOMMIOOffset::QueueNumMax);
        let queue1_size = core::cmp::min(queue1_size_hw, 256);

        if queue1_size > 0 {
            let q1_bytes = (queue1_size as usize * 16) + (queue1_size as usize * 2) + 6 + (queue1_size as usize * 8) + 6;
            let queue1_addr = Self::dma_alloc(q1_bytes, 4096);
            self.transmitq = Some(VirtQueue::new(1, queue1_size as u16, queue1_addr));

            // Set queue 1 addresses
            transport.write_reg(
                VirtIOMMIOOffset::QueueDescLow,
                (queue1_addr & 0xFFFFFFFF) as u32,
            );
            transport.write_reg(VirtIOMMIOOffset::QueueDescHigh, (queue1_addr >> 32) as u32);

            let avail_addr = queue1_addr + (queue1_size as u64 * 16);
            transport.write_reg(
                VirtIOMMIOOffset::QueueAvailLow,
                (avail_addr & 0xFFFFFFFF) as u32,
            );
            transport.write_reg(VirtIOMMIOOffset::QueueAvailHigh, (avail_addr >> 32) as u32);

            let used_addr = avail_addr + (queue1_size as u64 * 2) + 6;
            transport.write_reg(
                VirtIOMMIOOffset::QueueUsedLow,
                (used_addr & 0xFFFFFFFF) as u32,
            );
            transport.write_reg(VirtIOMMIOOffset::QueueUsedHigh, (used_addr >> 32) as u32);

            // Set queue size and enable
            transport.write_reg(VirtIOMMIOOffset::QueueNum, queue1_size);
            transport.write_reg(VirtIOMMIOOffset::QueueReady, 1);
        }

        Ok(())
    }

    /// Negotiate features (detect MultiPort)
    unsafe fn negotiate_features(&mut self, transport: &VirtIOMMIOTransport) -> u32 {
        // Read device features (low 32 bits)
        transport.write_reg(VirtIOMMIOOffset::DeviceFeaturesSel, 0);
        let dev = transport.read_reg(VirtIOMMIOOffset::DeviceFeatures);
        let mut features: u32 = 0;
        if (dev & (VirtIOConsoleFeatures::MultiPort as u32)) != 0 {
            self.multip = true;
            features |= VirtIOConsoleFeatures::MultiPort as u32;
        }
        features
    }

    /// Write data using VirtIO console
    pub fn write_data(&mut self, data: &[u8]) -> DriverResult<usize> {
        if !self.initialized {
            return Err(DriverError::InitFailed);
        }

        let transport = self.transport.as_ref().ok_or(DriverError::InitFailed)?;
        let transmitq = self.transmitq.as_mut().ok_or(DriverError::NotSupported)?;

        unsafe {
            // Copy data to buffer
            let len = core::cmp::min(data.len(), self.buffer.len());
            self.buffer[..len].copy_from_slice(&data[..len]);

            // Add buffer to transmit queue
            transmitq.add_buffer(self.buffer.as_ptr() as u64, len as u32, 0)?;

            // Notify device
            transport.write_reg(VirtIOMMIOOffset::QueueNotify, 1);

            // Wait for completion (simplified)
            for _ in 0..1000 {
                if let Some((_, written)) = transmitq.get_used_buffer() {
                    return Ok(written as usize);
                }
                core::hint::spin_loop();
            }

            Ok(len)
        }
    }

    /// Read data from VirtIO console
    pub fn read_data(&mut self, buffer: &mut [u8]) -> DriverResult<usize> {
        if !self.initialized {
            return Err(DriverError::InitFailed);
        }

        let receiveq = self.receiveq.as_mut().ok_or(DriverError::NotSupported)?;
        let transport = self.transport.as_ref().ok_or(DriverError::InitFailed)?;

        unsafe {
            if let Some((_, len)) = receiveq.get_used_buffer() {
                let read_len = core::cmp::min(len as usize, buffer.len());
                buffer[..read_len].copy_from_slice(&self.rx_buffer[..read_len]);

                // Re-post RX buffer to receive more data
                const VIRTQ_DESC_F_WRITE: u16 = 1 << 1;
                let _ = receiveq.add_buffer(self.rx_buffer.as_ptr() as u64, self.rx_buffer.len() as u32, VIRTQ_DESC_F_WRITE);
                // Notify device that RX buffer is available again (queue 0)
                transport.write_reg(VirtIOMMIOOffset::QueueNotify, 0);
                return Ok(read_len);
            }
        }

        Ok(0)
    }

    /// Poll the receive queue for control-plane frames following V0 framing.
    /// This treats all incoming virtconsole input as control frames.
    pub unsafe fn poll_control_frames(&mut self) -> DriverResult<()> {
        if !self.initialized { return Ok(()); }
        // Read into scratch buffer
        let mut tmp = [0u8; 256];
        loop {
            match self.read_data(&mut tmp) {
                Ok(n) if n > 0 => {
                    // Append to ctl_buf
                    let space = self.ctl_buf.len().saturating_sub(self.ctl_len);
                    let to_copy = core::cmp::min(space, n);
                    self.ctl_buf[self.ctl_len..self.ctl_len+to_copy].copy_from_slice(&tmp[..to_copy]);
                    self.ctl_len += to_copy;
                    // Try to parse frames
                    self.process_ctl_buffer();
                }
                _ => break,
            }
        }
        Ok(())
    }

    unsafe fn process_ctl_buffer(&mut self) {
        // Minimal V0 parser: look for 'C'(0x43), ver=0, then cmd, flags, len(u32), payload
        let mut offset = 0usize;
        while self.ctl_len.saturating_sub(offset) >= 8 {
            if self.ctl_buf[offset] != 0x43 { offset += 1; continue; }
            let ver = self.ctl_buf[offset+1];
            if ver != 0 { offset += 1; continue; }
            let len = u32::from_le_bytes([
                self.ctl_buf[offset+4],
                self.ctl_buf[offset+5],
                self.ctl_buf[offset+6],
                self.ctl_buf[offset+7],
            ]) as usize;
            if self.ctl_len.saturating_sub(offset) < 8 + len { break; }
            let frame = &self.ctl_buf[offset..offset+8+len];
            let t0 = crate::graph::now_cycles();
            let res = crate::control::handle_frame(frame);
            self.ctl_frames_rx = self.ctl_frames_rx.saturating_add(1);
            match res {
                Ok(()) => {
                    crate::uart_print(b"[CTL] ok\n");
                    let _ = self.send_ack(true);
                }
                Err(e) => {
                    crate::uart_print(b"[CTL] error\n");
                    self.ctl_errors = self.ctl_errors.saturating_add(1);
                    // Map error to a small hex code for host visibility
                    let code: u8 = match e {
                        crate::control::CtrlError::BadFrame => 0x01,
                        crate::control::CtrlError::Unsupported => 0x02,
                        crate::control::CtrlError::NoGraph => 0x03,
                        crate::control::CtrlError::Oversize => { crate::trace::metric_kv("ctl_oversize", 1); 0x04 },
                        crate::control::CtrlError::AuthFailed => 0x05,
                    };
                    let _ = self.send_ack_code(code);
                    // Periodically emit counters
                    if (self.ctl_errors & 0xFF) == 0 { crate::trace::metric_kv("ctl_errors", self.ctl_errors); }
                }
            }
            let t1 = crate::graph::now_cycles();
            let dt_ns = crate::graph::cycles_to_ns(t1.saturating_sub(t0));
            // Emit lightweight roundtrip timing in microseconds
            crate::trace::metric_kv("ctl_roundtrip_us", (dt_ns / 1000) as usize);
            offset += 8 + len;
        }
        if offset > 0 {
            // shift remaining
            let remain = self.ctl_len - offset;
            for i in 0..remain { self.ctl_buf[i] = self.ctl_buf[offset+i]; }
            self.ctl_len = remain;
        }
    }

    /// Poll control RX queue for multiport events (if enabled)
    pub unsafe fn poll_ctrl_events(&mut self) -> DriverResult<()> {
        if !self.initialized || !self.multip { return Ok(()); }
        // Defer actions that require &mut self outside of the ctrl queue borrow
        let mut port_to_open: Option<u32> = None;
        if let Some(ctrl) = self.ctrl_rxq.as_mut() {
            while let Some((_id, len)) = ctrl.get_used_buffer() {
                let n = core::cmp::min(len as usize, self.ctrl_buf.len());
                if n >= core::mem::size_of::<VirtIOConsoleControl>() {
                    let hdr_ptr = self.ctrl_buf.as_ptr() as *const VirtIOConsoleControl;
                    let hdr = core::ptr::read_unaligned(hdr_ptr);
                    match hdr.event {
                        x if x == VirtIOConsoleControlType::DeviceAdd as u16 => crate::uart_print(b"[VCON] CTRL DeviceAdd\n"),
                        x if x == VirtIOConsoleControlType::DeviceRemove as u16 => crate::uart_print(b"[VCON] CTRL DeviceRemove\n"),
                        x if x == VirtIOConsoleControlType::PortReady as u16 => crate::uart_print(b"[VCON] CTRL PortReady\n"),
                        x if x == VirtIOConsoleControlType::PortOpen as u16 => crate::uart_print(b"[VCON] CTRL PortOpen\n"),
                        x if x == VirtIOConsoleControlType::PortName as u16 => {
                            crate::uart_print(b"[VCON] CTRL PortName\n");
                            // Remaining bytes after header contain the UTF-8 name (may be NUL-terminated)
                            let name_off = core::mem::size_of::<VirtIOConsoleControl>();
                            let max = core::cmp::min(n.saturating_sub(name_off), 32);
                            let mut name_tmp = [0u8; 32];
                            for i in 0..max { name_tmp[i] = self.ctrl_buf[name_off + i]; }
                            // Check for sis.datactl substring
                            let mut matched = false;
                            const PAT: &[u8] = b"sis.datactl";
                            'scan: for i in 0..max {
                                if name_tmp[i] == 0 { break 'scan; }
                                if i + PAT.len() <= max {
                                    let mut ok = true;
                                    for j in 0..PAT.len() { if name_tmp[i+j] != PAT[j] { ok = false; break; } }
                                    if ok { matched = true; break 'scan; }
                                }
                            }
                            if matched { port_to_open = Some(hdr.id); }
                        }
                        _ => crate::uart_print(b"[VCON] CTRL event\n"),
                    }
                }
                // Re-post buffer
                let added = ctrl.add_buffer(self.ctrl_buf.as_ptr() as u64, self.ctrl_buf.len() as u32, 1 << 1);
                if added.is_err() { self.ctl_backpressure_drops = self.ctl_backpressure_drops.saturating_add(1); }
                if let Some(t) = &self.transport { t.write_reg(VirtIOMMIOOffset::QueueNotify, 2); }
            }
        }
        // Perform deferred actions now that ctrl_rxq borrow has ended
        if let Some(pid) = port_to_open {
            self.selected_port = Some(pid);
            crate::trace::metric_kv("ctl_selected_port", pid as usize);
            let _ = self.send_ctrl_event(VirtIOConsoleControlType::PortOpen as u16, pid, 1);
            crate::trace::metric_kv("ctl_port_bound", 1);
            crate::uart_print(b"[VCON] BOUND port to sis.datactl\n");
        }
        Ok(())
    }

    /// Send a minimal ACK/ERR response back to host over the data TX queue.
    /// This keeps the kernel framing (V0) write-only and provides simple status.
    unsafe fn send_ack(&mut self, ok: bool) -> DriverResult<()> {
        let msg: &[u8] = if ok { b"OK\n" } else { b"ERR\n" };
        if let Some(tq) = self.transmitq.as_mut() {
            if let Some(transport) = &self.transport {
                // Copy message into buffer and submit
                let n = core::cmp::min(msg.len(), self.buffer.len());
                self.buffer[..n].copy_from_slice(&msg[..n]);
                match tq.add_buffer(self.buffer.as_ptr() as u64, n as u32, 0) {
                    Ok(()) => {
                        transport.write_reg(VirtIOMMIOOffset::QueueNotify, 1);
                        // Poll for completion briefly
                        for _ in 0..256 {
                            if let Some((_id, _len)) = tq.get_used_buffer() { break; }
                            core::hint::spin_loop();
                        }
                        self.ctl_frames_tx = self.ctl_frames_tx.saturating_add(1);
                        // Emit counters periodically to avoid log spam
                        if (self.ctl_frames_tx & 0xFF) == 0 {
                            crate::trace::metric_kv("ctl_frames_rx", self.ctl_frames_rx);
                            crate::trace::metric_kv("ctl_frames_tx", self.ctl_frames_tx);
                            crate::trace::metric_kv("ctl_errors", self.ctl_errors);
                            crate::trace::metric_kv("ctl_backpressure_drops", self.ctl_backpressure_drops);
                        }
                        Ok(())
                    }
                    Err(_) => {
                        self.ctl_backpressure_drops = self.ctl_backpressure_drops.saturating_add(1);
                        Err(DriverError::ResourceError)
                    }
                }
            } else { Err(DriverError::InitFailed) }
        } else { Ok(()) }
    }

    /// Send an ERR with a hex code: "ERR 0xNN\n"
    unsafe fn send_ack_code(&mut self, code: u8) -> DriverResult<()> {
        let mut msg = [0u8; 12];
        // Build b"ERR 0xNN\n"
        let s = b"ERR 0x";
        msg[..6].copy_from_slice(s);
        let hex = b"0123456789ABCDEF";
        msg[6] = hex[(code >> 4) as usize];
        msg[7] = hex[(code & 0xF) as usize];
        msg[8] = b'\n';
        if let Some(tq) = self.transmitq.as_mut() {
            if let Some(transport) = &self.transport {
                let _ = tq.add_buffer(msg.as_ptr() as u64, 9, 0);
                transport.write_reg(VirtIOMMIOOffset::QueueNotify, 1);
                for _ in 0..256 { if let Some((_id,_len)) = tq.get_used_buffer() { break; } core::hint::spin_loop(); }
            }
        }
        Ok(())
    }

    /// Send a control event (e.g., PortOpen) on the control TX queue
    unsafe fn send_ctrl_event(&mut self, event: u16, id: u32, value: u16) -> DriverResult<()> {
        if let Some(txq) = self.ctrl_txq.as_mut() {
            if let Some(transport) = &self.transport {
                let mut msg = VirtIOConsoleControl { id, event, value };
                let ptr = &mut msg as *mut VirtIOConsoleControl as u64;
                let len = core::mem::size_of::<VirtIOConsoleControl>() as u32;
                match txq.add_buffer(ptr, len, 0) {
                    Ok(()) => {
                        transport.write_reg(VirtIOMMIOOffset::QueueNotify, 3);
                        // Poll briefly for completion
                        for _ in 0..256 { if let Some((_id, _len)) = txq.get_used_buffer() { break; } core::hint::spin_loop(); }
                        Ok(())
                    }
                    Err(_) => Err(DriverError::ResourceError),
                }
            } else { Err(DriverError::InitFailed) }
        } else { Err(DriverError::NotSupported) }
    }
}

impl Driver for VirtIOConsoleDriver {
    fn info(&self) -> DriverInfo {
        DriverInfo {
            name: "VirtIO Console",
            version: "1.0.0",
            supported_devices: &[DeviceId {
                vendor_id: 0x1AF4, // Red Hat (VirtIO)
                device_id: 3,      // Console
                class: 0x07,       // Communication controller
                subclass: 0x80,    // Other
            }],
        }
    }

    fn probe(&self, device: &DeviceInfo) -> bool {
        device.id.vendor_id == 0x1AF4 && device.id.device_id == 3 && device.id.class == 0x07
    }

    fn init(&mut self, device: &DeviceInfo) -> DriverResult<()> {
        unsafe {
            crate::uart_print(b"[VIRTIO-CONSOLE] Initializing VirtIO console driver\n");
        }

        // Create VirtIO transport
        let transport = VirtIOMMIOTransport::new(device.base_addr, device.size, device.irq)?;

        // Verify this is a console device
        if transport.device_type() != VirtIODeviceType::Console {
            return Err(DriverError::InvalidDevice);
        }

        unsafe {
            crate::uart_print(b"[VIRTIO-CONSOLE] Device verified as VirtIO console\n");
        }

        // Negotiate features
        let feats = unsafe { self.negotiate_features(&transport) };
        transport.init_device(feats)?;

        unsafe {
            crate::uart_print(b"[VIRTIO-CONSOLE] Device initialization complete\n");
        }

        self.transport = Some(transport);

        // Initialize virtqueues (data queues always)
        unsafe { self.init_virtqueues(device)?; }

        // Initialize control queues for multiport if present
        if self.multip {
            unsafe { crate::uart_print(b"[VIRTIO-CONSOLE] MULTIPORT enabled\n"); }
            let transport = self.transport.as_ref().ok_or(DriverError::InitFailed)?;
            // CTRL RX on queue 2
            unsafe {
                transport.write_reg(VirtIOMMIOOffset::QueueSel, 2);
                let qsz = transport.read_reg(VirtIOMMIOOffset::QueueNumMax);
                if qsz > 0 {
                    let bytes = (qsz as usize * 16) + (qsz as usize * 2) + 6 + (qsz as usize * 8) + 6;
                    let base = Self::dma_alloc(bytes, 4096);
                    self.ctrl_rxq = Some(VirtQueue::new(2, qsz as u16, base));
                    transport.write_reg(VirtIOMMIOOffset::QueueDescLow, (base & 0xFFFF_FFFF) as u32);
                    transport.write_reg(VirtIOMMIOOffset::QueueDescHigh, (base >> 32) as u32);
                    let avail = base + (qsz as u64 * 16);
                    transport.write_reg(VirtIOMMIOOffset::QueueAvailLow, (avail & 0xFFFF_FFFF) as u32);
                    transport.write_reg(VirtIOMMIOOffset::QueueAvailHigh, (avail >> 32) as u32);
                    let used = avail + (qsz as u64 * 2) + 6;
                    transport.write_reg(VirtIOMMIOOffset::QueueUsedLow, (used & 0xFFFF_FFFF) as u32);
                    transport.write_reg(VirtIOMMIOOffset::QueueUsedHigh, (used >> 32) as u32);
                    transport.write_reg(VirtIOMMIOOffset::QueueNum, qsz);
                    transport.write_reg(VirtIOMMIOOffset::QueueReady, 1);
                    if let Some(q) = self.ctrl_rxq.as_mut() {
                        let _ = q.add_buffer(self.ctrl_buf.as_ptr() as u64, self.ctrl_buf.len() as u32, 1 << 1);
                        transport.write_reg(VirtIOMMIOOffset::QueueNotify, 2);
                    }
                }
                // CTRL TX on queue 3
                transport.write_reg(VirtIOMMIOOffset::QueueSel, 3);
                let qsz3 = transport.read_reg(VirtIOMMIOOffset::QueueNumMax);
                if qsz3 > 0 {
                    let bytes = (qsz3 as usize * 16) + (qsz3 as usize * 2) + 6 + (qsz3 as usize * 8) + 6;
                    let base = Self::dma_alloc(bytes, 4096);
                    self.ctrl_txq = Some(VirtQueue::new(3, qsz3 as u16, base));
                    transport.write_reg(VirtIOMMIOOffset::QueueDescLow, (base & 0xFFFF_FFFF) as u32);
                    transport.write_reg(VirtIOMMIOOffset::QueueDescHigh, (base >> 32) as u32);
                    let avail = base + (qsz3 as u64 * 16);
                    transport.write_reg(VirtIOMMIOOffset::QueueAvailLow, (avail & 0xFFFF_FFFF) as u32);
                    transport.write_reg(VirtIOMMIOOffset::QueueAvailHigh, (avail >> 32) as u32);
                    let used = avail + (qsz3 as u64 * 2) + 6;
                    transport.write_reg(VirtIOMMIOOffset::QueueUsedLow, (used & 0xFFFF_FFFF) as u32);
                    transport.write_reg(VirtIOMMIOOffset::QueueUsedHigh, (used >> 32) as u32);
                    transport.write_reg(VirtIOMMIOOffset::QueueNum, qsz3);
                    transport.write_reg(VirtIOMMIOOffset::QueueReady, 1);
                }
            }
        }

        unsafe {
            crate::uart_print(b"[VIRTIO-CONSOLE] Virtqueues initialized\n");
        }

        Ok(())
    }

    fn start(&mut self) -> DriverResult<()> {
        if let Some(transport) = &self.transport {
            // Mark driver as ready
            transport.driver_ready();

            unsafe {
                crate::uart_print(b"[VIRTIO-CONSOLE] Driver marked as ready\n");
            }

            self.initialized = true;

            Ok(())
        } else {
            Err(DriverError::InitFailed)
        }
    }

    fn stop(&mut self) -> DriverResult<()> {
        if let Some(transport) = &self.transport {
            // Reset device
            transport.reset_device()?;
            unsafe {
                crate::uart_print(b"[VIRTIO-CONSOLE] Device reset\n");
            }
        }

        self.initialized = false;
        Ok(())
    }

    fn handle_irq(&mut self) -> DriverResult<()> {
        if !self.initialized {
            return Ok(());
        }

        let transport = self.transport.as_ref().ok_or(DriverError::InitFailed)?;

        // Read and acknowledge interrupts
        let int_status = transport.read_reg(VirtIOMMIOOffset::InterruptStatus);
        if int_status != 0 {
            transport.write_reg(VirtIOMMIOOffset::InterruptACK, int_status);

            unsafe {
                crate::uart_print(b"[VIRTIO-CONSOLE] Interrupt handled: ");
                self.print_hex(int_status);
                crate::uart_print(b"\n");
            }
        }

        // Attempt to drain RX and process control frames
        unsafe { let _ = self.poll_control_frames(); }
        // Poll control events if multiport
        unsafe { let _ = self.poll_ctrl_events(); }
        Ok(())
    }

    fn read(&mut self, _offset: u64, buffer: &mut [u8]) -> DriverResult<usize> {
        self.read_data(buffer)
    }

    fn write(&mut self, _offset: u64, data: &[u8]) -> DriverResult<usize> {
        self.write_data(data)
    }

}

impl VirtIOConsoleDriver {
    /// Helper to print numbers
    #[allow(dead_code)]
    unsafe fn print_number(&self, mut num: u32) {
        if num == 0 {
            crate::uart_print(b"0");
            return;
        }

        let mut digits = [0u8; 10];
        let mut i = 0;

        while num > 0 {
            digits[i] = b'0' + (num % 10) as u8;
            num /= 10;
            i += 1;
        }

        while i > 0 {
            i -= 1;
            crate::uart_print(&[digits[i]]);
        }
    }

    /// Helper to print hex numbers
    unsafe fn print_hex(&self, num: u32) {
        crate::uart_print(b"0x");
        for i in (0..8).rev() {
            let nibble = (num >> (i * 4)) & 0xF;
            let c = if nibble < 10 {
                b'0' + nibble as u8
            } else {
                b'A' + (nibble - 10) as u8
            };
            crate::uart_print(&[c]);
        }
    }
}

/// Global VirtIO console driver instance
static mut VIRTIO_CONSOLE_DRIVER: VirtIOConsoleDriver = VirtIOConsoleDriver::new();

/// Get reference to global VirtIO console driver
pub fn get_virtio_console_driver() -> &'static mut VirtIOConsoleDriver {
    unsafe {
        let driver_ptr = &raw mut VIRTIO_CONSOLE_DRIVER;
        &mut *driver_ptr
    }
}
