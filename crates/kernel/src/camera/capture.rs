/// Camera Capture Device - Phase G.5
///
/// USB camera (UVC) capture handling

use super::{Resolution, PixelFormat, Frame, CameraCapability};
use crate::lib::error::Result;
use alloc::string::String;
use alloc::vec::Vec;

/// Camera capture error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureError {
    NoDevice,
    NotStarted,
    InvalidFormat,
    DeviceError,
    BufferError,
}

/// Camera device
pub struct CameraDevice {
    device_name: String,
    device_id: u32,
    resolution: Resolution,
    pixel_format: PixelFormat,
    frame_rate: u32,
    capabilities: Vec<CameraCapability>,
    active: bool,
    frame_sequence: u32,
}

impl CameraDevice {
    /// Create a new camera device
    pub fn new(device_id: u32, device_name: String) -> Self {
        Self {
            device_name,
            device_id,
            resolution: Resolution::VGA,
            pixel_format: PixelFormat::YUYV,
            frame_rate: 30,
            capabilities: Vec::new(),
            active: false,
            frame_sequence: 0,
        }
    }

    /// Detect camera devices (USB Video Class - UVC)
    pub fn detect_devices() -> core::result::Result<Vec<Self>, CaptureError> {
        // In a real implementation, this would scan USB devices for UVC cameras
        crate::info!("camera::capture: scanning for UVC devices");

        #[cfg(not(feature = "hardware"))]
        {
            crate::warn!("camera::capture: hardware detection disabled, using mock device");
            let mut device = Self::new(0, String::from("Mock Webcam"));

            // Add mock capabilities
            device.capabilities.push(CameraCapability {
                resolution: Resolution::VGA,
                pixel_format: PixelFormat::YUYV,
                max_fps: 30,
            });
            device.capabilities.push(CameraCapability {
                resolution: Resolution::HD,
                pixel_format: PixelFormat::MJPEG,
                max_fps: 30,
            });

            Ok(alloc::vec![device])
        }

        #[cfg(feature = "hardware")]
        {
            // Real USB UVC device detection
            use crate::usb::{enumerate_devices, DeviceClass};

            let mut cameras = Vec::new();

            for usb_device in enumerate_devices() {
                if usb_device.class == DeviceClass::Video {
                    crate::info!("camera::capture: found UVC device: {}", usb_device.name);

                    let mut device = Self::new(usb_device.id, usb_device.name);

                    // Query capabilities from UVC descriptors
                    device.query_capabilities()?;

                    cameras.push(device);
                }
            }

            if cameras.is_empty() {
                Err(CaptureError::NoDevice)
            } else {
                Ok(cameras)
            }
        }
    }

    /// Query device capabilities
    fn query_capabilities(&mut self) -> core::result::Result<(), CaptureError> {
        // In a real implementation, this would:
        // 1. Send UVC GET_INFO requests
        // 2. Parse format descriptors
        // 3. Parse frame descriptors
        // 4. Build capability list

        Ok(())
    }

    /// Start camera capture
    pub fn start(&mut self) -> Result<()> {
        if self.active {
            return Ok(());
        }

        crate::info!(
            "camera::capture: starting capture on '{}' at {}x{} @ {} fps",
            self.device_name,
            self.resolution.width,
            self.resolution.height,
            self.frame_rate
        );

        // In a real implementation:
        // 1. Configure USB endpoints (isochronous or bulk)
        // 2. Set format (SET_CUR on VS_PROBE_CONTROL)
        // 3. Commit format (SET_CUR on VS_COMMIT_CONTROL)
        // 4. Start streaming (set alternate setting)
        // 5. Enable interrupt handler for frame data

        self.active = true;
        self.frame_sequence = 0;

        Ok(())
    }

    /// Stop camera capture
    pub fn stop(&mut self) -> Result<()> {
        if !self.active {
            return Ok(());
        }

        crate::info!("camera::capture: stopping capture on '{}'", self.device_name);

        // In a real implementation:
        // 1. Stop streaming (set alternate setting 0)
        // 2. Disable interrupt handler
        // 3. Flush buffers

        self.active = false;

        Ok(())
    }

