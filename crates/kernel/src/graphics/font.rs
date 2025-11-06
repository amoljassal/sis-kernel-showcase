/// Bitmap Font Rendering - Phase G.0
///
/// Provides simple bitmap font support for text rendering

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

/// A single glyph (character) in a font
#[derive(Debug, Clone)]
pub struct Glyph {
    pub bitmap: Vec<u8>,     // Alpha channel only (grayscale)
    pub width: u32,
    pub height: u32,
    pub bearing_x: i32,      // Horizontal bearing (offset from cursor)
    pub bearing_y: i32,      // Vertical bearing (baseline offset)
    pub advance: u32,        // Horizontal advance to next character
}

/// Bitmap font
pub struct Font {
    glyphs: BTreeMap<char, Glyph>,
    size: u32,
    line_height: u32,
}

impl Font {
    /// Create a new empty font
    pub fn new(size: u32, line_height: u32) -> Self {
        Self {
            glyphs: BTreeMap::new(),
            size,
            line_height,
        }
    }

    /// Add a glyph to the font
    pub fn add_glyph(&mut self, ch: char, glyph: Glyph) {
        self.glyphs.insert(ch, glyph);
    }

    /// Get a glyph by character
    pub fn get_glyph(&self, ch: char) -> Option<&Glyph> {
        self.glyphs.get(&ch).or_else(|| self.glyphs.get(&'?'))
    }

    /// Measure text dimensions
    pub fn measure_text(&self, text: &str) -> (u32, u32) {
        let width: u32 = text.chars()
            .filter_map(|ch| self.get_glyph(ch))
            .map(|g| g.advance)
            .sum();
        (width, self.line_height)
    }

    /// Get font size
    pub fn size(&self) -> u32 {
        self.size
    }

    /// Get line height
    pub fn line_height(&self) -> u32 {
        self.line_height
    }
}

/// Create a simple 8x16 bitmap font (system console font)
pub fn create_system_font() -> Font {
    let mut font = Font::new(16, 16);

    // Add basic ASCII characters (A-Z, a-z, 0-9, common symbols)
    // For now, we'll create simple rectangular glyphs as placeholders
    // In a real implementation, these would be pre-rendered bitmaps

    // Add uppercase letters A-Z
    for ch in 'A'..='Z' {
        font.add_glyph(ch, create_simple_glyph(8, 16));
    }

    // Add lowercase letters a-z
    for ch in 'a'..='z' {
        font.add_glyph(ch, create_simple_glyph(8, 16));
    }

    // Add digits 0-9
    for ch in '0'..='9' {
        font.add_glyph(ch, create_simple_glyph(8, 16));
    }

    // Add common symbols
    for ch in [' ', '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '-', '_', '=', '+',
               '[', ']', '{', '}', '\\', '|', ';', ':', '\'', '"', ',', '.', '<', '>', '/',
               '?', '~', '`'] {
        font.add_glyph(ch, create_simple_glyph(8, 16));
    }

    // Special handling for space (no visible pixels)
    font.add_glyph(' ', Glyph {
        bitmap: vec![0; 8 * 16],
        width: 8,
        height: 16,
        bearing_x: 0,
        bearing_y: 0,
        advance: 8,
    });

    font
}

/// Create a simple placeholder glyph (filled rectangle)
fn create_simple_glyph(width: u32, height: u32) -> Glyph {
    // Create a simple bitmap with some pattern to make text visible
    let mut bitmap = Vec::with_capacity((width * height) as usize);

    for y in 0..height {
        for x in 0..width {
            // Create a simple border pattern
            let alpha = if x == 0 || x == width - 1 || y == 0 || y == height - 1 {
                255
            } else if y < height / 4 || y > height * 3 / 4 {
                128
            } else {
                64
            };
            bitmap.push(alpha);
        }
    }

    Glyph {
        bitmap,
        width,
        height,
        bearing_x: 0,
        bearing_y: 0,
        advance: width,
    }
}

/// Create a larger display font (16x32)
pub fn create_display_font() -> Font {
    let mut font = Font::new(32, 32);

    // Add basic ASCII characters with larger size
    for ch in 'A'..='Z' {
        font.add_glyph(ch, create_simple_glyph(16, 32));
    }

    for ch in 'a'..='z' {
        font.add_glyph(ch, create_simple_glyph(16, 32));
    }

    for ch in '0'..='9' {
        font.add_glyph(ch, create_simple_glyph(16, 32));
    }

    for ch in [' ', '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '-', '_', '=', '+',
               '[', ']', '{', '}', '\\', '|', ';', ':', '\'', '"', ',', '.', '<', '>', '/',
               '?', '~', '`'] {
        font.add_glyph(ch, create_simple_glyph(16, 32));
    }

    font.add_glyph(' ', Glyph {
        bitmap: vec![0; 16 * 32],
        width: 16,
        height: 32,
        bearing_x: 0,
        bearing_y: 0,
        advance: 16,
    });

    font
}

/// Load a PSF2 (PC Screen Font 2) format font
/// This is a simple bitmap font format commonly used in Linux console
pub fn load_psf2_font(data: &[u8]) -> Option<Font> {
    if data.len() < 32 {
        return None;
    }

    // PSF2 magic: 0x864ab572
    let magic = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    if magic != 0x864ab572 {
        return None;
    }

    let version = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    let header_size = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
    let flags = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);
    let num_glyphs = u32::from_le_bytes([data[16], data[17], data[18], data[19]]);
    let bytes_per_glyph = u32::from_le_bytes([data[20], data[21], data[22], data[23]]);
    let height = u32::from_le_bytes([data[24], data[25], data[26], data[27]]);
    let width = u32::from_le_bytes([data[28], data[29], data[30], data[31]]);

    let mut font = Font::new(height, height);

    // Load glyphs
    let glyph_data_offset = header_size as usize;
    for i in 0..num_glyphs.min(256) {
        let offset = glyph_data_offset + (i * bytes_per_glyph) as usize;
        if offset + bytes_per_glyph as usize > data.len() {
            break;
        }

        let glyph_bytes = &data[offset..offset + bytes_per_glyph as usize];
        let mut bitmap = Vec::with_capacity((width * height) as usize);

        // Convert bitmap to alpha channel
        for byte in glyph_bytes {
            for bit in (0..8).rev() {
                if (byte & (1 << bit)) != 0 {
                    bitmap.push(255);
                } else {
                    bitmap.push(0);
                }
            }
        }

        bitmap.truncate((width * height) as usize);

        let glyph = Glyph {
            bitmap,
            width,
            height,
            bearing_x: 0,
            bearing_y: 0,
            advance: width,
        };

        if i < 128 {
            font.add_glyph(i as u8 as char, glyph);
        }
    }

    Some(font)
}
