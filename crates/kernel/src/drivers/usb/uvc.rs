//! USB Video Class (UVC) Driver
//!
//! Implements UVC 1.5 specification for USB webcams and cameras.
//! Supports MJPEG, YUYV, and uncompressed video formats.
//!
//! # UVC Architecture
//!
//! ```text
//! UVC Device
//!   ├─> Video Control Interface (interface 0)
//!   │     ├─> Input Terminal (camera sensor)
//!   │     ├─> Processing Unit (brightness, contrast, etc.)
//!   │     └─> Output Terminal (USB streaming)
//!   └─> Video Streaming Interface (interface 1+)
//!         ├─> Format descriptors (MJPEG, YUYV, etc.)
//!         ├─> Frame descriptors (resolutions, frame rates)
//!         └─> Isochronous/Bulk endpoints
//! ```

use super::{UsbDevice, DeviceClass};
use super::descriptor::{DescriptorIterator, InterfaceDescriptor, EndpointDescriptor};
use crate::drivers::{DriverError, DriverResult};
use crate::camera::{Resolution, PixelFormat, Frame, CameraCapability};
use alloc::vec::Vec;
use alloc::string::String;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use spin::Mutex;

/// UVC Class-Specific Descriptor Types
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UvcDescriptorType {
    Undefined = 0x00,
    Device = 0x01,
    Configuration = 0x02,
    String = 0x03,
    Interface = 0x04,
    Endpoint = 0x05,
    /// Video Control Interface Header (CS_INTERFACE)
    VcHeader = 0x24,
    /// Video Streaming Interface Header (CS_INTERFACE, same value, different context)
    VsHeader = 0x25,
}

/// UVC Video Control Descriptor Subtypes
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum VcSubtype {
    Undefined = 0x00,
    Header = 0x01,
    InputTerminal = 0x02,
    OutputTerminal = 0x03,
    SelectorUnit = 0x04,
    ProcessingUnit = 0x05,
    ExtensionUnit = 0x06,
}

/// UVC Video Streaming Descriptor Subtypes
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum VsSubtype {
    Undefined = 0x00,
    InputHeader = 0x01,
    OutputHeader = 0x02,
    StillImageFrame = 0x03,
    FormatUncompressed = 0x04,
    FrameUncompressed = 0x05,
    FormatMjpeg = 0x06,
    FrameMjpeg = 0x07,
    FormatMpeg2ts = 0x0A,
    FormatDv = 0x0C,
    Colorformat = 0x0D,
    FormatFrameBased = 0x10,
    FrameFrameBased = 0x11,
    FormatStreamBased = 0x12,
}

/// UVC Video Control Requests
#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum VideoControlRequest {
    SetCur = 0x01,
    GetCur = 0x81,
    GetMin = 0x82,
    GetMax = 0x83,
    GetRes = 0x84,
    GetLen = 0x85,
    GetInfo = 0x86,
    GetDef = 0x87,
}

/// UVC Video Streaming Control
#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum VideoStreamingControl {
    ProbeControl = 0x01,
    CommitControl = 0x02,
    StillProbeControl = 0x03,
    StillCommitControl = 0x04,
    StillImageTriggerControl = 0x05,
    StreamErrorCodeControl = 0x06,
    GenerateKeyFrameControl = 0x07,
    UpdateFrameSegmentControl = 0x08,
    SynchDelayControl = 0x09,
}

/// UVC Probe/Commit Control (48 bytes for UVC 1.5)
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct VideoProbeCommitControl {
    pub bmhint: u16,
    pub format_index: u8,
    pub frame_index: u8,
    pub frame_interval: u32,
    pub key_frame_rate: u16,
    pub pframe_rate: u16,
    pub comp_quality: u16,
    pub comp_window_size: u16,
    pub delay: u16,
    pub max_video_frame_size: u32,
    pub max_payload_transfer_size: u32,
    pub clock_frequency: u32,
    pub framing_info: u8,
    pub preferred_version: u8,
    pub min_version: u8,
    pub max_version: u8,
    pub usage: u8,
    pub bit_depth_luma: u8,
    pub settings: u8,
    pub max_number_of_ref_frames_plus_1: u8,
    pub rate_control_modes: u16,
    pub layout_per_stream: u64,
}

