/// Voice UI Widget - Phase G.5
///
/// Voice interaction widget with waveform visualization and transcript display

use crate::ui::{Widget, EventResponse, InputEvent, SizeConstraints, Size, Theme};
use crate::graphics::{DrawContext, Rect, Font, Color};
use alloc::string::String;
use alloc::vec::Vec;

/// Voice UI state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoiceState {
    Idle,           // Waiting for activation
    Listening,      // Actively listening
    Processing,     // Processing speech
    Speaking,       // JARVIS is responding
}

/// Voice UI Widget
pub struct VoiceUIWidget {
    state: VoiceState,
    audio_samples: Vec<f32>,      // Last N samples for waveform (normalized -1.0 to 1.0)
    max_samples: usize,            // Maximum samples to display
    transcript: Option<String>,    // User's speech transcript
    response: Option<String>,      // JARVIS response text
    confidence: f32,               // Speech recognition confidence (0.0 - 1.0)
    wake_word_active: bool,        // Wake word ("Hey JARVIS") detected
}

impl VoiceUIWidget {
    /// Create a new voice UI widget
    pub fn new() -> Self {
        Self {
            state: VoiceState::Idle,
            audio_samples: Vec::new(),
            max_samples: 200,
            transcript: None,
            response: None,
            confidence: 0.0,
            wake_word_active: false,
        }
    }

    /// Set voice state
    pub fn set_state(&mut self, state: VoiceState) {
        self.state = state;
    }

    /// Add audio samples for waveform
    pub fn add_audio_samples(&mut self, samples: &[i16]) {
        // Convert to normalized float and add to buffer
        for &sample in samples {
            let normalized = sample as f32 / 32768.0;
            self.audio_samples.push(normalized);
        }

        // Keep only recent samples
        if self.audio_samples.len() > self.max_samples {
            let excess = self.audio_samples.len() - self.max_samples;
            self.audio_samples.drain(0..excess);
        }
    }

    /// Clear audio samples
    pub fn clear_audio(&mut self) {
        self.audio_samples.clear();
    }

    /// Set transcript
    pub fn set_transcript(&mut self, transcript: String, confidence: f32) {
        self.transcript = Some(transcript);
        self.confidence = confidence;
    }

    /// Set response
    pub fn set_response(&mut self, response: String) {
        self.response = Some(response);
    }

    /// Clear transcript and response
    pub fn clear_conversation(&mut self) {
        self.transcript = None;
        self.response = None;
        self.confidence = 0.0;
    }

    /// Activate wake word
    pub fn activate_wake_word(&mut self) {
        self.wake_word_active = true;
        self.state = VoiceState::Listening;
    }

    /// Deactivate wake word
    pub fn deactivate_wake_word(&mut self) {
        self.wake_word_active = false;
        self.state = VoiceState::Idle;
    }

    /// Draw the waveform
    fn draw_waveform(&self, ctx: &mut DrawContext, bounds: Rect, theme: &Theme) {
        // Draw waveform background
        ctx.fill_rect(bounds, Color::from_rgb(20, 20, 25));
        ctx.draw_rect_outline(bounds, theme.border, 1);

        if self.audio_samples.is_empty() {
            return;
        }

        let center_y = bounds.y + bounds.height / 2;
        let amplitude_scale = (bounds.height / 2) as f32 * 0.8;

        // Draw center line
        ctx.fill_rect(
            Rect::new(bounds.x, center_y, bounds.width, 1),
            Color::from_rgb(60, 60, 65),
        );

        // Draw waveform
        let samples_to_draw = self.audio_samples.len().min(bounds.width as usize);
        let x_step = bounds.width as f32 / samples_to_draw as f32;

        let waveform_color = match self.state {
            VoiceState::Idle => Color::from_rgb(100, 100, 100),
            VoiceState::Listening => Color::from_rgb(0, 200, 255),
            VoiceState::Processing => Color::from_rgb(255, 200, 0),
            VoiceState::Speaking => Color::from_rgb(100, 255, 100),
        };

        for (i, &sample) in self.audio_samples.iter().rev().take(samples_to_draw).enumerate() {
            let x = bounds.x + bounds.width - (i as f32 * x_step) as u32;
            let y_offset = (sample * amplitude_scale) as i32;
            let y1 = (center_y as i32 - y_offset).max(bounds.y as i32) as u32;
            let y2 = (center_y as i32 + y_offset).min((bounds.y + bounds.height) as i32) as u32;

            ctx.fill_rect(Rect::new(x, y1, 2, y2 - y1 + 1), waveform_color);
        }
    }

