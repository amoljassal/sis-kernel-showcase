/// VirtIO GPU Device Driver - Phase G.0
///
/// Implements virtio-gpu device (Device ID 16) for 2D graphics
/// Provides framebuffer access and basic 2D rendering capabilities

use crate::lib::error::{Result, Errno};
use crate::virtio::{VirtIOMMIOTransport, VirtIOMMIOOffset, VirtIOStatus};
use crate::virtio::virtqueue::VirtQueue;
use crate::mm::PhysAddr;
use alloc::sync::Arc;
// use alloc::vec::Vec;
// use alloc::boxed::Box;
use spin::Mutex;
use core::ptr;

/// VirtIO GPU feature bits
const VIRTIO_GPU_F_VIRGL: u32 = 1 << 0;       // 3D rendering (we don't need this)
const VIRTIO_GPU_F_EDID: u32 = 1 << 1;        // EDID support for resolution detection

/// VirtIO GPU command types
const VIRTIO_GPU_CMD_GET_DISPLAY_INFO: u32 = 0x0100;
const VIRTIO_GPU_CMD_RESOURCE_CREATE_2D: u32 = 0x0101;
const VIRTIO_GPU_CMD_RESOURCE_UNREF: u32 = 0x0102;
const VIRTIO_GPU_CMD_SET_SCANOUT: u32 = 0x0103;
const VIRTIO_GPU_CMD_RESOURCE_FLUSH: u32 = 0x0104;
const VIRTIO_GPU_CMD_TRANSFER_TO_HOST_2D: u32 = 0x0105;
const VIRTIO_GPU_CMD_RESOURCE_ATTACH_BACKING: u32 = 0x0106;
const VIRTIO_GPU_CMD_RESOURCE_DETACH_BACKING: u32 = 0x0107;
const VIRTIO_GPU_CMD_GET_CAPSET_INFO: u32 = 0x0108;
const VIRTIO_GPU_CMD_GET_CAPSET: u32 = 0x0109;
const VIRTIO_GPU_CMD_GET_EDID: u32 = 0x010a;

/// VirtIO GPU response types
const VIRTIO_GPU_RESP_OK_NODATA: u32 = 0x1100;
const VIRTIO_GPU_RESP_OK_DISPLAY_INFO: u32 = 0x1101;
const VIRTIO_GPU_RESP_OK_CAPSET_INFO: u32 = 0x1102;
const VIRTIO_GPU_RESP_OK_CAPSET: u32 = 0x1103;
const VIRTIO_GPU_RESP_OK_EDID: u32 = 0x1104;

/// VirtIO GPU error codes
const VIRTIO_GPU_RESP_ERR_UNSPEC: u32 = 0x1200;
const VIRTIO_GPU_RESP_ERR_OUT_OF_MEMORY: u32 = 0x1201;
const VIRTIO_GPU_RESP_ERR_INVALID_SCANOUT_ID: u32 = 0x1202;
const VIRTIO_GPU_RESP_ERR_INVALID_RESOURCE_ID: u32 = 0x1203;
const VIRTIO_GPU_RESP_ERR_INVALID_CONTEXT_ID: u32 = 0x1204;
const VIRTIO_GPU_RESP_ERR_INVALID_PARAMETER: u32 = 0x1205;

/// VirtIO GPU pixel formats
const VIRTIO_GPU_FORMAT_B8G8R8A8_UNORM: u32 = 1;   // BGRA8888
const VIRTIO_GPU_FORMAT_B8G8R8X8_UNORM: u32 = 2;   // BGRX8888
const VIRTIO_GPU_FORMAT_A8R8G8B8_UNORM: u32 = 3;   // ARGB8888
const VIRTIO_GPU_FORMAT_X8R8G8B8_UNORM: u32 = 4;   // XRGB8888
const VIRTIO_GPU_FORMAT_R8G8B8A8_UNORM: u32 = 67;  // RGBA8888
const VIRTIO_GPU_FORMAT_X8B8G8R8_UNORM: u32 = 68;  // XBGR8888
const VIRTIO_GPU_FORMAT_A8B8G8R8_UNORM: u32 = 121; // ABGR8888
const VIRTIO_GPU_FORMAT_R8G8B8X8_UNORM: u32 = 134; // RGBX8888

