//! USB Audio Class (UAC) Driver
//!
//! Implements USB Audio Class 1.0 and 2.0 specifications for USB audio devices
//! including microphones, speakers, headsets, and audio interfaces.
//!
//! # UAC Architecture
//!
//! ```text
//! UAC Device
//!   ├─> Audio Control Interface (interface 0)
//!   │     ├─> Input Terminal (microphone, line-in)
//!   │     ├─> Output Terminal (speaker, headphone)
//!   │     ├─> Feature Unit (volume, mute, etc.)
//!   │     └─> Mixer Unit (audio mixing)
//!   └─> Audio Streaming Interface (interface 1+)
//!         ├─> Format descriptors (PCM, AC3, etc.)
//!         ├─> Sampling rates
//!         └─> Isochronous endpoints
//! ```

use super::{UsbDevice, DeviceClass};
use crate::drivers::{DriverError, DriverResult};
use alloc::vec::Vec;
use alloc::string::String;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use spin::Mutex;

/// USB Audio Class Specification Versions
#[repr(u16)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UacVersion {
    /// Audio Class 1.0
    Uac1 = 0x0100,
    /// Audio Class 2.0
    Uac2 = 0x0200,
}

/// Audio Control Descriptor Subtypes
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AcSubtype {
    Undefined = 0x00,
    Header = 0x01,
    InputTerminal = 0x02,
    OutputTerminal = 0x03,
    MixerUnit = 0x04,
    SelectorUnit = 0x05,
    FeatureUnit = 0x06,
    ProcessingUnit = 0x07,
    ExtensionUnit = 0x08,
}

/// Audio Streaming Descriptor Subtypes
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AsSubtype {
    Undefined = 0x00,
    General = 0x01,
    FormatType = 0x02,
    FormatSpecific = 0x03,
}

/// Audio Terminal Types
#[repr(u16)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TerminalType {
    UsbStreaming = 0x0101,
    Microphone = 0x0201,
    DesktopMicrophone = 0x0202,
    PersonalMicrophone = 0x0203,
    OmnidirectionalMicrophone = 0x0204,
    MicrophoneArray = 0x0205,
    Speaker = 0x0301,
    Headphones = 0x0302,
    Headset = 0x0303,
    SpeakerPhone = 0x0304,
    LineConnector = 0x0603,
    DigitalAudioInterface = 0x0604,
}

/// Audio Format Types
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FormatType {
    TypeI = 0x01,      // PCM, PCM8, IEE_FLOAT, ALAW, MULAW
    TypeII = 0x02,     // MPEG, AC-3
    TypeIII = 0x03,    // IEC1937 AC-3, IEC1937 MPEG-1, IEC1937 MPEG-2
}

/// Audio Data Formats
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AudioFormat {
    Pcm = 0x0001,
    Pcm8 = 0x0002,
    IeeeFloat = 0x0003,
    ALaw = 0x0004,
    MuLaw = 0x0005,
    Mpeg = 0x1001,
    Ac3 = 0x1002,
}

/// Audio Sample Rates
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SampleRate {
    /// 8 kHz - Telephone quality
    Rate8000 = 8000,
    /// 11.025 kHz - Low quality
    Rate11025 = 11025,
    /// 16 kHz - Wideband speech
    Rate16000 = 16000,
    /// 22.05 kHz - FM radio quality
    Rate22050 = 22050,
    /// 32 kHz - Miniature quality
    Rate32000 = 32000,
    /// 44.1 kHz - CD quality
    Rate44100 = 44100,
    /// 48 kHz - Professional audio
    Rate48000 = 48000,
    /// 88.2 kHz - High quality
    Rate88200 = 88200,
    /// 96 kHz - Studio quality
    Rate96000 = 96000,
    /// 176.4 kHz - Very high quality
    Rate176400 = 176400,
    /// 192 kHz - Ultra high quality
    Rate192000 = 192000,
}

impl SampleRate {
    pub fn from_hz(hz: u32) -> Option<Self> {
        match hz {
            8000 => Some(Self::Rate8000),
            11025 => Some(Self::Rate11025),
            16000 => Some(Self::Rate16000),
            22050 => Some(Self::Rate22050),
            32000 => Some(Self::Rate32000),
            44100 => Some(Self::Rate44100),
            48000 => Some(Self::Rate48000),
            88200 => Some(Self::Rate88200),
            96000 => Some(Self::Rate96000),
            176400 => Some(Self::Rate176400),
            192000 => Some(Self::Rate192000),
            _ => None,
        }
    }

    pub fn as_hz(&self) -> u32 {
        *self as u32
    }
}

/// Audio Stream Configuration
#[derive(Debug, Clone, Copy)]
pub struct AudioStreamConfig {
    pub format: AudioFormat,
    pub sample_rate: SampleRate,
    pub channels: u8,
    pub bit_depth: u8,
}

impl AudioStreamConfig {
    pub fn new(sample_rate: SampleRate, channels: u8, bit_depth: u8) -> Self {
        Self {
            format: AudioFormat::Pcm,
            sample_rate,
            channels,
            bit_depth,
        }
    }

