/// Theme System - Phase G.2
///
/// Provides styling for UI widgets

use crate::graphics::Color;

/// UI theme defining colors and styles
#[derive(Debug, Clone)]
pub struct Theme {
    // Background colors
    pub bg_primary: Color,
    pub bg_secondary: Color,
    pub bg_tertiary: Color,

    // Text colors
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_disabled: Color,

    // Accent colors
    pub accent: Color,
    pub accent_hover: Color,
    pub accent_pressed: Color,

    // Border colors
    pub border: Color,
    pub border_focus: Color,

    // Button colors
    pub button_bg: Color,
    pub button_bg_hover: Color,
    pub button_bg_pressed: Color,
    pub button_text: Color,

    // Input colors
    pub input_bg: Color,
    pub input_border: Color,
    pub input_border_focus: Color,
    pub input_text: Color,

    // Sizes
    pub border_radius: u32,
    pub border_width: u32,
    pub padding: u32,
    pub spacing: u32,
}

impl Theme {
    /// Create default dark theme (VS Code inspired)
    pub fn dark() -> Self {
        Self {
            bg_primary: Color::from_rgb(30, 30, 30),
            bg_secondary: Color::from_rgb(45, 45, 48),
            bg_tertiary: Color::from_rgb(60, 60, 60),

            text_primary: Color::from_rgb(204, 204, 204),
            text_secondary: Color::from_rgb(140, 140, 140),
            text_disabled: Color::from_rgb(80, 80, 80),

            accent: Color::from_rgb(0, 122, 204),
            accent_hover: Color::from_rgb(0, 140, 230),
            accent_pressed: Color::from_rgb(0, 100, 180),

            border: Color::from_rgb(80, 80, 80),
            border_focus: Color::from_rgb(0, 122, 204),

            button_bg: Color::from_rgb(50, 50, 52),
            button_bg_hover: Color::from_rgb(60, 60, 62),
            button_bg_pressed: Color::from_rgb(40, 40, 42),
            button_text: Color::from_rgb(204, 204, 204),

            input_bg: Color::from_rgb(60, 60, 60),
            input_border: Color::from_rgb(80, 80, 80),
            input_border_focus: Color::from_rgb(0, 122, 204),
            input_text: Color::from_rgb(204, 204, 204),

            border_radius: 3,
            border_width: 1,
            padding: 8,
            spacing: 4,
        }
    }

    /// Create light theme
    pub fn light() -> Self {
        Self {
            bg_primary: Color::from_rgb(255, 255, 255),
            bg_secondary: Color::from_rgb(240, 240, 240),
            bg_tertiary: Color::from_rgb(220, 220, 220),

            text_primary: Color::from_rgb(30, 30, 30),
            text_secondary: Color::from_rgb(100, 100, 100),
            text_disabled: Color::from_rgb(180, 180, 180),

            accent: Color::from_rgb(0, 122, 204),
            accent_hover: Color::from_rgb(0, 140, 230),
            accent_pressed: Color::from_rgb(0, 100, 180),

            border: Color::from_rgb(200, 200, 200),
            border_focus: Color::from_rgb(0, 122, 204),

            button_bg: Color::from_rgb(230, 230, 230),
            button_bg_hover: Color::from_rgb(210, 210, 210),
            button_bg_pressed: Color::from_rgb(190, 190, 190),
            button_text: Color::from_rgb(30, 30, 30),

            input_bg: Color::from_rgb(250, 250, 250),
            input_border: Color::from_rgb(200, 200, 200),
            input_border_focus: Color::from_rgb(0, 122, 204),
            input_text: Color::from_rgb(30, 30, 30),

            border_radius: 3,
            border_width: 1,
            padding: 8,
            spacing: 4,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

/// Theme mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Dark,
    Light,
}

impl ThemeMode {
    /// Toggle between dark and light mode
    pub fn toggle(&self) -> Self {
        match self {
            ThemeMode::Dark => ThemeMode::Light,
            ThemeMode::Light => ThemeMode::Dark,
        }
    }

    /// Get theme for this mode
    pub fn theme(&self) -> Theme {
        match self {
            ThemeMode::Dark => Theme::dark(),
            ThemeMode::Light => Theme::light(),
        }
    }
}

impl Default for ThemeMode {
    fn default() -> Self {
        ThemeMode::Dark
    }
}
