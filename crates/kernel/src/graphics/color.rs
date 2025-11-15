/// Color representation for graphics
///
/// RGBA color with 8 bits per channel

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    /// Create color from RGB (alpha = 255)
    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Create color from RGBA
    pub const fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Create color from 32-bit ARGB value
    pub const fn from_argb(argb: u32) -> Self {
        Self {
            a: ((argb >> 24) & 0xFF) as u8,
            r: ((argb >> 16) & 0xFF) as u8,
            g: ((argb >> 8) & 0xFF) as u8,
            b: (argb & 0xFF) as u8,
        }
    }

    /// Convert to 32-bit ARGB value (for framebuffer)
    pub const fn to_argb(self) -> u32 {
        ((self.a as u32) << 24) | ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    /// Convert to 32-bit BGRA value (alternative format)
    pub const fn to_bgra(self) -> u32 {
        ((self.b as u32) << 24) | ((self.g as u32) << 16) | ((self.r as u32) << 8) | (self.a as u32)
    }

    /// Create color with modified alpha
    pub const fn with_alpha(self, alpha: u8) -> Self {
        Self { a: alpha, ..self }
    }

    /// Lighten color by factor (0.0 = no change, 1.0 = white)
    pub fn lighten(self, factor: f32) -> Self {
        let factor = factor.clamp(0.0, 1.0);
        Self {
            r: (self.r as f32 + (255.0 - self.r as f32) * factor) as u8,
            g: (self.g as f32 + (255.0 - self.g as f32) * factor) as u8,
            b: (self.b as f32 + (255.0 - self.b as f32) * factor) as u8,
            a: self.a,
        }
    }

    /// Darken color by factor (0.0 = no change, 1.0 = black)
    pub fn darken(self, factor: f32) -> Self {
        let factor = factor.clamp(0.0, 1.0);
        Self {
            r: (self.r as f32 * (1.0 - factor)) as u8,
            g: (self.g as f32 * (1.0 - factor)) as u8,
            b: (self.b as f32 * (1.0 - factor)) as u8,
            a: self.a,
        }
    }
}

// Common color constants
impl Color {
    pub const BLACK: Color = Color::from_rgb(0, 0, 0);
    pub const WHITE: Color = Color::from_rgb(255, 255, 255);
    pub const RED: Color = Color::from_rgb(255, 0, 0);
    pub const GREEN: Color = Color::from_rgb(0, 255, 0);
    pub const BLUE: Color = Color::from_rgb(0, 0, 255);
    pub const YELLOW: Color = Color::from_rgb(255, 255, 0);
    pub const CYAN: Color = Color::from_rgb(0, 255, 255);
    pub const MAGENTA: Color = Color::from_rgb(255, 0, 255);
    pub const GRAY: Color = Color::from_rgb(128, 128, 128);
    pub const DARK_GRAY: Color = Color::from_rgb(64, 64, 64);
    pub const LIGHT_GRAY: Color = Color::from_rgb(192, 192, 192);
    pub const TRANSPARENT: Color = Color::from_rgba(0, 0, 0, 0);

    // Modern UI colors (inspired by VS Code dark theme)
    pub const UI_BG_DARK: Color = Color::from_rgb(30, 30, 30);
    pub const UI_BG_MEDIUM: Color = Color::from_rgb(45, 45, 48);
    pub const UI_BG_LIGHT: Color = Color::from_rgb(60, 60, 60);
    pub const UI_ACCENT: Color = Color::from_rgb(0, 122, 204);
    pub const UI_TEXT: Color = Color::from_rgb(204, 204, 204);
    pub const UI_TEXT_DIM: Color = Color::from_rgb(140, 140, 140);
    pub const UI_BORDER: Color = Color::from_rgb(80, 80, 80);
    pub const UI_SELECTION: Color = Color::from_rgb(38, 79, 120);
}
