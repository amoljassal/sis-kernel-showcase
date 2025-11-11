/// VirtIO Block Device Driver
///
/// Implements virtio-blk device (Device ID 2) for block storage
/// Supports read/write operations through virtqueues
///
/// Phase 8 Milestone 3: Enhanced with zero-copy DMA and pipelined I/O
/// for improved throughput (target: >100 MB/s sequential reads).

use crate::lib::error::{Result, Errno};
use crate::virtio::{VirtIOMMIOTransport, VirtIOMMIOOffset};
use crate::virtio::virtqueue::{VirtQueue, PREFERRED_QUEUE_SIZE};
use crate::block::{BlockDevice, BlockDeviceOps, register_block_device};
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use alloc::string::String;
use spin::Mutex;
use core::ptr::NonNull;

/// VirtIO Block Device feature bits
const VIRTIO_BLK_F_SIZE_MAX: u32 = 1 << 1;    // Maximum segment size
const VIRTIO_BLK_F_SEG_MAX: u32 = 1 << 2;     // Maximum number of segments
const VIRTIO_BLK_F_GEOMETRY: u32 = 1 << 4;    // Disk geometry available
const VIRTIO_BLK_F_RO: u32 = 1 << 5;          // Device is read-only
const VIRTIO_BLK_F_BLK_SIZE: u32 = 1 << 6;    // Block size of disk
const VIRTIO_BLK_F_FLUSH: u32 = 1 << 9;       // Cache flush command support
const VIRTIO_BLK_F_TOPOLOGY: u32 = 1 << 10;   // Topology information available

/// VirtIO Block request types
const VIRTIO_BLK_T_IN: u32 = 0;       // Read
const VIRTIO_BLK_T_OUT: u32 = 1;      // Write
const VIRTIO_BLK_T_FLUSH: u32 = 4;    // Flush
const VIRTIO_BLK_T_DISCARD: u32 = 11; // Discard
const VIRTIO_BLK_T_WRITE_ZEROES: u32 = 13; // Write zeroes

/// VirtIO Block request status
const VIRTIO_BLK_S_OK: u8 = 0;
const VIRTIO_BLK_S_IOERR: u8 = 1;
const VIRTIO_BLK_S_UNSUPP: u8 = 2;

/// DMA buffer pool configuration (Phase 8)
const DMA_BUFFER_COUNT: usize = 64;
const DMA_BUFFER_SIZE: usize = 4096;

