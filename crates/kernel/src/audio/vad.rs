/// Voice Activity Detection - Phase G.5
///
/// Detects voice activity in audio stream using energy and zero-crossing rate

use super::SAMPLE_RATE;

/// VAD result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VADResult {
    /// Voice is active
    VoiceActive,
    /// Silence detected (after voice)
    Silence,
    /// No voice detected
    NoVoice,
}

/// Voice Activity Detector
pub struct VAD {
    /// Energy threshold (0.0 - 1.0)
    energy_threshold: f32,
    /// Zero crossing rate threshold (0.0 - 1.0)
    zcr_threshold: f32,
    /// Silence duration before marking voice as ended (ms)
    silence_duration_ms: u32,
    /// Voice detected flag
    voice_detected: bool,
    /// Silence counter (in frames)
    silence_frames: u32,
    /// Frame size in milliseconds
    frame_size_ms: u32,
    /// Sample rate
    sample_rate: u32,
}

impl VAD {
    /// Create a new VAD with default thresholds
    pub fn new() -> Self {
        Self::with_thresholds(0.01, 0.3, 500)
    }

    /// Create a new VAD with custom thresholds
    pub fn with_thresholds(
        energy_threshold: f32,
        zcr_threshold: f32,
        silence_duration_ms: u32,
    ) -> Self {
        Self {
            energy_threshold,
            zcr_threshold,
            silence_duration_ms,
            voice_detected: false,
            silence_frames: 0,
            frame_size_ms: 30, // 30ms frames (480 samples at 16kHz)
            sample_rate: SAMPLE_RATE,
        }
    }

    /// Process an audio frame and detect voice activity
    pub fn process_frame(&mut self, samples: &[i16]) -> VADResult {
        if samples.is_empty() {
            return VADResult::NoVoice;
        }

        let energy = self.calculate_energy(samples);
        let zcr = self.calculate_zero_crossing_rate(samples);

        // Voice is active if energy is high and ZCR is low (voiced speech)
        // or if energy is moderate and ZCR is high (unvoiced speech like 's', 'sh')
        let is_voice = (energy > self.energy_threshold && zcr < self.zcr_threshold)
            || (energy > self.energy_threshold * 0.5 && zcr < self.zcr_threshold * 1.5);

        if is_voice {
            self.voice_detected = true;
            self.silence_frames = 0;
            VADResult::VoiceActive
        } else {
            if self.voice_detected {
                // Increment silence counter
                self.silence_frames += 1;

                let silence_ms = self.silence_frames * self.frame_size_ms;

                if silence_ms >= self.silence_duration_ms {
                    // Silence threshold reached, mark voice as ended
                    self.voice_detected = false;
                    self.silence_frames = 0;
                    VADResult::NoVoice
                } else {
                    // Still in potential silence pause
                    VADResult::Silence
                }
            } else {
                VADResult::NoVoice
            }
        }
    }

    /// Calculate energy of audio frame
    fn calculate_energy(&self, samples: &[i16]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }

        let sum_squares: f32 = samples
            .iter()
            .map(|&s| {
                let normalized = s as f32 / 32768.0;
                normalized * normalized
            })
            .sum();