/// Default framebuffer resolution
const DEFAULT_WIDTH: u32 = 1280;
const DEFAULT_HEIGHT: u32 = 720;

/// Maximum scanouts (displays)
const VIRTIO_GPU_MAX_SCANOUTS: usize = 16;

/// VirtIO GPU control header
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct VirtioGpuCtrlHdr {
    cmd_type: u32,
    flags: u32,
    fence_id: u64,
    ctx_id: u32,
    padding: u32,
}

/// VirtIO GPU rectangle
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VirtioGpuRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// VirtIO GPU display information
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct VirtioGpuDisplayOne {
    r: VirtioGpuRect,
    enabled: u32,
    flags: u32,
}

/// VirtIO GPU display info response
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct VirtioGpuRespDisplayInfo {
    hdr: VirtioGpuCtrlHdr,
    pmodes: [VirtioGpuDisplayOne; VIRTIO_GPU_MAX_SCANOUTS],
}

/// VirtIO GPU 2D resource creation
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct VirtioGpuResourceCreate2D {
    hdr: VirtioGpuCtrlHdr,
    resource_id: u32,
    format: u32,
    width: u32,
    height: u32,
}

/// VirtIO GPU resource attach backing
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct VirtioGpuResourceAttachBacking {
    hdr: VirtioGpuCtrlHdr,
    resource_id: u32,
    nr_entries: u32,
}

/// VirtIO GPU memory entry
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct VirtioGpuMemEntry {
    addr: u64,
    length: u32,
    padding: u32,
}

/// VirtIO GPU set scanout
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct VirtioGpuSetScanout {
    hdr: VirtioGpuCtrlHdr,
    r: VirtioGpuRect,
    scanout_id: u32,
    resource_id: u32,
}

/// VirtIO GPU resource flush
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct VirtioGpuResourceFlush {
    hdr: VirtioGpuCtrlHdr,
    r: VirtioGpuRect,
    resource_id: u32,
    padding: u32,
}

/// VirtIO GPU transfer to host 2D
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct VirtioGpuTransferToHost2D {
    hdr: VirtioGpuCtrlHdr,
    r: VirtioGpuRect,
    offset: u64,
    resource_id: u32,
    padding: u32,
}

/// VirtIO GPU device configuration
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VirtioGpuConfig {
    pub events_read: u32,
    pub events_clear: u32,
    pub num_scanouts: u32,
    pub num_capsets: u32,
}

/// VirtIO GPU device
pub struct VirtioGpu {
    transport: Arc<Mutex<VirtIOMMIOTransport>>,
    control_queue: Arc<Mutex<VirtQueue>>,
    cursor_queue: Arc<Mutex<VirtQueue>>,
    framebuffer: *mut u32,
    framebuffer_phys: PhysAddr,
    framebuffer_size: usize,
    resolution: (u32, u32),
    scanout_id: u32,
    resource_id: u32,
    next_resource_id: u32,
}

unsafe impl Send for VirtioGpu {}
unsafe impl Sync for VirtioGpu {}