    /// Capture a frame
    pub fn capture_frame(&mut self) -> Result<Frame> {
        if !self.active {
            return Err(crate::lib::error::Errno::EINVAL);
        }

        // In a real implementation, this would:
        // 1. Wait for frame buffer from interrupt handler
        // 2. Copy frame data
        // 3. Return frame with metadata

        // For now, generate a test pattern
        let frame = self.generate_test_frame();

        self.frame_sequence += 1;

        Ok(frame)
    }

    /// Generate a test frame (placeholder)
    fn generate_test_frame(&self) -> Frame {
        let pixel_count = self.resolution.pixel_count();
        let bytes_per_pixel = self.pixel_format.bytes_per_pixel();
        let size = pixel_count * bytes_per_pixel;

        let mut data = alloc::vec![0u8; size];

        // Generate a gradient pattern
        match self.pixel_format {
            PixelFormat::RGB24 | PixelFormat::BGR24 => {
                for y in 0..self.resolution.height {
                    for x in 0..self.resolution.width {
                        let index = ((y * self.resolution.width + x) * 3) as usize;
                        if index + 2 < data.len() {
                            data[index] = (x * 255 / self.resolution.width) as u8; // R
                            data[index + 1] = (y * 255 / self.resolution.height) as u8; // G
                            data[index + 2] = 128; // B
                        }
                    }
                }
            }

            PixelFormat::RGBA32 => {
                for y in 0..self.resolution.height {
                    for x in 0..self.resolution.width {
                        let index = ((y * self.resolution.width + x) * 4) as usize;
                        if index + 3 < data.len() {
                            data[index] = (x * 255 / self.resolution.width) as u8;
                            data[index + 1] = (y * 255 / self.resolution.height) as u8;
                            data[index + 2] = 128;
                            data[index + 3] = 255; // Alpha
                        }
                    }
                }
            }

            PixelFormat::Gray8 => {
                for y in 0..self.resolution.height {
                    for x in 0..self.resolution.width {
                        let index = (y * self.resolution.width + x) as usize;
                        if index < data.len() {
                            data[index] = ((x + y) * 255 / (self.resolution.width + self.resolution.height)) as u8;
                        }
                    }
                }
            }

            _ => {}
        }

        Frame::new(
            data,
            self.resolution,
            self.pixel_format,
            crate::time::get_uptime_ms() * 1000, // Convert to microseconds
            self.frame_sequence,
        )
    }

    /// Set capture resolution
    pub fn set_resolution(&mut self, resolution: Resolution) -> Result<()> {
        if self.active {
            return Err(crate::lib::error::Errno::EBUSY);
        }

        // Check if resolution is supported
        let supported = self
            .capabilities
            .iter()
            .any(|cap| cap.resolution == resolution);

        if !supported {
            return Err(crate::lib::error::Errno::EINVAL);
        }

        self.resolution = resolution;
        Ok(())
    }

    /// Set pixel format
    pub fn set_pixel_format(&mut self, format: PixelFormat) -> Result<()> {
        if self.active {
            return Err(crate::lib::error::Errno::EBUSY);
        }

        self.pixel_format = format;
        Ok(())
    }

    /// Set frame rate
    pub fn set_frame_rate(&mut self, fps: u32) -> Result<()> {
        if self.active {
            return Err(crate::lib::error::Errno::EBUSY);
        }

        self.frame_rate = fps;
        Ok(())
    }

    /// Get device name
    pub fn name(&self) -> &str {
        &self.device_name
    }

    /// Get device ID
    pub fn id(&self) -> u32 {
        self.device_id
    }

    /// Get current resolution
    pub fn resolution(&self) -> Resolution {
        self.resolution
    }

    /// Get current pixel format
    pub fn pixel_format(&self) -> PixelFormat {
        self.pixel_format
    }

    /// Get frame rate
    pub fn frame_rate(&self) -> u32 {
        self.frame_rate
    }

    /// Get capabilities
    pub fn capabilities(&self) -> &[CameraCapability] {
        &self.capabilities
    }

    /// Check if device is active
    pub fn is_active(&self) -> bool {
        self.active
    }
}

/// UVC interrupt handler (called when frame data arrives)
pub fn handle_uvc_interrupt(device_id: u32, frame_data: &[u8]) {
    // This would be called by the USB interrupt handler
    // when new frame data arrives from the device

    crate::trace!("camera::capture: received {} bytes from device {}", frame_data.len(), device_id);
}