impl VideoProbeCommitControl {
    pub fn new() -> Self {
        Self {
            bmhint: 0x0001,  // dwFrameInterval
            format_index: 1,
            frame_index: 1,
            frame_interval: 333333,  // 30 fps (100ns units)
            key_frame_rate: 0,
            pframe_rate: 0,
            comp_quality: 0,
            comp_window_size: 0,
            delay: 0,
            max_video_frame_size: 0,
            max_payload_transfer_size: 0,
            clock_frequency: 0,
            framing_info: 0,
            preferred_version: 0,
            min_version: 0,
            max_version: 0,
            usage: 0,
            bit_depth_luma: 0,
            settings: 0,
            max_number_of_ref_frames_plus_1: 0,
            rate_control_modes: 0,
            layout_per_stream: 0,
        }
    }

    /// Convert frame interval to FPS
    pub fn get_fps(&self) -> u32 {
        if self.frame_interval > 0 {
            10_000_000 / self.frame_interval
        } else {
            0
        }
    }

    /// Set FPS (converts to frame interval)
    pub fn set_fps(&mut self, fps: u32) {
        if fps > 0 {
            self.frame_interval = 10_000_000 / fps;
        }
    }
}

impl Default for VideoProbeCommitControl {
    fn default() -> Self {
        Self::new()
    }
}

/// UVC Format Descriptor
#[derive(Debug, Clone)]
pub struct UvcFormat {
    pub format_index: u8,
    pub pixel_format: PixelFormat,
    pub frames: Vec<UvcFrame>,
}

/// UVC Frame Descriptor
#[derive(Debug, Clone)]
pub struct UvcFrame {
    pub frame_index: u8,
    pub resolution: Resolution,
    pub min_interval: u32,  // 100ns units
    pub max_interval: u32,
    pub default_interval: u32,
    pub intervals: Vec<u32>,
}

impl UvcFrame {
    /// Get supported frame rates
    pub fn get_frame_rates(&self) -> Vec<u32> {
        if self.intervals.is_empty() {
            // Continuous frame rate
            let min_fps = 10_000_000 / self.max_interval;
            let max_fps = 10_000_000 / self.min_interval;
            alloc::vec![min_fps, 30.min(max_fps), max_fps]
        } else {
            // Discrete frame rates
            self.intervals.iter().map(|&interval| {
                10_000_000 / interval
            }).collect()
        }
    }
}

/// UVC Camera Device
pub struct UvcCamera {
    /// USB device information
    usb_device: UsbDevice,

    /// Video control interface number
    control_interface: u8,

    /// Video streaming interface number
    streaming_interface: u8,

    /// Supported formats
    formats: Vec<UvcFormat>,

    /// Current format index
    current_format: u8,

    /// Current frame index
    current_frame: u8,

    /// Current probe/commit control
    probe_commit: VideoProbeCommitControl,

    /// Camera is streaming
    streaming: AtomicBool,

    /// Frame sequence number
    frame_sequence: AtomicU32,

    /// Frame buffer
    frame_buffer: Mutex<Vec<u8>>,
}

impl UvcCamera {
    /// Create new UVC camera from USB device
    pub fn new(usb_device: UsbDevice) -> DriverResult<Self> {
        let mut camera = Self {
            usb_device,
            control_interface: 0,
            streaming_interface: 1,
            formats: Vec::new(),
            current_format: 1,
            current_frame: 1,
            probe_commit: VideoProbeCommitControl::new(),
            streaming: AtomicBool::new(false),
            frame_sequence: AtomicU32::new(0),
            frame_buffer: Mutex::new(Vec::new()),
        };

        // Parse UVC descriptors
        camera.parse_descriptors()?;

        crate::info!(
            "[UVC] Initialized camera: {} ({}x{} @ {}fps)",
            camera.usb_device.name,
            camera.get_current_resolution().width,
            camera.get_current_resolution().height,
            camera.probe_commit.get_fps()
        );

        Ok(camera)
    }