impl VirtioGpu {
    /// Create and initialize a new VirtIO GPU device
    pub fn new(transport: VirtIOMMIOTransport) -> Result<Arc<Mutex<Self>>> {
        let transport = Arc::new(Mutex::new(transport));

        crate::info!("virtio-gpu: initializing device");
        // Reset device
        {
            let t = transport.lock();
            t.reset_device().map_err(|_| Errno::EIO)?;

            // Acknowledge device
            t.write_reg(VirtIOMMIOOffset::Status, VirtIOStatus::Acknowledge as u32);

            // Driver loaded
            let status = t.read_reg(VirtIOMMIOOffset::Status) | VirtIOStatus::Driver as u32;
            t.write_reg(VirtIOMMIOOffset::Status, status);
        }

        // Negotiate features (disable VIRGL 3D, enable EDID, ack VERSION_1 if offered)
        {
            let t = transport.lock();

            // Read device features
            t.write_reg(VirtIOMMIOOffset::DeviceFeaturesSel, 0);
            let device_features = t.read_reg(VirtIOMMIOOffset::DeviceFeatures);
            crate::info!("virtio-gpu: device features=0x{:08x}", device_features);
            t.write_reg(VirtIOMMIOOffset::DeviceFeaturesSel, 1);
            let device_features_hi = t.read_reg(VirtIOMMIOOffset::DeviceFeatures);
            crate::info!("virtio-gpu: device features[63:32]=0x{:08x}", device_features_hi);

            // We want EDID but not VIRGL
            let mut driver_features = 0u32;
            if device_features & VIRTIO_GPU_F_EDID != 0 {
                driver_features |= VIRTIO_GPU_F_EDID;
            }

            // Write driver features
            t.write_reg(VirtIOMMIOOffset::DriverFeaturesSel, 0);
            t.write_reg(VirtIOMMIOOffset::DriverFeatures, driver_features);
            crate::info!("virtio-gpu: driver features=0x{:08x}", driver_features);

            // High feature word: ack VERSION_1 if offered
            let mut driver_features_hi = 0u32;
            // VIRTIO_F_VERSION_1 is bit 32 overall -> bit 0 in the high 32-bit word
            if (device_features_hi & 0x1) != 0 { driver_features_hi |= 0x1; }
            t.write_reg(VirtIOMMIOOffset::DriverFeaturesSel, 1);
            t.write_reg(VirtIOMMIOOffset::DriverFeatures, driver_features_hi);
            crate::info!("virtio-gpu: driver features[63:32]=0x{:08x}", driver_features_hi);

            // Features OK
            let status = t.read_reg(VirtIOMMIOOffset::Status) | VirtIOStatus::FeaturesOK as u32;
            t.write_reg(VirtIOMMIOOffset::Status, status);

            // Verify features accepted
            if t.read_reg(VirtIOMMIOOffset::Status) & VirtIOStatus::FeaturesOK as u32 == 0 {
                return Err(Errno::EIO);
            }
        }

        // Create virtqueues (control queue 0, cursor queue 1) and program MMIO
        // Read QueueNumMax to choose a valid size.
        let mmio_ver = transport.lock().version();
        let use_legacy = mmio_ver == 1;
        if use_legacy { crate::info!("virtio-gpu: MMIO version=1 (legacy path)"); }

        let (mut control_queue, mut cursor_queue) = {
            let t = transport.lock();
            // Control queue
            t.write_reg(VirtIOMMIOOffset::QueueSel, 0);
            let cq_max = t.read_reg(VirtIOMMIOOffset::QueueNumMax);
            if cq_max == 0 { crate::warn!("virtio-gpu: control queue not available"); return Err(Errno::EIO); }
            // Prefer full size on GPU
            let cq_size = cq_max as u16;
            crate::info!("virtio-gpu: control queue max={}, using {}", cq_max, cq_size);
            drop(t);
            let cq = if use_legacy { VirtQueue::new_contiguous(0, cq_size) } else { VirtQueue::new(0, cq_size) };
            let cq = cq.map_err(|_| Errno::ENOMEM)?;

            let t = transport.lock();
            // Cursor queue
            t.write_reg(VirtIOMMIOOffset::QueueSel, 1);
            let kq_max = t.read_reg(VirtIOMMIOOffset::QueueNumMax);
            if kq_max == 0 { crate::warn!("virtio-gpu: cursor queue not available"); return Err(Errno::EIO); }
            let kq_size = core::cmp::min(256u16, kq_max as u16);
            crate::info!("virtio-gpu: cursor queue max={}, using {}", kq_max, kq_size);
            drop(t);
            let kq = if use_legacy { VirtQueue::new_contiguous(1, kq_size) } else { VirtQueue::new(1, kq_size) };
            let kq = kq.map_err(|_| Errno::ENOMEM)?;
            (cq, kq)
        };

        {
            let mut t = transport.lock();
            if use_legacy {
                t.setup_queue_legacy(&control_queue).map_err(|_| Errno::EIO)?;
                t.setup_queue_legacy(&cursor_queue).map_err(|_| Errno::EIO)?;
            } else {
                t.setup_queue(&mut control_queue).map_err(|_| Errno::EIO)?;
                t.setup_queue(&mut cursor_queue).map_err(|_| Errno::EIO)?;
            }
        }

        // Allocate framebuffer memory (BGRA8888 format, 4 bytes per pixel)
        let width = DEFAULT_WIDTH;
        let height = DEFAULT_HEIGHT;
        let framebuffer_size = (width * height * 4) as usize;

        // Allocate physically contiguous memory for framebuffer
        let framebuffer_phys = crate::mm::alloc_phys_pages(
            (framebuffer_size + 4095) / 4096
        ).ok_or(Errno::ENOMEM)? as PhysAddr;

        // Use identity-mapped VA for the framebuffer. Our bring-up page tables
        // currently map RAM with an identity mapping (low VA == PA). Using
        // phys_to_virt() would add KERNEL_BASE and point at an unmapped VA.
        let framebuffer = framebuffer_phys as usize as *mut u32;
        
        // Zero out framebuffer
        unsafe {
            ptr::write_bytes(framebuffer, 0, (width * height) as usize);
        }

        let control_queue = Arc::new(Mutex::new(control_queue));
        let cursor_queue = Arc::new(Mutex::new(cursor_queue));

        // Driver OK
        { transport.lock().driver_ready(); }
        crate::info!("virtio-gpu: queues ready, DRIVER_OK set");

        let gpu = Self {
            transport,
            control_queue,
            cursor_queue,
            framebuffer,
            framebuffer_phys,
            framebuffer_size,
            resolution: (width, height),
            scanout_id: 0,
            resource_id: 1,
            next_resource_id: 2,
        };

        // Initialize GPU: create resource, attach backing, set scanout
        let gpu_arc = Arc::new(Mutex::new(gpu));

        {
            let mut gpu = gpu_arc.lock();
            crate::info!("virtio-gpu: creating resource and setting scanout");
            gpu.init_display()?;
        }

        crate::info!("virtio-gpu: initialized {}x{} framebuffer", width, height);

        Ok(gpu_arc)
    }

