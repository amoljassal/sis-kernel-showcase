/// Voice UI Module - Phase G.5
///
/// Voice interface widgets for JARVIS integration

pub mod ui;
pub mod wake_word;

pub use ui::VoiceUIWidget;
pub use wake_word::{WakeWordDetector, WakeWordResult};
