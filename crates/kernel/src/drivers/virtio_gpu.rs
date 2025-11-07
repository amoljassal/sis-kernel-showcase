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

        // Negotiate features (disable VIRGL 3D, enable EDID)
        {
            let t = transport.lock();

            // Read device features
            t.write_reg(VirtIOMMIOOffset::DeviceFeaturesSel, 0);
            let device_features = t.read_reg(VirtIOMMIOOffset::DeviceFeatures);

            // We want EDID but not VIRGL
            let mut driver_features = 0u32;
            if device_features & VIRTIO_GPU_F_EDID != 0 {
                driver_features |= VIRTIO_GPU_F_EDID;
            }

            // Write driver features
            t.write_reg(VirtIOMMIOOffset::DriverFeaturesSel, 0);
            t.write_reg(VirtIOMMIOOffset::DriverFeatures, driver_features);

            // Features OK
            let status = t.read_reg(VirtIOMMIOOffset::Status) | VirtIOStatus::FeaturesOK as u32;
            t.write_reg(VirtIOMMIOOffset::Status, status);

            // Verify features accepted
            if t.read_reg(VirtIOMMIOOffset::Status) & VirtIOStatus::FeaturesOK as u32 == 0 {
                return Err(Errno::EIO);
            }
        }

        // Create virtqueues (control queue 0, cursor queue 1)
        let control_queue = {
            let _t = transport.lock();
            VirtQueue::new(0, 64).map_err(|_| Errno::ENOMEM)?
        };

        let cursor_queue = {
            let t = transport.lock();
            VirtQueue::new(1, 16).map_err(|_| Errno::ENOMEM)?
        };

        // Allocate framebuffer memory (BGRA8888 format, 4 bytes per pixel)
        let width = DEFAULT_WIDTH;
        let height = DEFAULT_HEIGHT;
        let framebuffer_size = (width * height * 4) as usize;

        // Allocate physically contiguous memory for framebuffer
        let framebuffer_phys = crate::mm::alloc_phys_pages(
            (framebuffer_size + 4095) / 4096
        ).ok_or(Errno::ENOMEM)? as PhysAddr;

        let framebuffer = crate::mm::phys_to_virt(framebuffer_phys) as *mut u32;

        // Zero out framebuffer
        unsafe {
            ptr::write_bytes(framebuffer, 0, (width * height) as usize);
        }

        let control_queue = Arc::new(Mutex::new(control_queue));
        let cursor_queue = Arc::new(Mutex::new(cursor_queue));

        // Driver OK
        {
            let t = transport.lock();
            let status = t.read_reg(VirtIOMMIOOffset::Status) | VirtIOStatus::DriverOK as u32;
            t.write_reg(VirtIOMMIOOffset::Status, status);
        }

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
            gpu.init_display()?;
        }

        crate::info!("virtio-gpu: initialized {}x{} framebuffer", width, height);

        Ok(gpu_arc)
    }

    /// Initialize display by creating resource and setting scanout
    fn init_display(&mut self) -> Result<()> {
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
            format: VIRTIO_GPU_FORMAT_B8G8R8A8_UNORM,  // BGRA8888
            width,
            height,
        };

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
        let cmd_bytes = unsafe {
            core::slice::from_raw_parts(
                cmd as *const T as *const u8,
                core::mem::size_of::<T>()
            )
        };

        // Allocate response buffer
        let mut response = VirtioGpuCtrlHdr {
            cmd_type: 0,
            flags: 0,
            fence_id: 0,
            ctx_id: 0,
            padding: 0,
        };

        let response_bytes = unsafe {
            core::slice::from_raw_parts_mut(
                &mut response as *mut VirtioGpuCtrlHdr as *mut u8,
                core::mem::size_of::<VirtioGpuCtrlHdr>()
            )
        };

        // Submit to control queue
        let mut queue = self.control_queue.lock();
        queue.add_chain(&[cmd_bytes], &[response_bytes])
            .map_err(|_| Errno::EIO)?;

        // Notify device
        let transport = self.transport.lock();
        transport.write_reg(VirtIOMMIOOffset::QueueNotify, 0);
        drop(transport);

        // Wait for response
        queue.wait_for_used().map_err(|_| Errno::EIO)?;

        // Check response
        if response.cmd_type != VIRTIO_GPU_RESP_OK_NODATA {
            return Err(Errno::EIO);
        }

        Ok(())
    }

    /// Submit command with additional data
    fn submit_command_with_data<T, D>(&mut self, cmd: &T, data: &D) -> Result<()> {
        let cmd_bytes = unsafe {
            core::slice::from_raw_parts(
                cmd as *const T as *const u8,
                core::mem::size_of::<T>()
            )
        };

        let data_bytes = unsafe {
            core::slice::from_raw_parts(
                data as *const D as *const u8,
                core::mem::size_of::<D>()
            )
        };

        // Allocate response buffer
        let mut response = VirtioGpuCtrlHdr {
            cmd_type: 0,
            flags: 0,
            fence_id: 0,
            ctx_id: 0,
            padding: 0,
        };

        let response_bytes = unsafe {
            core::slice::from_raw_parts_mut(
                &mut response as *mut VirtioGpuCtrlHdr as *mut u8,
                core::mem::size_of::<VirtioGpuCtrlHdr>()
            )
        };

        // Submit to control queue
        let mut queue = self.control_queue.lock();
        queue.add_chain(&[cmd_bytes, data_bytes], &[response_bytes])
            .map_err(|_| Errno::EIO)?;

        // Notify device
        let transport = self.transport.lock();
        transport.write_reg(VirtIOMMIOOffset::QueueNotify, 0);
        drop(transport);

        // Wait for response
        queue.wait_for_used().map_err(|_| Errno::EIO)?;

        // Check response
        if response.cmd_type != VIRTIO_GPU_RESP_OK_NODATA {
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