    /// Initialize display by creating resource and setting scanout
    fn init_display(&mut self) -> Result<()> {
        // Probe display info first to validate control queue
        if let Ok(di) = self.get_display_info() {
            let ns = di.pmodes[0];
            crate::info!(
                "virtio-gpu: display0 enabled={} rect=({}, {}) {}x{}",
                ns.enabled, ns.r.x, ns.r.y, ns.r.width, ns.r.height
            );
        } else {
            crate::warn!("virtio-gpu: GET_DISPLAY_INFO failed; continuing");
        }

        // Create 2D resource
        self.create_2d_resource(self.resource_id, self.resolution.0, self.resolution.1)?;

        // Attach backing store
        self.attach_backing(self.resource_id)?;

        // Set scanout
        self.set_scanout(self.scanout_id, self.resource_id)?;

        Ok(())
    }

    /// Create a 2D resource
    fn create_2d_resource(&mut self, resource_id: u32, width: u32, height: u32) -> Result<()> {
        let cmd = VirtioGpuResourceCreate2D {
            hdr: VirtioGpuCtrlHdr {
                cmd_type: VIRTIO_GPU_CMD_RESOURCE_CREATE_2D,
                flags: 0,
                fence_id: 0,
                ctx_id: 0,
                padding: 0,
            },
            resource_id,
            // XRGB8888 is commonly supported for scanout
            format: VIRTIO_GPU_FORMAT_X8R8G8B8_UNORM,
            width,
            height,
        };

        crate::debug!("virtio-gpu: RESOURCE_CREATE_2D id={} {}x{}", resource_id, width, height);
        self.submit_command(&cmd)?;
        Ok(())
    }

    /// Attach backing store to resource
    fn attach_backing(&mut self, resource_id: u32) -> Result<()> {
        let cmd = VirtioGpuResourceAttachBacking {
            hdr: VirtioGpuCtrlHdr {
                cmd_type: VIRTIO_GPU_CMD_RESOURCE_ATTACH_BACKING,
                flags: 0,
                fence_id: 0,
                ctx_id: 0,
                padding: 0,
            },
            resource_id,
            nr_entries: 1,
        };

        let entry = VirtioGpuMemEntry {
            addr: self.framebuffer_phys as u64,
            length: self.framebuffer_size as u32,
            padding: 0,
        };

        crate::debug!(
            "virtio-gpu: ATTACH_BACKING id={} addr=0x{:x} len={}",
            resource_id,
            self.framebuffer_phys,
            self.framebuffer_size
        );
        self.submit_command_with_data(&cmd, &entry)?;
        Ok(())
    }