    /// Parse UVC descriptors
    fn parse_descriptors(&mut self) -> DriverResult<()> {
        // In a real implementation, this would parse the USB configuration
        // descriptor to extract UVC-specific information including:
        // - Video Control Interface descriptors
        // - Video Streaming Interface descriptors
        // - Format and Frame descriptors
        // - Endpoint information

        // For now, add mock formats
        self.formats.push(UvcFormat {
            format_index: 1,
            pixel_format: PixelFormat::YUYV,
            frames: alloc::vec![
                UvcFrame {
                    frame_index: 1,
                    resolution: Resolution::VGA,
                    min_interval: 333333,  // 30 fps
                    max_interval: 1000000, // 10 fps
                    default_interval: 333333,
                    intervals: alloc::vec![333333, 400000, 500000],
                },
                UvcFrame {
                    frame_index: 2,
                    resolution: Resolution::HD,
                    min_interval: 333333,
                    max_interval: 1000000,
                    default_interval: 333333,
                    intervals: alloc::vec![333333, 666666],
                },
            ],
        });

        self.formats.push(UvcFormat {
            format_index: 2,
            pixel_format: PixelFormat::MJPEG,
            frames: alloc::vec![
                UvcFrame {
                    frame_index: 1,
                    resolution: Resolution::VGA,
                    min_interval: 333333,
                    max_interval: 1000000,
                    default_interval: 333333,
                    intervals: alloc::vec![333333],
                },
                UvcFrame {
                    frame_index: 2,
                    resolution: Resolution::HD,
                    min_interval: 333333,
                    max_interval: 1000000,
                    default_interval: 333333,
                    intervals: alloc::vec![333333],
                },
                UvcFrame {
                    frame_index: 3,
                    resolution: Resolution::FULL_HD,
                    min_interval: 333333,
                    max_interval: 1000000,
                    default_interval: 333333,
                    intervals: alloc::vec![333333],
                },
            ],
        });

        Ok(())
    }

    /// Get device name
    pub fn name(&self) -> &str {
        &self.usb_device.name
    }

    /// Get device ID
    pub fn device_id(&self) -> u8 {
        self.usb_device.id
    }

    /// Get current resolution
    pub fn get_current_resolution(&self) -> Resolution {
        for format in &self.formats {
            if format.format_index == self.current_format {
                for frame in &format.frames {
                    if frame.frame_index == self.current_frame {
                        return frame.resolution;
                    }
                }
            }
        }
        Resolution::VGA
    }

    /// Get current pixel format
    pub fn get_current_format(&self) -> PixelFormat {
        for format in &self.formats {
            if format.format_index == self.current_format {
                return format.pixel_format;
            }
        }
        PixelFormat::YUYV
    }

    /// Get camera capabilities
    pub fn get_capabilities(&self) -> Vec<CameraCapability> {
        let mut caps = Vec::new();

        for format in &self.formats {
            for frame in &format.frames {
                let frame_rates = frame.get_frame_rates();
                let max_fps = *frame_rates.iter().max().unwrap_or(&30);

                caps.push(CameraCapability {
                    resolution: frame.resolution,
                    pixel_format: format.pixel_format,
                    max_fps,
                });
            }
        }

        caps
    }

    /// Set video format
    pub fn set_format(&mut self, resolution: Resolution, pixel_format: PixelFormat, fps: u32) -> DriverResult<()> {
        // Find matching format and frame
        for format in &self.formats {
            if format.pixel_format != pixel_format {
                continue;
            }

            for frame in &format.frames {
                if frame.resolution == resolution {
                    self.current_format = format.format_index;
                    self.current_frame = frame.frame_index;
                    self.probe_commit.format_index = format.format_index;
                    self.probe_commit.frame_index = frame.frame_index;
                    self.probe_commit.set_fps(fps);

                    crate::info!(
                        "[UVC] Set format: {}x{} {:?} @ {}fps",
                        resolution.width,
                        resolution.height,
                        pixel_format,
                        fps
                    );

                    return Ok(());
                }
            }
        }

        Err(DriverError::NotSupported)
    }