    /// Draw state indicator
    fn draw_state_indicator(&self, ctx: &mut DrawContext, x: u32, y: u32, font: &Font, theme: &Theme) {
        let (icon, text, color) = match self.state {
            VoiceState::Idle => ("⏸", "Idle - Say 'Hey JARVIS' to activate", Color::from_rgb(150, 150, 150)),
            VoiceState::Listening => ("🎤", "Listening...", Color::from_rgb(0, 200, 255)),
            VoiceState::Processing => ("⚡", "Processing...", Color::from_rgb(255, 200, 0)),
            VoiceState::Speaking => ("🔊", "JARVIS is speaking...", Color::from_rgb(100, 255, 100)),
        };

        // Draw icon
        ctx.draw_text(x, y, icon, font, color);

        // Draw text
        ctx.draw_text(x + 30, y, text, font, color);

        // Draw wake word indicator
        if self.wake_word_active {
            let indicator_text = "● Active";
            ctx.draw_text(x + 300, y, indicator_text, font, Color::from_rgb(100, 255, 100));
        }
    }
}

impl Default for VoiceUIWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for VoiceUIWidget {
    fn draw(&self, ctx: &mut DrawContext, bounds: Rect, theme: &Theme, font: &Font) {
        // Draw background
        ctx.fill_rect(bounds, Color::from_rgb(25, 25, 30));

        let mut y = bounds.y + 15;
        let x = bounds.x + 15;

        // Draw title
        ctx.draw_text(
            x,
            y,
            "JARVIS Voice Interface",
            font,
            theme.accent,
        );

        y += 30;

        // Draw state indicator
        self.draw_state_indicator(ctx, x, y, font, theme);

        y += 35;

        // Draw waveform
        let waveform_bounds = Rect::new(x, y, bounds.width - 30, 100);
        self.draw_waveform(ctx, waveform_bounds, theme);

        y += 115;

        // Draw transcript section
        if let Some(ref transcript) = self.transcript {
            ctx.draw_text(x, y, "You:", font, theme.text_secondary);
            y += 20;

            // Draw transcript text
            ctx.draw_text(x + 10, y, transcript, font, theme.text_primary);

            // Draw confidence
            let confidence_text = alloc::format!("(Confidence: {:.0}%)", self.confidence * 100.0);
            ctx.draw_text(
                x + bounds.width - 150,
                y,
                &confidence_text,
                font,
                if self.confidence > 0.8 {
                    Color::from_rgb(100, 255, 100)
                } else if self.confidence > 0.5 {
                    Color::from_rgb(255, 200, 0)
                } else {
                    Color::from_rgb(255, 100, 100)
                },
            );

            y += 30;
        }

        // Draw response section
        if let Some(ref response) = self.response {
            ctx.draw_text(x, y, "JARVIS:", font, Color::from_rgb(100, 200, 255));
            y += 20;

            // Draw response text (with word wrapping if needed)
            let words = response.split(' ');
            let mut line = String::new();
            let max_line_width = bounds.width - 50;

            for word in words {
                let test_line = if line.is_empty() {
                    word.to_string()
                } else {
                    alloc::format!("{} {}", line, word)
                };

                let (width, _) = font.measure_text(&test_line);

                if width > max_line_width && !line.is_empty() {
                    // Draw current line and start new one
                    ctx.draw_text(x + 10, y, &line, font, Color::from_rgb(200, 255, 200));
                    y += 20;
                    line = word.to_string();
                } else {
                    line = test_line;
                }
            }

            // Draw remaining line
            if !line.is_empty() {
                ctx.draw_text(x + 10, y, &line, font, Color::from_rgb(200, 255, 200));
            }

            y += 25;
        }

        // Draw help text
        y = bounds.y + bounds.height - 30;
        ctx.draw_text(
            x,
            y,
            "Integration ready for Whisper STT + OpenAI TTS",
            font,
            Color::from_rgb(120, 120, 120),
        );

        // Draw border
        ctx.draw_rect_outline(bounds, theme.border, 1);
    }

    fn handle_event(&mut self, event: &InputEvent, bounds: Rect) -> EventResponse {
        // Voice UI doesn't handle mouse/keyboard events directly
        // It's controlled programmatically by audio processing
        EventResponse::Ignored
    }

    fn preferred_size(&self, constraints: &SizeConstraints, font: &Font) -> Size {
        let base_height = 300;
        let transcript_height = if self.transcript.is_some() { 60 } else { 0 };
        let response_height = if self.response.is_some() { 80 } else { 0 };

        Size::new(
            600,
            base_height + transcript_height + response_height,
        )
    }
}