    /// Set scanout (map resource to display)
    fn set_scanout(&mut self, scanout_id: u32, resource_id: u32) -> Result<()> {
        let cmd = VirtioGpuSetScanout {
            hdr: VirtioGpuCtrlHdr {
                cmd_type: VIRTIO_GPU_CMD_SET_SCANOUT,
                flags: 0,
                fence_id: 0,
                ctx_id: 0,
                padding: 0,
            },
            r: VirtioGpuRect {
                x: 0,
                y: 0,
                width: self.resolution.0,
                height: self.resolution.1,
            },
            scanout_id,
            resource_id,
        };

        self.submit_command(&cmd)?;
        Ok(())
    }

    /// Flush framebuffer region to display
    pub fn flush(&mut self, x: u32, y: u32, w: u32, h: u32) -> Result<()> {
        // First, transfer to host
        let transfer_cmd = VirtioGpuTransferToHost2D {
            hdr: VirtioGpuCtrlHdr {
                cmd_type: VIRTIO_GPU_CMD_TRANSFER_TO_HOST_2D,
                flags: 0,
                fence_id: 0,
                ctx_id: 0,
                padding: 0,
            },
            r: VirtioGpuRect { x, y, width: w, height: h },
            offset: 0,
            resource_id: self.resource_id,
            padding: 0,
        };

        self.submit_command(&transfer_cmd)?;

        // Then, flush to display
        let flush_cmd = VirtioGpuResourceFlush {
            hdr: VirtioGpuCtrlHdr {
                cmd_type: VIRTIO_GPU_CMD_RESOURCE_FLUSH,
                flags: 0,
                fence_id: 0,
                ctx_id: 0,
                padding: 0,
            },
            r: VirtioGpuRect { x, y, width: w, height: h },
            resource_id: self.resource_id,
            padding: 0,
        };

        self.submit_command(&flush_cmd)?;
        Ok(())
    }

    /// Get framebuffer as mutable slice
    pub fn get_framebuffer(&self) -> &mut [u32] {
        let pixel_count = (self.resolution.0 * self.resolution.1) as usize;
        unsafe { core::slice::from_raw_parts_mut(self.framebuffer, pixel_count) }
    }

    /// Get framebuffer resolution
    pub fn resolution(&self) -> (u32, u32) {
        self.resolution
    }

    /// Submit command without additional data
    fn submit_command<T>(&mut self, cmd: &T) -> Result<()> {
        // Allocate DMA-friendly memory for cmd and response
        let cmd_len = core::mem::size_of::<T>();
        let resp_len = core::mem::size_of::<VirtioGpuCtrlHdr>();
        let cmd_pa = crate::mm::alloc_phys_pages((cmd_len + 4095) / 4096).ok_or(Errno::ENOMEM)?;
        let resp_pa = crate::mm::alloc_phys_pages((resp_len + 4095) / 4096).ok_or(Errno::ENOMEM)?;
        let cmd_va = cmd_pa as usize as *mut u8;
        let resp_va = resp_pa as usize as *mut u8;

        unsafe {
            core::ptr::copy_nonoverlapping(cmd as *const T as *const u8, cmd_va, cmd_len);
            core::ptr::write_bytes(resp_va, 0, resp_len);
        }

        let cmd_bytes = unsafe { core::slice::from_raw_parts(cmd_va as *const u8, cmd_len) };
        let response_bytes = unsafe { core::slice::from_raw_parts_mut(resp_va, resp_len) };

        // Submit to control queue
        let mut queue = self.control_queue.lock();
        match queue.add_chain(&[cmd_bytes], &[response_bytes]) {
            Ok(_) => {}
            Err(e) => { crate::warn!("virtio-gpu: add_chain failed: {:?}", e); return Err(Errno::EIO); }
        }

        // Notify device
        let transport = self.transport.lock();
        transport.write_reg(VirtIOMMIOOffset::QueueNotify, 0);
        drop(transport);

        // Wait for response
        if let Err(e) = queue.wait_for_used() {
            crate::warn!("virtio-gpu: wait_for_used timeout: {:?}", e);
            // Free DMA buffers
            crate::mm::free_pages(cmd_pa, 0);
            crate::mm::free_pages(resp_pa, 0);
            return Err(Errno::EIO);
        }

        // Check response
        let mut hdr = VirtioGpuCtrlHdr { cmd_type: 0, flags: 0, fence_id: 0, ctx_id: 0, padding: 0 };
        unsafe { core::ptr::copy_nonoverlapping(resp_va as *const u8, &mut hdr as *mut _ as *mut u8, resp_len); }
        // Free DMA buffers
        crate::mm::free_pages(cmd_pa, 0);
        crate::mm::free_pages(resp_pa, 0);

        if hdr.cmd_type != VIRTIO_GPU_RESP_OK_NODATA {
            crate::warn!("virtio-gpu: unexpected resp=0x{:x}", hdr.cmd_type);
            return Err(Errno::EIO);
        }

        Ok(())
    }