    /// Negotiate streaming parameters with camera
    fn negotiate_streaming(&mut self) -> DriverResult<()> {
        // Step 1: PROBE - get camera's suggested parameters
        // This would involve USB control transfers to:
        // GET_CUR with VideoStreamingControl::ProbeControl

        // Step 2: Optionally adjust parameters and SET_CUR

        // Step 3: COMMIT the final parameters
        // SET_CUR with VideoStreamingControl::CommitControl

        crate::debug!("[UVC] Negotiated streaming parameters");
        Ok(())
    }

    /// Start video streaming
    pub fn start_streaming(&mut self) -> DriverResult<()> {
        if self.streaming.load(Ordering::Acquire) {
            return Ok(()); // Already streaming
        }

        // Negotiate streaming parameters
        self.negotiate_streaming()?;

        // Select alternate setting for streaming interface
        // (alternate setting 0 = zero bandwidth, >0 = streaming)

        // Start isochronous/bulk transfers

        self.streaming.store(true, Ordering::Release);
        crate::info!("[UVC] Started streaming");

        Ok(())
    }

    /// Stop video streaming
    pub fn stop_streaming(&mut self) -> DriverResult<()> {
        if !self.streaming.load(Ordering::Acquire) {
            return Ok(()); // Not streaming
        }

        // Select alternate setting 0 (zero bandwidth)

        // Stop transfers

        self.streaming.store(false, Ordering::Release);
        crate::info!("[UVC] Stopped streaming");

        Ok(())
    }

    /// Capture a single frame
    pub fn capture_frame(&mut self) -> DriverResult<Frame> {
        if !self.streaming.load(Ordering::Acquire) {
            return Err(DriverError::InvalidState);
        }

        let sequence = self.frame_sequence.fetch_add(1, Ordering::SeqCst);
        let resolution = self.get_current_resolution();
        let pixel_format = self.get_current_format();

        // In a real implementation, this would:
        // 1. Wait for a complete frame from isochronous/bulk transfers
        // 2. Assemble frame from multiple USB packets
        // 3. Handle UVC payload headers (FID, EOF markers)
        // 4. Decompress MJPEG if needed

        // For now, return a mock frame
        let frame_size = match pixel_format {
            PixelFormat::YUYV => resolution.pixel_count() * 2,
            PixelFormat::MJPEG => resolution.pixel_count() / 10, // Compressed ~10:1
            _ => resolution.pixel_count() * 3,
        };

        Ok(Frame {
            data: alloc::vec![0u8; frame_size],
            resolution,
            format: pixel_format,
            timestamp: crate::time::get_timestamp_us(),
            sequence,
        })
    }

    /// Check if streaming
    pub fn is_streaming(&self) -> bool {
        self.streaming.load(Ordering::Acquire)
    }
}

/// Detect all UVC cameras on USB bus
pub fn detect_uvc_cameras() -> Vec<UvcCamera> {
    let mut cameras = Vec::new();

    // Get all USB devices
    let usb_devices = crate::drivers::usb::enumerate_devices();

    for device in usb_devices {
        // Check if device is a video class device
        if device.class == DeviceClass::Video {
            match UvcCamera::new(device) {
                Ok(camera) => {
                    crate::info!("[UVC] Detected camera: {}", camera.name());
                    cameras.push(camera);
                }
                Err(e) => {
                    crate::warn!("[UVC] Failed to initialize camera: {:?}", e);
                }
            }
        }
    }

    cameras
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probe_commit_fps() {
        let mut ctrl = VideoProbeCommitControl::new();
        ctrl.set_fps(30);
        assert_eq!(ctrl.get_fps(), 30);

        ctrl.set_fps(60);
        assert_eq!(ctrl.get_fps(), 60);
    }

    #[test]
    fn test_probe_commit_size() {
        assert_eq!(core::mem::size_of::<VideoProbeCommitControl>(), 48);
    }
}