    /// Get bytes per sample
    pub fn bytes_per_sample(&self) -> usize {
        (self.bit_depth as usize + 7) / 8
    }

    /// Get bytes per frame
    pub fn bytes_per_frame(&self) -> usize {
        self.channels as usize * self.bytes_per_sample()
    }

    /// Get bytes per second
    pub fn bytes_per_second(&self) -> usize {
        self.sample_rate.as_hz() as usize * self.bytes_per_frame()
    }
}

/// Audio Feature Unit Controls
#[derive(Debug, Clone, Copy)]
pub struct FeatureControls {
    pub mute: bool,
    pub volume: i16,      // In 1/256 dB units
    pub bass: i16,
    pub mid: i16,
    pub treble: i16,
    pub auto_gain: bool,
}

impl FeatureControls {
    pub fn new() -> Self {
        Self {
            mute: false,
            volume: 0,      // 0 dB
            bass: 0,
            mid: 0,
            treble: 0,
            auto_gain: false,
        }
    }

    /// Set volume in decibels (0 = 0 dB, -100 = -100 dB, etc.)
    pub fn set_volume_db(&mut self, db: i16) {
        self.volume = db * 256;
    }

    /// Get volume in decibels
    pub fn get_volume_db(&self) -> i16 {
        self.volume / 256
    }

    /// Set volume as percentage (0-100)
    pub fn set_volume_percent(&mut self, percent: u8) {
        // Assuming -80 dB to 0 dB range
        let percent = percent.min(100);
        let db = -80 + (percent as i16 * 80 / 100);
        self.set_volume_db(db);
    }

    /// Get volume as percentage (0-100)
    pub fn get_volume_percent(&self) -> u8 {
        let db = self.get_volume_db();
        let percent = ((db + 80) * 100 / 80).max(0).min(100);
        percent as u8
    }
}

impl Default for FeatureControls {
    fn default() -> Self {
        Self::new()
    }
}

/// USB Audio Device
pub struct UsbAudioDevice {
    /// USB device information
    usb_device: UsbDevice,

    /// Audio control interface
    control_interface: u8,

    /// Audio streaming interface
    streaming_interface: u8,

    /// Device is input (microphone) or output (speaker)
    is_input: bool,

    /// UAC version
    uac_version: UacVersion,

    /// Current stream configuration
    config: AudioStreamConfig,

    /// Feature controls
    controls: Mutex<FeatureControls>,

    /// Device is streaming
    streaming: AtomicBool,

    /// Sample counter
    sample_count: AtomicU32,

    /// Audio buffer
    audio_buffer: Mutex<Vec<u8>>,
}

impl UsbAudioDevice {
    /// Create new USB audio device
    pub fn new(usb_device: UsbDevice, is_input: bool) -> DriverResult<Self> {
        let device = Self {
            usb_device,
            control_interface: 0,
            streaming_interface: 1,
            is_input,
            uac_version: UacVersion::Uac1,
            config: AudioStreamConfig::new(SampleRate::Rate48000, 2, 16),
            controls: Mutex::new(FeatureControls::new()),
            streaming: AtomicBool::new(false),
            sample_count: AtomicU32::new(0),
            audio_buffer: Mutex::new(Vec::new()),
        };

        crate::info!(
            "[UAC] Initialized {} device: {} ({} Hz, {} ch, {} bit)",
            if is_input { "input" } else { "output" },
            device.usb_device.name,
            device.config.sample_rate.as_hz(),
            device.config.channels,
            device.config.bit_depth
        );

        Ok(device)
    }

    /// Get device name
    pub fn name(&self) -> &str {
        &self.usb_device.name
    }

    /// Get device ID
    pub fn device_id(&self) -> u8 {
        self.usb_device.id
    }

    /// Check if device is input (microphone)
    pub fn is_input_device(&self) -> bool {
        self.is_input
    }

    /// Check if device is output (speaker)
    pub fn is_output_device(&self) -> bool {
        !self.is_input
    }

    /// Set audio configuration
    pub fn set_config(&mut self, config: AudioStreamConfig) -> DriverResult<()> {
        if self.streaming.load(Ordering::Acquire) {
            return Err(DriverError::Busy);
        }

        // In a real implementation, this would:
        // 1. Send SET_CUR request to Audio Streaming interface
        // 2. Configure sampling frequency
        // 3. Set format type and channel configuration
        // 4. Adjust endpoint packet sizes

        self.config = config;

        crate::info!(
            "[UAC] Set config: {} Hz, {} ch, {} bit",
            config.sample_rate.as_hz(),
            config.channels,
            config.bit_depth
        );

        Ok(())
    }

    /// Get current configuration
    pub fn get_config(&self) -> AudioStreamConfig {
        self.config
    }

    /// Set volume
    pub fn set_volume(&self, volume_percent: u8) -> DriverResult<()> {
        let mut controls = self.controls.lock();
        controls.set_volume_percent(volume_percent);

        // In a real implementation, this would send
        // SET_CUR request to Feature Unit with volume control

        crate::debug!("[UAC] Set volume: {}%", volume_percent);
        Ok(())
    }