    /// Submit command with typed response buffer
    fn submit_command_with_resp<T, R>(&mut self, cmd: &T, resp: &mut R) -> Result<()> {
        // Use DMA-friendly pages for both cmd and response
        let cmd_len = core::mem::size_of::<T>();
        let resp_len = core::mem::size_of::<R>();
        let cmd_pa = crate::mm::alloc_phys_pages((cmd_len + 4095) / 4096).ok_or(Errno::ENOMEM)?;
        let resp_pa = crate::mm::alloc_phys_pages((resp_len + 4095) / 4096).ok_or(Errno::ENOMEM)?;
        let cmd_va = cmd_pa as usize as *mut u8;
        let resp_va = resp_pa as usize as *mut u8;

        unsafe {
            core::ptr::copy_nonoverlapping(cmd as *const T as *const u8, cmd_va, cmd_len);
            core::ptr::write_bytes(resp_va, 0, resp_len);
        }

        let cmd_bytes = unsafe { core::slice::from_raw_parts(cmd_va as *const u8, cmd_len) };
        let resp_bytes = unsafe { core::slice::from_raw_parts_mut(resp_va, resp_len) };

        let mut queue = self.control_queue.lock();
        match queue.add_chain(&[cmd_bytes], &[resp_bytes]) {
            Ok(_) => {}
            Err(e) => { crate::warn!("virtio-gpu: add_chain(resp) failed: {:?}", e); return Err(Errno::EIO); }
        }
        let transport = self.transport.lock();
        transport.write_reg(VirtIOMMIOOffset::QueueNotify, 0);
        drop(transport);
        if let Err(e) = queue.wait_for_used() {
            crate::warn!("virtio-gpu: wait_for_used (resp) timeout: {:?}", e);
            crate::mm::free_pages(cmd_pa, 0);
            crate::mm::free_pages(resp_pa, 0);
            return Err(Errno::EIO);
        }

        // Copy back response and free
        unsafe {
            core::ptr::copy_nonoverlapping(resp_va as *const u8, resp as *mut R as *mut u8, resp_len);
        }
        crate::mm::free_pages(cmd_pa, 0);
        crate::mm::free_pages(resp_pa, 0);
        Ok(())
    }

    /// Get display information from the device
    fn get_display_info(&mut self) -> Result<VirtioGpuRespDisplayInfo> {
        let cmd = VirtioGpuCtrlHdr {
            cmd_type: VIRTIO_GPU_CMD_GET_DISPLAY_INFO,
            flags: 0,
            fence_id: 0,
            ctx_id: 0,
            padding: 0,
        };
        let mut resp: VirtioGpuRespDisplayInfo = VirtioGpuRespDisplayInfo {
            hdr: VirtioGpuCtrlHdr { cmd_type: 0, flags: 0, fence_id: 0, ctx_id: 0, padding: 0 },
            pmodes: [VirtioGpuDisplayOne { r: VirtioGpuRect { x:0, y:0, width:0, height:0 }, enabled: 0, flags: 0}; VIRTIO_GPU_MAX_SCANOUTS],
        };
        self.submit_command_with_resp(&cmd, &mut resp)?;
        if resp.hdr.cmd_type != VIRTIO_GPU_RESP_OK_DISPLAY_INFO {
            crate::warn!("virtio-gpu: GET_DISPLAY_INFO unexpected resp=0x{:x}", resp.hdr.cmd_type);
            return Err(Errno::EIO);
        }
        Ok(resp)
    }

