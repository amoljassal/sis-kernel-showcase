/// Audio Output Device - Phase G.5
///
/// Speaker/audio output handling

use super::{RingBuffer, SAMPLE_RATE, CHANNELS, BUFFER_SIZE};
use crate::lib::error::Result;
use alloc::string::String;

/// Audio output error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputError {
    NoDevice,
    NotStarted,
    BufferFull,
    DeviceError,
}

/// Audio output device
pub struct AudioOutputDevice {
    device_name: String,
    device_id: u32,
    buffer: RingBuffer<i16, 16384>,  // ~1 second at 16kHz
    sample_rate: u32,
    channels: u8,
    active: bool,
    volume: u8,  // 0-100
}

impl AudioOutputDevice {
    /// Create a new audio output device
    pub fn new(device_id: u32, device_name: String) -> Self {
        Self {
            device_name,
            device_id,
            buffer: RingBuffer::new(),
            sample_rate: SAMPLE_RATE,
            channels: CHANNELS,
            active: false,
            volume: 80,  // Default 80% volume
        }
    }

    /// Detect audio output device
    pub fn detect() -> core::result::Result<Self, OutputError> {
        // Try to detect audio output device (virtio-snd or USB audio)
        // In a real implementation, this would scan for audio devices

        crate::info!("audio::output: scanning for audio output devices");

        // Try virtio-snd first
        #[cfg(feature = "virtio-snd")]
        {
            if let Ok(device) = Self::detect_virtio_snd() {
                return Ok(device);
            }
        }

        // Try USB audio output
        #[cfg(feature = "hardware")]
        {
            if let Ok(device) = Self::detect_usb_audio() {
                return Ok(device);
            }
        }

        // Mock device for testing
        crate::warn!("audio::output: no hardware device found, using mock device");
        Ok(Self::new(0, String::from("Mock Speaker")))
    }

    #[cfg(feature = "virtio-snd")]
    fn detect_virtio_snd() -> core::result::Result<Self, OutputError> {
        // Scan virtio devices for sound device
        use crate::drivers::virtio::scan_virtio_devices;

        for device in scan_virtio_devices() {
            if device.device_type == VIRTIO_ID_SOUND {
                crate::info!("audio::output: found virtio-snd device");
                return Ok(Self::new(device.id, String::from("VirtIO Sound")));
            }
        }

        Err(OutputError::NoDevice)
    }

    #[cfg(feature = "hardware")]
    fn detect_usb_audio() -> core::result::Result<Self, OutputError> {
        // Scan USB devices for audio output
        use crate::usb::{enumerate_devices, DeviceClass};

        for device in enumerate_devices() {
            if device.class == DeviceClass::Audio && device.is_output {
                crate::info!("audio::output: found USB audio device: {}", device.name);
                return Ok(Self::new(device.id, device.name));
            }
        }

        Err(OutputError::NoDevice)
    }

    /// Start audio playback
    pub fn start(&mut self) -> Result<()> {
        if self.active {
            return Ok(());
        }

        crate::info!("audio::output: starting playback on '{}'", self.device_name);

        // In a real implementation:
        // 1. Configure audio device (sample rate, format, channels)
        // 2. Set up DMA for audio streaming
        // 3. Enable interrupt handler for buffer refill requests
        // 4. Start audio stream

        self.active = true;
        self.buffer.clear();

        Ok(())
    }

    /// Stop audio playback
    pub fn stop(&mut self) -> Result<()> {
        if !self.active {
            return Ok(());
        }

        crate::info!("audio::output: stopping playback on '{}'", self.device_name);

        // In a real implementation:
        // 1. Stop DMA transfers
        // 2. Disable interrupt handler
        // 3. Flush buffers
        // 4. Stop audio stream

        self.active = false;

        Ok(())
    }

    /// Write audio samples for playback
    pub fn write(&mut self, samples: &[i16]) -> Result<usize> {
        if !self.active {
            // Auto-start if not active
            self.start()?;
        }

        // Apply volume control
        let mut scaled_samples = alloc::vec::Vec::with_capacity(samples.len());
        let volume_scale = self.volume as f32 / 100.0;

        for &sample in samples {
            let scaled = (sample as f32 * volume_scale) as i16;
            scaled_samples.push(scaled);
        }

        let written = self.buffer.write_slice(&scaled_samples);

        // In a real implementation, if buffer is full, we might:
        // - Block until space is available (blocking mode)
        // - Return immediately with partial write (non-blocking mode)
        // - Drop oldest samples (real-time mode)

        Ok(written)
    }

    /// Read samples from internal buffer (called by playback interrupt)
    pub fn read_from_buffer(&mut self, output: &mut [i16]) -> usize {
        self.buffer.read_slice(output)
    }

    /// Get number of samples buffered
    pub fn buffered(&self) -> usize {
        self.buffer.len()
    }

    /// Check if device is active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Get device name
    pub fn name(&self) -> &str {
        &self.device_name
    }

    /// Get device ID
    pub fn id(&self) -> u32 {
        self.device_id
    }

    /// Set volume (0-100)
    pub fn set_volume(&mut self, volume: u8) {
        self.volume = volume.min(100);
        crate::info!("audio::output: volume set to {}%", self.volume);
    }

    /// Get current volume
    pub fn volume(&self) -> u8 {
        self.volume
    }

    /// Get sample rate
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Get number of channels
    pub fn channels(&self) -> u8 {
        self.channels
    }

    /// Clear internal buffer
    pub fn clear_buffer(&mut self) {
        self.buffer.clear();
    }
}

/// Audio output interrupt handler (called when device needs more samples)
pub fn handle_audio_output_interrupt(device_id: u32, buffer: &mut [i16]) {
    // This would be called by the interrupt handler when the device
    // needs more samples to play

    // Find the device and read samples from its buffer
    // In a real implementation, we'd have a registry of active devices

    crate::trace!("audio::output: device {} requesting {} samples", device_id, buffer.len());
}

/// Constants for virtio-snd
#[cfg(feature = "virtio-snd")]
const VIRTIO_ID_SOUND: u32 = 25;
