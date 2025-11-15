/// Audio Infrastructure - Phase G.5
///
/// Audio input/output pipeline for voice assistant integration

pub mod input;
pub mod output;
pub mod vad;
pub mod buffer;

pub use input::{AudioInputDevice, InputError};
pub use output::{AudioOutputDevice, OutputError};
pub use vad::{VAD, VADResult};
pub use buffer::RingBuffer;

use spin::Mutex;
use alloc::sync::Arc;
use crate::lib::error::Result;

/// Audio sample rate (16kHz for voice)
pub const SAMPLE_RATE: u32 = 16000;

/// Audio channels (mono for voice)
pub const CHANNELS: u8 = 1;

/// Buffer size in samples
pub const BUFFER_SIZE: usize = 4096;

/// Audio format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
    PCM16,      // 16-bit PCM
    PCM32,      // 32-bit PCM
    Float32,    // 32-bit float
}

/// Audio manager
pub struct AudioManager {
    input_device: Option<Arc<Mutex<AudioInputDevice>>>,
    output_device: Option<Arc<Mutex<AudioOutputDevice>>>,
    sample_rate: u32,
    channels: u8,
    format: AudioFormat,
    recording: bool,
}

impl AudioManager {
    /// Create a new audio manager
    pub fn new() -> Self {
        Self {
            input_device: None,
            output_device: None,
            sample_rate: SAMPLE_RATE,
            channels: CHANNELS,
            format: AudioFormat::PCM16,
            recording: false,
        }
    }

    /// Initialize audio devices
    pub fn init(&mut self) -> Result<()> {
        // Try to detect USB audio input device
        match AudioInputDevice::detect() {
            Ok(device) => {
                crate::info!("audio: detected input device - {}", device.name());
                self.input_device = Some(Arc::new(Mutex::new(device)));
            }
            Err(e) => {
                crate::warn!("audio: no input device found - {:?}", e);
            }
        }

        // Try to detect audio output device
        match AudioOutputDevice::detect() {
            Ok(device) => {
                crate::info!("audio: detected output device - {}", device.name());
                self.output_device = Some(Arc::new(Mutex::new(device)));
            }
            Err(e) => {
                crate::warn!("audio: no output device found - {:?}", e);
            }
        }

        Ok(())
    }

    /// Start recording audio
    pub fn start_recording(&mut self) -> Result<()> {
        if let Some(ref input) = self.input_device {
            input.lock().start()?;
            self.recording = true;
            crate::info!("audio: started recording");
            Ok(())
        } else {
            Err(crate::lib::error::Errno::ENODEV)
        }
    }

    /// Stop recording audio
    pub fn stop_recording(&mut self) -> Result<()> {
        if let Some(ref input) = self.input_device {
            input.lock().stop()?;
            self.recording = false;
            crate::info!("audio: stopped recording");
            Ok(())
        } else {
            Err(crate::lib::error::Errno::ENODEV)
        }
    }

    /// Read audio samples from input
    pub fn read_samples(&self, buffer: &mut [i16]) -> Result<usize> {
        if let Some(ref input) = self.input_device {
            input.lock().read(buffer)
        } else {
            Err(crate::lib::error::Errno::ENODEV)
        }
    }

    /// Write audio samples to output
    pub fn write_samples(&mut self, buffer: &[i16]) -> Result<usize> {
        if let Some(ref output) = self.output_device {
            output.lock().write(buffer)
        } else {
            Err(crate::lib::error::Errno::ENODEV)
        }
    }

    /// Play audio buffer
    pub fn play_audio(&mut self, samples: &[i16]) -> Result<()> {
        self.write_samples(samples)?;
        Ok(())
    }

    /// Check if recording is active
    pub fn is_recording(&self) -> bool {
        self.recording
    }

    /// Get sample rate
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Get number of channels
    pub fn channels(&self) -> u8 {
        self.channels
    }

    /// Check if input device is available
    pub fn has_input(&self) -> bool {
        self.input_device.is_some()
    }

    /// Check if output device is available
    pub fn has_output(&self) -> bool {
        self.output_device.is_some()
    }
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Global audio manager
static AUDIO_MANAGER: Mutex<Option<AudioManager>> = Mutex::new(None);

/// Initialize global audio manager
pub fn init() -> Result<()> {
    let mut manager = AudioManager::new();
    manager.init()?;
    *AUDIO_MANAGER.lock() = Some(manager);
    crate::info!("audio: audio subsystem initialized");
    Ok(())
}

/// Get global audio manager
pub fn get_manager() -> Option<&'static Mutex<Option<AudioManager>>> {
    Some(&AUDIO_MANAGER)
}

/// Start recording
pub fn start_recording() -> Result<()> {
    if let Some(ref mut manager) = *AUDIO_MANAGER.lock() {
        manager.start_recording()
    } else {
        Err(crate::lib::error::Errno::ENODEV)
    }
}

/// Stop recording
pub fn stop_recording() -> Result<()> {
    if let Some(ref mut manager) = *AUDIO_MANAGER.lock() {
        manager.stop_recording()
    } else {
        Err(crate::lib::error::Errno::ENODEV)
    }
}

/// Read audio samples
pub fn read_samples(buffer: &mut [i16]) -> Result<usize> {
    if let Some(ref manager) = *AUDIO_MANAGER.lock() {
        manager.read_samples(buffer)
    } else {
        Err(crate::lib::error::Errno::ENODEV)
    }
}

/// Write audio samples
pub fn write_samples(buffer: &[i16]) -> Result<usize> {
    if let Some(ref mut manager) = *AUDIO_MANAGER.lock() {
        manager.write_samples(buffer)
    } else {
        Err(crate::lib::error::Errno::ENODEV)
    }
}