    /// Get volume
    pub fn get_volume(&self) -> u8 {
        self.controls.lock().get_volume_percent()
    }

    /// Mute audio
    pub fn mute(&self) -> DriverResult<()> {
        let mut controls = self.controls.lock();
        controls.mute = true;

        // Send SET_CUR request with mute control
        crate::debug!("[UAC] Muted");
        Ok(())
    }

    /// Unmute audio
    pub fn unmute(&self) -> DriverResult<()> {
        let mut controls = self.controls.lock();
        controls.mute = false;

        // Send SET_CUR request with mute control
        crate::debug!("[UAC] Unmuted");
        Ok(())
    }

    /// Check if muted
    pub fn is_muted(&self) -> bool {
        self.controls.lock().mute
    }

    /// Start audio streaming
    pub fn start_streaming(&mut self) -> DriverResult<()> {
        if self.streaming.load(Ordering::Acquire) {
            return Ok(());
        }

        // In a real implementation:
        // 1. Select alternate setting for streaming interface
        // 2. Configure isochronous endpoint
        // 3. Start USB transfers
        // 4. Setup DMA buffers for audio data

        self.streaming.store(true, Ordering::Release);
        self.sample_count.store(0, Ordering::Release);

        crate::info!("[UAC] Started streaming");
        Ok(())
    }

    /// Stop audio streaming
    pub fn stop_streaming(&mut self) -> DriverResult<()> {
        if !self.streaming.load(Ordering::Acquire) {
            return Ok(());
        }

        // Select alternate setting 0 (zero bandwidth)
        // Stop USB transfers

        self.streaming.store(false, Ordering::Release);
        crate::info!("[UAC] Stopped streaming");
        Ok(())
    }

    /// Write audio data (for output devices)
    pub fn write_audio(&mut self, data: &[u8]) -> DriverResult<usize> {
        if !self.is_output_device() {
            return Err(DriverError::InvalidState);
        }

        if !self.streaming.load(Ordering::Acquire) {
            return Err(DriverError::InvalidState);
        }

        // In a real implementation:
        // 1. Copy data to USB transfer buffer
        // 2. Submit isochronous transfer
        // 3. Handle synchronization and timing

        let samples = data.len() / self.config.bytes_per_frame();
        self.sample_count.fetch_add(samples as u32, Ordering::SeqCst);

        Ok(data.len())
    }

    /// Read audio data (for input devices)
    pub fn read_audio(&mut self, buffer: &mut [u8]) -> DriverResult<usize> {
        if !self.is_input_device() {
            return Err(DriverError::InvalidState);
        }

        if !self.streaming.load(Ordering::Acquire) {
            return Err(DriverError::InvalidState);
        }

        // In a real implementation:
        // 1. Wait for audio data from isochronous endpoint
        // 2. Copy data from USB buffer to user buffer
        // 3. Handle synchronization

        let samples = buffer.len() / self.config.bytes_per_frame();
        self.sample_count.fetch_add(samples as u32, Ordering::SeqCst);

        // Return mock data for now
        for byte in buffer.iter_mut() {
            *byte = 0;
        }

        Ok(buffer.len())
    }

    /// Get total samples processed
    pub fn get_sample_count(&self) -> u32 {
        self.sample_count.load(Ordering::Acquire)
    }

    /// Check if streaming
    pub fn is_streaming(&self) -> bool {
        self.streaming.load(Ordering::Acquire)
    }
}

/// Detect all USB audio devices
pub fn detect_audio_devices() -> Vec<UsbAudioDevice> {
    let mut devices = Vec::new();

    // Get all USB devices
    let usb_devices = crate::drivers::usb::enumerate_devices();

    for device in usb_devices {
        // Check if device is an audio class device
        if device.class == DeviceClass::Audio {
            // Determine if input or output by parsing descriptors
            // For now, create both possibilities
            match UsbAudioDevice::new(device.clone(), false) {
                Ok(audio_device) => {
                    crate::info!("[UAC] Detected output device: {}", audio_device.name());
                    devices.push(audio_device);
                }
                Err(e) => {
                    crate::warn!("[UAC] Failed to initialize output device: {:?}", e);
                }
            }
        }
    }

    devices
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_rate_conversion() {
        assert_eq!(SampleRate::from_hz(48000), Some(SampleRate::Rate48000));
        assert_eq!(SampleRate::Rate48000.as_hz(), 48000);
    }

    #[test]
    fn test_audio_config_calculations() {
        let config = AudioStreamConfig::new(SampleRate::Rate48000, 2, 16);
        assert_eq!(config.bytes_per_sample(), 2);
        assert_eq!(config.bytes_per_frame(), 4);
        assert_eq!(config.bytes_per_second(), 192000);
    }

    #[test]
    fn test_volume_control() {
        let mut controls = FeatureControls::new();
        controls.set_volume_percent(50);
        assert_eq!(controls.get_volume_percent(), 50);

        controls.set_volume_db(-20);
        assert_eq!(controls.get_volume_db(), -20);
    }
}