        sum_squares / samples.len() as f32
    }

    /// Calculate zero crossing rate
    fn calculate_zero_crossing_rate(&self, samples: &[i16]) -> f32 {
        if samples.len() < 2 {
            return 0.0;
        }

        let mut crossings = 0;

        for i in 1..samples.len() {
            if (samples[i] >= 0 && samples[i - 1] < 0) || (samples[i] < 0 && samples[i - 1] >= 0)
            {
                crossings += 1;
            }
        }

        crossings as f32 / (samples.len() - 1) as f32
    }

    /// Reset VAD state
    pub fn reset(&mut self) {
        self.voice_detected = false;
        self.silence_frames = 0;
    }

    /// Set energy threshold
    pub fn set_energy_threshold(&mut self, threshold: f32) {
        self.energy_threshold = threshold.max(0.0).min(1.0);
    }

    /// Set zero crossing rate threshold
    pub fn set_zcr_threshold(&mut self, threshold: f32) {
        self.zcr_threshold = threshold.max(0.0).min(1.0);
    }

    /// Set silence duration threshold
    pub fn set_silence_duration(&mut self, duration_ms: u32) {
        self.silence_duration_ms = duration_ms;
    }

    /// Get energy threshold
    pub fn energy_threshold(&self) -> f32 {
        self.energy_threshold
    }

    /// Get ZCR threshold
    pub fn zcr_threshold(&self) -> f32 {
        self.zcr_threshold
    }

    /// Check if voice is currently detected
    pub fn is_voice_active(&self) -> bool {
        self.voice_detected
    }
}

impl Default for VAD {
    fn default() -> Self {
        Self::new()
    }
}

/// Adaptive VAD that adjusts thresholds based on background noise
pub struct AdaptiveVAD {
    vad: VAD,
    noise_energy: f32,
    snr_threshold: f32, // Signal-to-noise ratio threshold (dB)
    adaptation_rate: f32,
}

impl AdaptiveVAD {
    /// Create a new adaptive VAD
    pub fn new() -> Self {
        Self {
            vad: VAD::new(),
            noise_energy: 0.001, // Initial noise estimate
            snr_threshold: 6.0,  // 6 dB SNR threshold
            adaptation_rate: 0.1,
        }
    }

    /// Process audio frame with adaptive thresholds
    pub fn process_frame(&mut self, samples: &[i16]) -> VADResult {
        let energy = self.vad.calculate_energy(samples);

        // Update noise estimate during silence
        if !self.vad.voice_detected {
            self.noise_energy = self.noise_energy * (1.0 - self.adaptation_rate)
                + energy * self.adaptation_rate;
        }

        // Calculate signal-to-noise ratio
        let snr_linear = if self.noise_energy > 0.0 {
            energy / self.noise_energy
        } else {
            100.0
        };

        let snr_db = 10.0 * libm::log10f(snr_linear);

        // Adjust energy threshold based on SNR
        if snr_db > self.snr_threshold {
            self.vad
                .set_energy_threshold(self.noise_energy * 2.0);
        }

        self.vad.process_frame(samples)
    }

    /// Reset adaptive state
    pub fn reset(&mut self) {
        self.vad.reset();
        self.noise_energy = 0.001;
    }
}

impl Default for AdaptiveVAD {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vad_silence() {
        let mut vad = VAD::new();
        let silence = vec![0i16; 480]; // 30ms of silence at 16kHz

        let result = vad.process_frame(&silence);
        assert_eq!(result, VADResult::NoVoice);
    }

    #[test]
    fn test_vad_voice() {
        let mut vad = VAD::new();

        // Generate a simple sine wave (voice-like signal)
        let mut signal = vec![0i16; 480];
        for (i, sample) in signal.iter_mut().enumerate() {
            *sample = (10000.0 * libm::sinf(2.0 * core::f32::consts::PI * 200.0 * i as f32 / 16000.0))
                as i16;
        }

        let result = vad.process_frame(&signal);
        assert_eq!(result, VADResult::VoiceActive);
    }

    #[test]
    fn test_vad_silence_after_voice() {
        let mut vad = VAD::new();

        // Voice signal
        let mut voice = vec![0i16; 480];
        for (i, sample) in voice.iter_mut().enumerate() {
            *sample = (10000.0 * libm::sinf(2.0 * core::f32::consts::PI * 200.0 * i as f32 / 16000.0))
                as i16;
        }

        vad.process_frame(&voice);
        assert_eq!(vad.is_voice_active(), true);

        // Silence
        let silence = vec![0i16; 480];
        let result = vad.process_frame(&silence);
        assert_eq!(result, VADResult::Silence);
    }
}
