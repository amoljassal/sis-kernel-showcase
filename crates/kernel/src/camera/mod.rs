/// Camera Infrastructure - Phase G.5
///
/// Camera capture framework for computer vision integration

pub mod capture;
pub mod format;

pub use capture::{CameraDevice, CaptureError};
pub use format::{PixelFormat, Frame};

use spin::Mutex;
use alloc::sync::Arc;
use alloc::vec::Vec;
use crate::lib::error::Result;

/// Camera resolution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

impl Resolution {
    pub const VGA: Resolution = Resolution { width: 640, height: 480 };
    pub const HD: Resolution = Resolution { width: 1280, height: 720 };
    pub const FULL_HD: Resolution = Resolution { width: 1920, height: 1080 };

    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub fn pixel_count(&self) -> usize {
        (self.width * self.height) as usize
    }
}

/// Camera capability
#[derive(Debug, Clone)]
pub struct CameraCapability {
    pub resolution: Resolution,
    pub pixel_format: PixelFormat,
    pub max_fps: u32,
}

/// Camera manager
pub struct CameraManager {
    devices: Vec<Arc<Mutex<CameraDevice>>>,
    active_device: Option<Arc<Mutex<CameraDevice>>>,
}

impl CameraManager {
    /// Create a new camera manager
    pub fn new() -> Self {
        Self {
            devices: Vec::new(),
            active_device: None,
        }
    }

    /// Initialize camera devices
    pub fn init(&mut self) -> Result<()> {
        // Detect USB cameras (UVC - USB Video Class)
        match CameraDevice::detect_devices() {
            Ok(devices) => {
                crate::info!("camera: detected {} camera device(s)", devices.len());
                for device in devices {
                    let device_arc = Arc::new(Mutex::new(device));
                    self.devices.push(device_arc);
                }
            }
            Err(e) => {
                crate::warn!("camera: no devices found - {:?}", e);
            }
        }

        // Set first device as active if available
        if !self.devices.is_empty() {
            self.active_device = Some(self.devices[0].clone());
        }

        Ok(())
    }

    /// Get active camera device
    pub fn get_active_device(&self) -> Option<Arc<Mutex<CameraDevice>>> {
        self.active_device.clone()
    }

    /// Set active device by index
    pub fn set_active_device(&mut self, index: usize) -> Result<()> {
        if index < self.devices.len() {
            self.active_device = Some(self.devices[index].clone());
            Ok(())
        } else {
            Err(crate::lib::error::Errno::EINVAL)
        }
    }

    /// Get all devices
    pub fn get_devices(&self) -> &[Arc<Mutex<CameraDevice>>] {
        &self.devices
    }

    /// Start capture on active device
    pub fn start_capture(&mut self) -> Result<()> {
        if let Some(ref device) = self.active_device {
            device.lock().start()?;
            crate::info!("camera: started capture");
            Ok(())
        } else {
            Err(crate::lib::error::Errno::ENODEV)
        }
    }

    /// Stop capture on active device
    pub fn stop_capture(&mut self) -> Result<()> {
        if let Some(ref device) = self.active_device {
            device.lock().stop()?;
            crate::info!("camera: stopped capture");
            Ok(())
        } else {
            Err(crate::lib::error::Errno::ENODEV)
        }
    }

    /// Capture a frame from active device
    pub fn capture_frame(&self) -> Result<Frame> {
        if let Some(ref device) = self.active_device {
            device.lock().capture_frame()
        } else {
            Err(crate::lib::error::Errno::ENODEV)
        }
    }

    /// Check if any device is available
    pub fn has_device(&self) -> bool {
        !self.devices.is_empty()
    }

    /// Get number of devices
    pub fn device_count(&self) -> usize {
        self.devices.len()
    }
}

impl Default for CameraManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Global camera manager
static CAMERA_MANAGER: Mutex<Option<CameraManager>> = Mutex::new(None);

/// Initialize global camera manager
pub fn init() -> Result<()> {
    let mut manager = CameraManager::new();
    manager.init()?;
    *CAMERA_MANAGER.lock() = Some(manager);
    crate::info!("camera: camera subsystem initialized");
    Ok(())
}

/// Get global camera manager
pub fn get_manager() -> Option<&'static Mutex<Option<CameraManager>>> {
    Some(&CAMERA_MANAGER)
}

/// Start camera capture
pub fn start_capture() -> Result<()> {
    if let Some(ref mut manager) = *CAMERA_MANAGER.lock() {
        manager.start_capture()
    } else {
        Err(crate::lib::error::Errno::ENODEV)
    }
}

/// Stop camera capture
pub fn stop_capture() -> Result<()> {
    if let Some(ref mut manager) = *CAMERA_MANAGER.lock() {
        manager.stop_capture()
    } else {
        Err(crate::lib::error::Errno::ENODEV)
    }
}

/// Capture a frame
pub fn capture_frame() -> Result<Frame> {
    if let Some(ref manager) = *CAMERA_MANAGER.lock() {
        manager.capture_frame()
    } else {
        Err(crate::lib::error::Errno::ENODEV)
    }
}

/// Integration points for future computer vision:
///
/// 1. Object Detection:
///    ```rust
///    pub fn detect_objects(frame: &Frame) -> Vec<Detection> {
///        // Run YOLOv5 or similar model
///        // Return bounding boxes and classes
///    }
///    ```
///
/// 2. Face Detection:
///    ```rust
///    pub fn detect_faces(frame: &Frame) -> Vec<FaceRect> {
///        // Run face detection model
///        // Return face locations
///    }
///    ```
///
/// 3. Optical Character Recognition:
///    ```rust
///    pub fn recognize_text(frame: &Frame) -> Vec<TextBlock> {
///        // Run OCR model (Tesseract or similar)
///        // Return recognized text and locations
///    }
///    ```
///
/// 4. Scene Understanding:
///    ```rust
///    pub fn understand_scene(frame: &Frame) -> SceneDescription {
///        // Run multimodal vision-language model
///        // Return semantic description of scene
///    }
///    ```
///
/// 5. Vision Pipeline:
///    ```
///    Camera → Frame Capture → Preprocessing → Vision Model →
///    JARVIS Context → Voice/Visual Response
///    ```