/// VirtIO Block device configuration
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct VirtioBlkConfig {
    /// Capacity in 512-byte sectors
    capacity: u64,
    /// Maximum segment size
    size_max: u32,
    /// Maximum number of segments
    seg_max: u32,
    /// Geometry (cylinders, heads, sectors)
    geometry: VirtioBlkGeometry,
    /// Block size (logical block size)
    blk_size: u32,
    /// Topology information
    topology: VirtioBlkTopology,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct VirtioBlkGeometry {
    cylinders: u16,
    heads: u8,
    sectors: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct VirtioBlkTopology {
    physical_block_exp: u8,
    alignment_offset: u8,
    min_io_size: u16,
    opt_io_size: u32,
}

/// VirtIO Block request header
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct VirtioBlkReq {
    /// Request type (VIRTIO_BLK_T_*)
    req_type: u32,
    /// Reserved (must be 0)
    reserved: u32,
    /// Starting sector
    sector: u64,
}

/// DMA buffer for zero-copy operations (Phase 8)
#[derive(Debug)]
struct DmaBuffer {
    /// Physical address
    physical_addr: u64,
    /// Virtual address
    virtual_addr: NonNull<u8>,
    /// In use flag
    in_use: bool,
}

// SAFETY: DmaBuffer contains raw pointers but is only accessed while holding the
// buffer pool's mutex lock. The pointers are valid kernel memory addresses that
// remain valid for the lifetime of the DMA buffer.
unsafe impl Send for DmaBuffer {}
unsafe impl Sync for DmaBuffer {}

/// DMA buffer pool for zero-copy I/O (Phase 8)
struct BufferPool {
    /// Pre-allocated DMA buffers
    buffers: Vec<DmaBuffer>,
    /// Free list (indices of available buffers)
    free_list: Vec<usize>,
}

impl BufferPool {
    /// Create new buffer pool with pre-allocated DMA buffers
    fn new() -> Result<Self> {
        let mut buffers = Vec::with_capacity(DMA_BUFFER_COUNT);
        let mut free_list = Vec::with_capacity(DMA_BUFFER_COUNT);

        for i in 0..DMA_BUFFER_COUNT {
            // Allocate physically contiguous page (4KB)
            let page_phys = crate::mm::alloc_page()
                .ok_or(Errno::ENOMEM)?;

            // Get virtual address (direct mapping assumed)
            let page_virt = crate::mm::phys_to_virt(page_phys);
            let virtual_addr = NonNull::new(page_virt as *mut u8)
                .ok_or(Errno::ENOMEM)?;

            buffers.push(DmaBuffer {
                physical_addr: page_phys,
                virtual_addr,
                in_use: false,
            });

            free_list.push(i);
        }

        Ok(BufferPool { buffers, free_list })
    }

    /// Allocate a DMA buffer from the pool
    fn allocate(&mut self) -> Option<usize> {
        let idx = self.free_list.pop()?;
        self.buffers[idx].in_use = true;
        Some(idx)
    }

    /// Free a DMA buffer back to the pool
    fn free(&mut self, idx: usize) {
        if idx < self.buffers.len() && self.buffers[idx].in_use {
            self.buffers[idx].in_use = false;
            self.free_list.push(idx);
        }
    }

    /// Find buffer index by virtual address
    fn find_buffer(&self, addr: usize) -> Option<usize> {
        self.buffers.iter().position(|buf| {
            let buf_addr = buf.virtual_addr.as_ptr() as usize;
            addr >= buf_addr && addr < buf_addr + DMA_BUFFER_SIZE
        })
    }

    /// Get buffer by index
    fn get(&self, idx: usize) -> Option<&DmaBuffer> {
        self.buffers.get(idx)
    }

    /// Get statistics
    fn stats(&self) -> (usize, usize) {
        (self.buffers.len(), self.free_list.len())
    }
}

/// VirtIO Block device driver
pub struct VirtioBlkDevice {
    /// MMIO transport
    transport: Arc<Mutex<VirtIOMMIOTransport>>,
    /// Request virtqueue
    queue: Arc<Mutex<VirtQueue>>,
    /// Device capacity in sectors
    capacity_sectors: u64,
    /// Block size in bytes
    block_size: u32,
    /// Device name
    name: String,
    /// DMA buffer pool (Phase 8: zero-copy support)
    dma_pool: Mutex<BufferPool>,
}

impl VirtioBlkDevice {
    /// Initialize a VirtIO block device
    pub fn new(transport: VirtIOMMIOTransport, name: String) -> Result<Self> {
        let transport = Arc::new(Mutex::new(transport));

        // Negotiate features (common path)
        let features = VIRTIO_BLK_F_SIZE_MAX | VIRTIO_BLK_F_SEG_MAX | VIRTIO_BLK_F_BLK_SIZE | VIRTIO_BLK_F_FLUSH;
        transport.lock().init_device(features).map_err(|_| Errno::EIO)?;

        // Read device configuration
        let config = Self::read_config(&transport.lock());
        crate::info!("virtio-blk: capacity = {} sectors ({} MB)",
                     config.capacity, config.capacity / 2048);
        crate::info!("virtio-blk: block_size = {} bytes", config.blk_size);

        // Initialize virtqueue (queue 0), handle legacy (version==1) and modern (>=2)
        let version = transport.lock().version();
        let queue = if version >= 2 {
            // Modern path: use split virtqueues with explicit addresses
            // Phase 8: Prefer larger queue size (256) for better throughput
            let queue_size = {
                let t = transport.lock();
                t.write_reg(VirtIOMMIOOffset::QueueSel, 0);
                let max_size = t.read_reg(VirtIOMMIOOffset::QueueNumMax);
                if max_size == 0 || max_size > 32768 { return Err(Errno::EINVAL); }
                // Use preferred size (256) if device supports it, otherwise use max
                core::cmp::min(PREFERRED_QUEUE_SIZE, max_size as u16)
            };
            crate::info!("virtio-blk: using queue size {}", queue_size);
            let q = VirtQueue::new(0, queue_size)?;
            {
                let t = transport.lock();
                t.write_reg(VirtIOMMIOOffset::QueueSel, 0);
                t.write_reg(VirtIOMMIOOffset::QueueNum, queue_size as u32);
                let (desc_addr, avail_addr, used_addr) = q.get_addresses();
                t.write_reg(VirtIOMMIOOffset::QueueDescLow, (desc_addr & 0xFFFF_FFFF) as u32);
                t.write_reg(VirtIOMMIOOffset::QueueDescHigh, (desc_addr >> 32) as u32);
                t.write_reg(VirtIOMMIOOffset::QueueAvailLow, (avail_addr & 0xFFFF_FFFF) as u32);
                t.write_reg(VirtIOMMIOOffset::QueueAvailHigh, (avail_addr >> 32) as u32);
                t.write_reg(VirtIOMMIOOffset::QueueUsedLow, (used_addr & 0xFFFF_FFFF) as u32);
                t.write_reg(VirtIOMMIOOffset::QueueUsedHigh, (used_addr >> 32) as u32);
                t.write_reg(VirtIOMMIOOffset::QueueReady, 1);
            }
            q
        } else {
            // Legacy path: contiguous region + QueuePFN/QueueAlign/GuestPageSize
            // Phase 8: Prefer larger queue size (256) for better throughput
            let queue_size = {
                let t = transport.lock();
                t.write_reg(VirtIOMMIOOffset::QueueSel, 0);
                let max_size = t.read_reg(VirtIOMMIOOffset::QueueNumMax);
                if max_size == 0 || max_size > 32768 { return Err(Errno::EINVAL); }
                core::cmp::min(PREFERRED_QUEUE_SIZE, max_size as u16)
            };
            crate::info!("virtio-blk: using queue size {} (legacy)", queue_size);
            let q = VirtQueue::new_contiguous(0, queue_size)?;
            {
                let t = transport.lock();
                t.write_reg(VirtIOMMIOOffset::GuestPageSize, 4096);
                t.write_reg(VirtIOMMIOOffset::QueueSel, 0);
                t.write_reg(VirtIOMMIOOffset::QueueNum, queue_size as u32);
                t.write_reg(VirtIOMMIOOffset::QueueAlign, 4096);
                let pfn = (q.region_base() >> 12) as u32;
                t.write_reg(VirtIOMMIOOffset::QueuePFN, pfn);
                // No QueueReady in legacy
            }
            q
        };

        // Mark driver as ready (common)
        transport.lock().driver_ready();

        let queue = Arc::new(Mutex::new(queue));

        // Phase 8: Initialize DMA buffer pool for zero-copy I/O
        let dma_pool = BufferPool::new()?;
        crate::info!("virtio-blk: initialized DMA buffer pool ({} buffers)", DMA_BUFFER_COUNT);

        Ok(Self {
            transport,
            queue,
            capacity_sectors: config.capacity,
            block_size: config.blk_size,
            name,
            dma_pool: Mutex::new(dma_pool),
        })
    }

    /// Read device configuration
    fn read_config(transport: &VirtIOMMIOTransport) -> VirtioBlkConfig {
        // Read capacity as two u32 values from config space (offsets 0 and 4)
        let cap_lo = transport.read_config_u32(0) as u64;
        let cap_hi = transport.read_config_u32(4) as u64;
        let capacity = (cap_hi << 32) | cap_lo;

        // Block size (common offset for virtio-blk)
        let blk_size = transport.read_config_u32(20);

        VirtioBlkConfig {
            capacity,
            size_max: 0,
            seg_max: 0,
            geometry: VirtioBlkGeometry { cylinders: 0, heads: 0, sectors: 0 },
            blk_size: if blk_size == 0 { 512 } else { blk_size },
            topology: VirtioBlkTopology {
                physical_block_exp: 0,
                alignment_offset: 0,
                min_io_size: 0,
                opt_io_size: 0,
            },
        }
    }

    /// Submit a block I/O request
    fn submit_request(&self, req_type: u32, sector: u64, buffer: &mut [u8]) -> Result<usize> {
        // Allocate request header
        let mut req_header = VirtioBlkReq {
            req_type,
            reserved: 0,
            sector,
        };
        let req_header_addr = &mut req_header as *mut VirtioBlkReq as u64;

        // Allocate status byte
        let mut status: u8 = 0xFF;
        let status_addr = &mut status as *mut u8 as u64;

        // Build descriptor chain
        let buffers = if req_type == VIRTIO_BLK_T_OUT {
            // Write: header (read) -> data (read) -> status (write)
            vec![
                (req_header_addr, core::mem::size_of::<VirtioBlkReq>() as u32, false),
                (buffer.as_ptr() as u64, buffer.len() as u32, false),
                (status_addr, 1, true),
            ]
        } else {
            // Read: header (read) -> data (write) -> status (write)
            vec![
                (req_header_addr, core::mem::size_of::<VirtioBlkReq>() as u32, false),
                (buffer.as_mut_ptr() as u64, buffer.len() as u32, true),
                (status_addr, 1, true),
            ]
        };

        // Add to virtqueue
        {
            let mut queue = self.queue.lock();
            queue.add_buf(&buffers)?;

            // Notify device
            if queue.notify_needed() {
                self.transport.lock().write_reg(VirtIOMMIOOffset::QueueNotify, 0);
            }
        }

        // Wait for completion (bounded spin)
        let mut spins: usize = 0;
        loop {
            let mut queue = self.queue.lock();
            if let Some((desc_id, len)) = queue.get_used_buf() {
                // Check status
                if status != VIRTIO_BLK_S_OK {
                    crate::warn!("virtio-blk: I/O error (status={})", status);
                    return Err(Errno::EIO);
                }

                // For reads, return the length of data read
                // For writes, return the length of data written
                if req_type == VIRTIO_BLK_T_IN {
                    return Ok(buffer.len());
                } else {
                    return Ok(buffer.len());
                }
            }

            // Yield to avoid busy-waiting (Phase B: simple spin for now)
            core::hint::spin_loop();
            spins = spins.wrapping_add(1);
            if spins > 50_000_000 {
                crate::warn!("virtio-blk: request timed out (type={}, sector={})", req_type, sector);
                return Err(Errno::ETIMEDOUT);
            }
        }
    }

    /// Read block using zero-copy DMA (Phase 8)
    ///
    /// Returns a slice pointing directly to the DMA buffer (no copy).
    /// Caller must call `release_buffer()` when done with the data.
    ///
    /// # Arguments
    /// * `sector` - Starting sector number
    ///
    /// # Returns
    /// * `(buffer_index, data_slice)` - Buffer index and data slice
    pub fn read_block_zerocopy(&self, sector: u64) -> Result<(usize, &'static [u8])> {
        // Allocate DMA buffer
        let buf_idx = {
            let mut pool = self.dma_pool.lock();
            pool.allocate().ok_or(Errno::ENOSPC)?
        };

        let (physical_addr, virtual_addr) = {
            let pool = self.dma_pool.lock();
            let buf = pool.get(buf_idx).ok_or(Errno::EINVAL)?;
            (buf.physical_addr, buf.virtual_addr.as_ptr() as u64)
        };

        // Allocate request header
        let mut req_header = VirtioBlkReq {
            req_type: VIRTIO_BLK_T_IN,
            reserved: 0,
            sector,
        };
        let req_header_addr = &mut req_header as *mut VirtioBlkReq as u64;

        // Allocate status byte
        let mut status: u8 = 0xFF;
        let status_addr = &mut status as *mut u8 as u64;

        // Build descriptor chain: header (read) -> DMA buffer (write) -> status (write)
        let buffers = vec![
            (req_header_addr, core::mem::size_of::<VirtioBlkReq>() as u32, false),
            (physical_addr, DMA_BUFFER_SIZE as u32, true),
            (status_addr, 1, true),
        ];

        // Submit request
        {
            let mut queue = self.queue.lock();
            queue.add_buf(&buffers)?;

            // Notify device
            if queue.notify_needed() {
                self.transport.lock().write_reg(VirtIOMMIOOffset::QueueNotify, 0);
            }
        }

        // Wait for completion
        let mut spins: usize = 0;
        loop {
            let mut queue = self.queue.lock();
            if let Some((_desc_id, _len)) = queue.get_used_buf() {
                // Check status
                if status != VIRTIO_BLK_S_OK {
                    // Free buffer on error
                    let mut pool = self.dma_pool.lock();
                    pool.free(buf_idx);
                    crate::warn!("virtio-blk: zero-copy read error (status={})", status);
                    return Err(Errno::EIO);
                }

                // Return zero-copy slice (points directly to DMA buffer)
                let slice = unsafe {
                    core::slice::from_raw_parts(virtual_addr as *const u8, DMA_BUFFER_SIZE)
                };
                return Ok((buf_idx, slice));
            }

            core::hint::spin_loop();
            spins = spins.wrapping_add(1);
            if spins > 50_000_000 {
                // Free buffer on timeout
                let mut pool = self.dma_pool.lock();
                pool.free(buf_idx);
                crate::warn!("virtio-blk: zero-copy read timeout (sector={})", sector);
                return Err(Errno::ETIMEDOUT);
            }
        }
    }

    /// Release DMA buffer back to pool (Phase 8)
    ///
    /// Must be called after finishing with zero-copy data.
    ///
    /// # Arguments
    /// * `buf_idx` - Buffer index returned by `read_block_zerocopy()`
    pub fn release_buffer(&self, buf_idx: usize) {
        let mut pool = self.dma_pool.lock();
        pool.free(buf_idx);
    }

    /// Get DMA buffer pool statistics (Phase 8)
    ///
    /// Returns (total_buffers, free_buffers)
    pub fn get_dma_stats(&self) -> (usize, usize) {
        let pool = self.dma_pool.lock();
        pool.stats()
    }
}

/// BlockDeviceOps implementation for VirtioBlkDevice
struct VirtioBlkOps;

impl BlockDeviceOps for VirtioBlkOps {
    fn read_sectors(&self, dev: &BlockDevice, sector: u64, buf: &mut [u8]) -> Result<()> {
        // Get device from opaque pointer (we store Arc<VirtioBlkDevice> in the device)
        // For now, use a global registry approach
        let drv = get_virtio_blk_driver(&dev.name)?;
        drv.submit_request(VIRTIO_BLK_T_IN, sector, buf)?;
        Ok(())
    }

    fn write_sectors(&self, dev: &BlockDevice, sector: u64, buf: &[u8]) -> Result<()> {
        let drv = get_virtio_blk_driver(&dev.name)?;
        let mut buf_mut = buf.to_vec(); // Copy to mutable buffer
        drv.submit_request(VIRTIO_BLK_T_OUT, sector, &mut buf_mut)?;
        Ok(())
    }

    fn flush(&self, dev: &BlockDevice) -> Result<()> {
        let drv = get_virtio_blk_driver(&dev.name)?;
        let mut dummy = [0u8; 0];
        drv.submit_request(VIRTIO_BLK_T_FLUSH, 0, &mut dummy)?;
        Ok(())
    }
}

static VIRTIO_BLK_OPS: VirtioBlkOps = VirtioBlkOps;

/// Global registry of VirtIO block drivers
use alloc::collections::BTreeMap;
static VIRTIO_BLK_DRIVERS: Mutex<Option<BTreeMap<String, Arc<VirtioBlkDevice>>>> = Mutex::new(None);

fn get_virtio_blk_driver(name: &str) -> Result<Arc<VirtioBlkDevice>> {
    let drivers = VIRTIO_BLK_DRIVERS.lock();
    let drivers = drivers.as_ref().ok_or(Errno::ENODEV)?;
    drivers.get(name).cloned().ok_or(Errno::ENODEV)
}

/// Register and initialize a VirtIO block device
pub fn register_virtio_blk(transport: VirtIOMMIOTransport, name: String) -> Result<Arc<BlockDevice>> {
    // Initialize driver
    let driver = Arc::new(VirtioBlkDevice::new(transport, name.clone())?);

    // Register in global registry
    {
        let mut drivers = VIRTIO_BLK_DRIVERS.lock();
        if drivers.is_none() {
            *drivers = Some(BTreeMap::new());
        }
        drivers.as_mut().unwrap().insert(name.clone(), driver.clone());
    }

    // Create BlockDevice
    let block_dev = BlockDevice::new(
        name,
        8, // Major number for SCSI/virtio-blk
        0, // Minor number
        driver.capacity_sectors,
        &VIRTIO_BLK_OPS,
    );

    let dev = register_block_device(block_dev);

    // Probe for partitions
    crate::info!("virtio-blk: probing partitions on {}", dev.name);
    match crate::block::partition::register_partitions(&dev) {
        Ok(partitions) => {
            if !partitions.is_empty() {
                crate::info!("virtio-blk: found {} partition(s) on {}",
                           partitions.len(), dev.name);
            }
        }
        Err(e) => {
            crate::warn!("virtio-blk: partition probing failed: {:?}", e);
        }
    }

    Ok(dev)
}
