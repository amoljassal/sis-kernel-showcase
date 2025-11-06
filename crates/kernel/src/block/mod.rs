/// Block layer - abstraction for block devices
///
/// Provides a unified interface for block devices (virtio-blk, SCSI, etc.)
/// with request queuing and synchronous I/O operations.

pub mod partition;

use crate::lib::error::{Errno, Result};
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use alloc::collections::VecDeque;
use spin::Mutex;

/// Block device abstraction
pub struct BlockDevice {
    /// Device name (e.g., "vda", "sda")
    pub name: String,
    /// Major device number
    pub major: u32,
    /// Minor device number
    pub minor: u32,
    /// Capacity in sectors (512 bytes each)
    pub capacity_sectors: u64,
    /// Sector size in bytes (typically 512)
    pub sector_size: usize,
    /// Operations for this device
    pub ops: &'static dyn BlockDeviceOps,
}

impl BlockDevice {
    pub fn new(
        name: String,
        major: u32,
        minor: u32,
        capacity_sectors: u64,
        ops: &'static dyn BlockDeviceOps,
    ) -> Self {
        Self {
            name,
            major,
            minor,
            capacity_sectors,
            sector_size: 512, // Standard sector size
            ops,
        }
    }

    /// Read sectors from the device
    pub fn read_sectors(&self, sector: u64, buf: &mut [u8]) -> Result<()> {
        if sector >= self.capacity_sectors {
            return Err(Errno::EINVAL);
        }
        let sector_count = buf.len() / self.sector_size;
        if sector + sector_count as u64 > self.capacity_sectors {
            return Err(Errno::EINVAL);
        }
        self.ops.read_sectors(self, sector, buf)
    }

    /// Write sectors to the device
    pub fn write_sectors(&self, sector: u64, buf: &[u8]) -> Result<()> {
        if sector >= self.capacity_sectors {
            return Err(Errno::EINVAL);
        }
        let sector_count = buf.len() / self.sector_size;
        if sector + sector_count as u64 > self.capacity_sectors {
            return Err(Errno::EINVAL);
        }
        self.ops.write_sectors(self, sector, buf)
    }

    /// Flush any cached writes to disk
    pub fn flush(&self) -> Result<()> {
        self.ops.flush(self)
    }

    /// Get device capacity in bytes
    pub fn capacity_bytes(&self) -> u64 {
        self.capacity_sectors * self.sector_size as u64
    }
}

/// Block device operations trait
pub trait BlockDeviceOps: Send + Sync {
    /// Read sectors from the device
    fn read_sectors(&self, dev: &BlockDevice, sector: u64, buf: &mut [u8]) -> Result<()>;

    /// Write sectors to the device
    fn write_sectors(&self, dev: &BlockDevice, sector: u64, buf: &[u8]) -> Result<()>;

    /// Flush any cached writes
    fn flush(&self, dev: &BlockDevice) -> Result<()>;
}

/// Block I/O operation type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockOp {
    Read,
    Write,
    Flush,
}

/// Block I/O request
pub struct BlockRequest {
    /// Target device
    pub device: Arc<BlockDevice>,
    /// Operation type
    pub operation: BlockOp,
    /// Starting sector
    pub sector: u64,
    /// Data buffer
    pub buffer: Vec<u8>,
}

impl BlockRequest {
    pub fn new_read(device: Arc<BlockDevice>, sector: u64, size: usize) -> Self {
        Self {
            device,
            operation: BlockOp::Read,
            sector,
            buffer: vec![0u8; size],
        }
    }

    pub fn new_write(device: Arc<BlockDevice>, sector: u64, buffer: Vec<u8>) -> Self {
        Self {
            device,
            operation: BlockOp::Write,
            sector,
            buffer,
        }
    }

    pub fn new_flush(device: Arc<BlockDevice>) -> Self {
        Self {
            device,
            operation: BlockOp::Flush,
            sector: 0,
            buffer: Vec::new(),
        }
    }
}

/// Simple FIFO request queue for block I/O
pub struct RequestQueue {
    pending: Mutex<VecDeque<BlockRequest>>,
}

impl RequestQueue {
    pub fn new() -> Self {
        Self {
            pending: Mutex::new(VecDeque::new()),
        }
    }

    /// Submit a request to the queue
    pub fn submit(&self, request: BlockRequest) {
        self.pending.lock().push_back(request);
    }

    /// Get the next pending request
    pub fn pop(&self) -> Option<BlockRequest> {
        self.pending.lock().pop_front()
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.pending.lock().is_empty()
    }

    /// Get queue length
    pub fn len(&self) -> usize {
        self.pending.lock().len()
    }
}

/// Global registry of block devices
static BLOCK_DEVICES: Mutex<Vec<Arc<BlockDevice>>> = Mutex::new(Vec::new());

/// Register a block device
pub fn register_block_device(device: BlockDevice) -> Arc<BlockDevice> {
    let dev = Arc::new(device);
    BLOCK_DEVICES.lock().push(dev.clone());
    crate::info!("block: registered device {} ({}MB)",
                 dev.name, dev.capacity_bytes() / 1024 / 1024);
    dev
}

/// Get a block device by name
pub fn get_block_device(name: &str) -> Option<Arc<BlockDevice>> {
    BLOCK_DEVICES.lock().iter()
        .find(|dev| dev.name == name)
        .cloned()
}

/// List all registered block devices
pub fn list_block_devices() -> Vec<Arc<BlockDevice>> {
    BLOCK_DEVICES.lock().clone()
}

impl core::fmt::Debug for BlockDevice {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("BlockDevice")
            .field("name", &self.name)
            .field("major", &self.major)
            .field("minor", &self.minor)
            .field("capacity_sectors", &self.capacity_sectors)
            .field("capacity_mb", &(self.capacity_bytes() / 1024 / 1024))
            .finish()
    }
}
