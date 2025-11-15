/// Camera Pixel Formats - Phase G.5
///
/// Pixel format definitions for camera frames

use alloc::vec::Vec;
use super::Resolution;

/// Pixel format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    /// RGB24 (8 bits per channel, 24 bits per pixel)
    RGB24,
    /// RGBA32 (8 bits per channel + alpha, 32 bits per pixel)
    RGBA32,
    /// BGR24 (8 bits per channel, 24 bits per pixel, reversed order)
    BGR24,
    /// YUYV (YUV 4:2:2, common for USB cameras)
    YUYV,
    /// MJPEG (Motion JPEG compressed)
    MJPEG,
    /// Grayscale (8 bits per pixel)
    Gray8,
}

impl PixelFormat {
    /// Get bytes per pixel
    pub fn bytes_per_pixel(&self) -> usize {
        match self {
            PixelFormat::RGB24 | PixelFormat::BGR24 => 3,
            PixelFormat::RGBA32 => 4,
            PixelFormat::YUYV => 2,
            PixelFormat::MJPEG => 0, // Variable (compressed)
            PixelFormat::Gray8 => 1,
        }
    }

    /// Check if format is compressed
    pub fn is_compressed(&self) -> bool {
        matches!(self, PixelFormat::MJPEG)
    }
}

/// Camera frame
pub struct Frame {
    pub data: Vec<u8>,
    pub resolution: Resolution,
    pub format: PixelFormat,
    pub timestamp: u64,  // Microseconds
    pub sequence: u32,   // Frame sequence number
}

impl Frame {
    /// Create a new frame
    pub fn new(
        data: Vec<u8>,
        resolution: Resolution,
        format: PixelFormat,
        timestamp: u64,
        sequence: u32,
    ) -> Self {
        Self {
            data,
            resolution,
            format,
            timestamp,
            sequence,
        }
    }

    /// Get frame size in bytes
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Check if frame is valid
    pub fn is_valid(&self) -> bool {
        if self.format.is_compressed() {
            !self.data.is_empty()
        } else {
            let expected_size =
                self.resolution.pixel_count() * self.format.bytes_per_pixel();
            self.data.len() == expected_size
        }
    }

    /// Convert to RGB24 format
    pub fn to_rgb24(&self) -> Option<Vec<u8>> {
        match self.format {
            PixelFormat::RGB24 => Some(self.data.clone()),

            PixelFormat::RGBA32 => {
                // Strip alpha channel
                let mut rgb = Vec::with_capacity(self.resolution.pixel_count() * 3);
                for chunk in self.data.chunks(4) {
                    if chunk.len() >= 3 {
                        rgb.push(chunk[0]);
                        rgb.push(chunk[1]);
                        rgb.push(chunk[2]);
                    }
                }
                Some(rgb)
            }

            PixelFormat::BGR24 => {
                // Swap R and B channels
                let mut rgb = Vec::with_capacity(self.data.len());
                for chunk in self.data.chunks(3) {
                    if chunk.len() == 3 {
                        rgb.push(chunk[2]); // R
                        rgb.push(chunk[1]); // G
                        rgb.push(chunk[0]); // B
                    }
                }
                Some(rgb)
            }

            PixelFormat::YUYV => {
                // Convert YUYV to RGB
                let mut rgb = Vec::with_capacity(self.resolution.pixel_count() * 3);

                for chunk in self.data.chunks(4) {
                    if chunk.len() < 4 {
                        continue;
                    }

                    let y0 = chunk[0] as i32;
                    let u = chunk[1] as i32 - 128;
                    let y1 = chunk[2] as i32;
                    let v = chunk[3] as i32 - 128;

                    // First pixel
                    let (r0, g0, b0) = yuv_to_rgb(y0, u, v);
                    rgb.push(r0);
                    rgb.push(g0);
                    rgb.push(b0);

                    // Second pixel
                    let (r1, g1, b1) = yuv_to_rgb(y1, u, v);
                    rgb.push(r1);
                    rgb.push(g1);
                    rgb.push(b1);
                }

                Some(rgb)
            }

            PixelFormat::Gray8 => {
                // Convert grayscale to RGB (replicate channel)
                let mut rgb = Vec::with_capacity(self.resolution.pixel_count() * 3);
                for &gray in &self.data {
                    rgb.push(gray);
                    rgb.push(gray);
                    rgb.push(gray);
                }
                Some(rgb)
            }

            PixelFormat::MJPEG => {
                // Would need JPEG decoder
                None
            }
        }
    }

    /// Convert to grayscale
    pub fn to_grayscale(&self) -> Option<Vec<u8>> {
        match self.format {
            PixelFormat::Gray8 => Some(self.data.clone()),

            PixelFormat::RGB24 => {
                let mut gray = Vec::with_capacity(self.resolution.pixel_count());
                for chunk in self.data.chunks(3) {
                    if chunk.len() == 3 {
                        // Weighted average for luminance
                        let luma = (0.299 * chunk[0] as f32
                            + 0.587 * chunk[1] as f32
                            + 0.114 * chunk[2] as f32) as u8;
                        gray.push(luma);
                    }
                }
                Some(gray)
            }

            PixelFormat::RGBA32 => {
                let mut gray = Vec::with_capacity(self.resolution.pixel_count());
                for chunk in self.data.chunks(4) {
                    if chunk.len() >= 3 {
                        let luma = (0.299 * chunk[0] as f32
                            + 0.587 * chunk[1] as f32
                            + 0.114 * chunk[2] as f32) as u8;
                        gray.push(luma);
                    }
                }
                Some(gray)
            }

            _ => None,
        }
    }

    /// Get pixel at (x, y) as RGB
    pub fn get_pixel_rgb(&self, x: u32, y: u32) -> Option<(u8, u8, u8)> {
        if x >= self.resolution.width || y >= self.resolution.height {
            return None;
        }

        let index = (y * self.resolution.width + x) as usize;

        match self.format {
            PixelFormat::RGB24 => {
                let offset = index * 3;
                if offset + 2 < self.data.len() {
                    Some((self.data[offset], self.data[offset + 1], self.data[offset + 2]))
                } else {
                    None
                }
            }

            PixelFormat::RGBA32 => {
                let offset = index * 4;
                if offset + 2 < self.data.len() {
                    Some((self.data[offset], self.data[offset + 1], self.data[offset + 2]))
                } else {
                    None
                }
            }

            PixelFormat::Gray8 => {
                if index < self.data.len() {
                    let gray = self.data[index];
                    Some((gray, gray, gray))
                } else {
                    None
                }
            }

            _ => None,
        }
    }
}

/// Convert YUV to RGB
fn yuv_to_rgb(y: i32, u: i32, v: i32) -> (u8, u8, u8) {
    let r = (y + ((1436 * v) >> 10)).clamp(0, 255) as u8;
    let g = (y - ((354 * u + 732 * v) >> 10)).clamp(0, 255) as u8;
    let b = (y + ((1814 * u) >> 10)).clamp(0, 255) as u8;
    (r, g, b)
}