    /// Submit command with additional data
    fn submit_command_with_data<T, D>(&mut self, cmd: &T, data: &D) -> Result<()> {
        let cmd_len = core::mem::size_of::<T>();
        let data_len = core::mem::size_of::<D>();
        let resp_len = core::mem::size_of::<VirtioGpuCtrlHdr>();

        let cmd_pa = crate::mm::alloc_phys_pages((cmd_len + 4095) / 4096).ok_or(Errno::ENOMEM)?;
        let data_pa = crate::mm::alloc_phys_pages((data_len + 4095) / 4096).ok_or(Errno::ENOMEM)?;
        let resp_pa = crate::mm::alloc_phys_pages((resp_len + 4095) / 4096).ok_or(Errno::ENOMEM)?;

        let cmd_va = cmd_pa as usize as *mut u8;
        let data_va = data_pa as usize as *mut u8;
        let resp_va = resp_pa as usize as *mut u8;

        unsafe {
            core::ptr::copy_nonoverlapping(cmd as *const T as *const u8, cmd_va, cmd_len);
            core::ptr::copy_nonoverlapping(data as *const D as *const u8, data_va, data_len);
            core::ptr::write_bytes(resp_va, 0, resp_len);
        }

        let cmd_bytes = unsafe { core::slice::from_raw_parts(cmd_va as *const u8, cmd_len) };
        let data_bytes = unsafe { core::slice::from_raw_parts(data_va as *const u8, data_len) };
        let response_bytes = unsafe { core::slice::from_raw_parts_mut(resp_va, resp_len) };

        // Submit to control queue
        let mut queue = self.control_queue.lock();
        match queue.add_chain(&[cmd_bytes, data_bytes], &[response_bytes]) {
            Ok(_) => {}
            Err(e) => {
                crate::mm::free_pages(cmd_pa, 0);
                crate::mm::free_pages(data_pa, 0);
                crate::mm::free_pages(resp_pa, 0);
                crate::warn!("virtio-gpu: add_chain(data) failed: {:?}", e); return Err(Errno::EIO);
            }
        }

        // Notify device
        let transport = self.transport.lock();
        transport.write_reg(VirtIOMMIOOffset::QueueNotify, 0);
        drop(transport);

        // Wait for response
        if let Err(e) = queue.wait_for_used() {
            crate::warn!("virtio-gpu: wait_for_used (data) timeout: {:?}", e);
            crate::mm::free_pages(cmd_pa, 0);
            crate::mm::free_pages(data_pa, 0);
            crate::mm::free_pages(resp_pa, 0);
            return Err(Errno::EIO);
        }

        // Check response and free
        let mut hdr = VirtioGpuCtrlHdr { cmd_type: 0, flags: 0, fence_id: 0, ctx_id: 0, padding: 0 };
        unsafe { core::ptr::copy_nonoverlapping(resp_va as *const u8, &mut hdr as *mut _ as *mut u8, resp_len); }
        crate::mm::free_pages(cmd_pa, 0);
        crate::mm::free_pages(data_pa, 0);
        crate::mm::free_pages(resp_pa, 0);
        if hdr.cmd_type != VIRTIO_GPU_RESP_OK_NODATA {
            crate::warn!("virtio-gpu: unexpected resp(data)=0x{:x}", hdr.cmd_type);
            return Err(Errno::EIO);
        }

        Ok(())
    }

    /// Allocate new resource ID
    fn next_resource_id(&mut self) -> u32 {
        let id = self.next_resource_id;
        self.next_resource_id += 1;
        id
    }
}

/// Global GPU device instance
static GPU_DEVICE: Mutex<Option<Arc<Mutex<VirtioGpu>>>> = Mutex::new(None);

/// Initialize virtio-gpu device
pub fn init(transport: VirtIOMMIOTransport) -> Result<()> {
    let gpu = VirtioGpu::new(transport)?;
    *GPU_DEVICE.lock() = Some(gpu);
    Ok(())
}

/// Get global GPU device
pub fn get_gpu() -> Option<Arc<Mutex<VirtioGpu>>> {
    GPU_DEVICE.lock().clone()
}
