/// UI Event System - Phase G.2
///
/// Provides event types and handling for UI interactions

use crate::graphics::Rect;

/// Input event from mouse or keyboard
#[derive(Debug, Clone, Copy)]
pub enum InputEvent {
    /// Mouse moved to position
    MouseMove { x: u32, y: u32 },

    /// Mouse button pressed or released
    MouseButton {
        button: MouseButton,
        pressed: bool,
        x: u32,
        y: u32,
    },

    /// Key pressed
    KeyPress {
        key: KeyCode,
        modifiers: KeyModifiers,
    },

    /// Key released
    KeyRelease {
        key: KeyCode,
    },

    /// Text input (character typed)
    TextInput {
        character: char,
    },
}

/// Mouse button
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left = 0,
    Right = 1,
    Middle = 2,
}

/// Keyboard key codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCode {
    Unknown,

    // Letters
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,

    // Numbers
    Key0, Key1, Key2, Key3, Key4, Key5, Key6, Key7, Key8, Key9,

    // Function keys
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,

    // Control keys
    Escape,
    Enter,
    Tab,
    Backspace,
    Delete,
    Insert,
    Space,

    // Arrow keys
    Left,
    Right,
    Up,
    Down,

    // Modifier keys
    LeftShift,
    RightShift,
    LeftCtrl,
    RightCtrl,
    LeftAlt,
    RightAlt,

    // Other
    Home,
    End,
    PageUp,
    PageDown,
}

/// Keyboard modifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyModifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

impl KeyModifiers {
    pub const fn none() -> Self {
        Self {
            shift: false,
            ctrl: false,
            alt: false,
        }
    }

    pub const fn shift() -> Self {
        Self {
            shift: true,
            ctrl: false,
            alt: false,
        }
    }

    pub const fn ctrl() -> Self {
        Self {
            shift: false,
            ctrl: true,
            alt: false,
        }
    }

    pub const fn alt() -> Self {
        Self {
            shift: false,
            ctrl: false,
            alt: true,
        }
    }
}

/// Event response from widget
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventResponse {
    /// Event was consumed, stop propagation
    Consumed,

    /// Event was ignored, continue propagation
    Ignored,

    /// Event consumed and needs redraw
    NeedsRedraw,
}

/// Size constraints for layout
#[derive(Debug, Clone, Copy)]
pub struct SizeConstraints {
    pub min_width: u32,
    pub max_width: u32,
    pub min_height: u32,
    pub max_height: u32,
}

impl SizeConstraints {
    pub fn new(min_width: u32, max_width: u32, min_height: u32, max_height: u32) -> Self {
        Self {
            min_width,
            max_width,
            min_height,
            max_height,
        }
    }

    pub fn tight(width: u32, height: u32) -> Self {
        Self {
            min_width: width,
            max_width: width,
            min_height: height,
            max_height: height,
        }
    }

    pub fn loose(max_width: u32, max_height: u32) -> Self {
        Self {
            min_width: 0,
            max_width,
            min_height: 0,
            max_height,
        }
    }

    pub fn constrain(&self, width: u32, height: u32) -> (u32, u32) {
        let w = width.clamp(self.min_width, self.max_width);
        let h = height.clamp(self.min_height, self.max_height);
        (w, h)
    }
}

/// Widget size
#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}
