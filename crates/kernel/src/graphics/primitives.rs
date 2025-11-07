/// 2D Graphics Primitives - Phase G.0
///
/// Provides basic 2D drawing operations: rectangles, lines, circles, text

use super::color::Color;
use super::font::Font;
use crate::drivers::virtio_gpu::VirtioGpu;
use alloc::sync::Arc;
use spin::Mutex;

/// Rectangle bounds
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }

    pub fn contains(&self, px: u32, py: u32) -> bool {
        px >= self.x && px < (self.x + self.width) &&
        py >= self.y && py < (self.y + self.height)
    }

    pub fn shrink_by(&self, horizontal: u32, vertical: u32) -> Self {
        Self {
            x: self.x + horizontal,
            y: self.y + vertical,
            width: self.width.saturating_sub(horizontal * 2),
            height: self.height.saturating_sub(vertical * 2),
        }
    }
}

/// 2D drawing context
pub struct DrawContext {
    gpu: Arc<Mutex<VirtioGpu>>,
    width: u32,
    height: u32,
    dirty_rect: Option<Rect>,
}

impl DrawContext {
    /// Create new drawing context from GPU device
    pub fn new(gpu: Arc<Mutex<VirtioGpu>>) -> Self {
        let (width, height) = gpu.lock().resolution();
        Self {
            gpu,
            width,
            height,
            dirty_rect: None,
        }
    }

    /// Get framebuffer dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Draw a single pixel
    pub fn draw_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x >= self.width || y >= self.height {
            return;
        }

        let mut wrote = false;
        {
            let gpu = self.gpu.lock();
            let fb = gpu.get_framebuffer();
            let offset = (y * self.width + x) as usize;

            if offset < fb.len() {
                fb[offset] = color.to_argb();
                wrote = true;
            }
        }
        if wrote {
            self.mark_dirty(x, y, 1, 1);
        }
    }

    /// Fill a rectangle with solid color
    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        let argb = color.to_argb();
        {
            let gpu = self.gpu.lock();
            let fb = gpu.get_framebuffer();

            let x_end = (rect.x + rect.width).min(self.width);
            let y_end = (rect.y + rect.height).min(self.height);

            for y in rect.y..y_end {
                for x in rect.x..x_end {
                    let offset = (y * self.width + x) as usize;
                    if offset < fb.len() {
                        fb[offset] = argb;
                    }
                }
            }
        }

        self.mark_dirty(rect.x, rect.y, rect.width, rect.height);
    }

    /// Draw rectangle outline
    pub fn draw_rect_outline(&mut self, rect: Rect, color: Color, thickness: u32) {
        if thickness == 0 {
            return;
        }

        // Top
        self.fill_rect(
            Rect::new(rect.x, rect.y, rect.width, thickness),
            color
        );

        // Bottom
        if rect.height > thickness {
            self.fill_rect(
                Rect::new(rect.x, rect.y + rect.height - thickness, rect.width, thickness),
                color
            );
        }

        // Left
        if rect.height > thickness * 2 {
            self.fill_rect(
                Rect::new(rect.x, rect.y + thickness, thickness, rect.height - thickness * 2),
                color
            );
        }

        // Right
        if rect.width > thickness && rect.height > thickness * 2 {
            self.fill_rect(
                Rect::new(rect.x + rect.width - thickness, rect.y + thickness, thickness, rect.height - thickness * 2),
                color
            );
        }
    }

    /// Draw a line using Bresenham's algorithm
    pub fn draw_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: Color) {
        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx - dy;

        let mut x = x0;
        let mut y = y0;

        loop {
            if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
                self.draw_pixel(x as u32, y as u32, color);
            }

            if x == x1 && y == y1 {
                break;
            }

            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }
    }

    /// Draw a circle using midpoint circle algorithm
    pub fn draw_circle(&mut self, cx: i32, cy: i32, radius: i32, color: Color) {
        let mut x = radius;
        let mut y = 0;
        let mut err = 0;

        while x >= y {
            // Draw 8 octants
            self.plot_circle_points(cx, cy, x, y, color);

            if err <= 0 {
                y += 1;
                err += 2 * y + 1;
            }

            if err > 0 {
                x -= 1;
                err -= 2 * x + 1;
            }
        }
    }

    fn plot_circle_points(&mut self, cx: i32, cy: i32, x: i32, y: i32, color: Color) {
        let points = [
            (cx + x, cy + y),
            (cx + y, cy + x),
            (cx - y, cy + x),
            (cx - x, cy + y),
            (cx - x, cy - y),
            (cx - y, cy - x),
            (cx + y, cy - x),
            (cx + x, cy - y),
        ];

        for (px, py) in points.iter() {
            if *px >= 0 && *px < self.width as i32 && *py >= 0 && *py < self.height as i32 {
                self.draw_pixel(*px as u32, *py as u32, color);
            }
        }
    }

    /// Draw filled circle
    pub fn fill_circle(&mut self, cx: i32, cy: i32, radius: i32, color: Color) {
        let mut x = radius;
        let mut y = 0;
        let mut err = 0;

        while x >= y {
            // Draw horizontal lines for each octant
            self.draw_line(cx - x, cy + y, cx + x, cy + y, color);
            self.draw_line(cx - y, cy + x, cx + y, cy + x, color);
            self.draw_line(cx - x, cy - y, cx + x, cy - y, color);
            self.draw_line(cx - y, cy - x, cx + y, cy - x, color);

            if err <= 0 {
                y += 1;
                err += 2 * y + 1;
            }

            if err > 0 {
                x -= 1;
                err -= 2 * x + 1;
            }
        }
    }

    /// Draw text at position
    pub fn draw_text(&mut self, x: u32, y: u32, text: &str, font: &Font, color: Color) {
        let mut cursor_x = x;
        for ch in text.chars() {
            if let Some(glyph) = font.get_glyph(ch) {
                self.draw_glyph(cursor_x, y, glyph, color);
                cursor_x += glyph.advance;

                // Break if we exceed screen width
                if cursor_x >= self.width {
                    break;
                }
            }
        }
    }

    /// Draw a single glyph
    fn draw_glyph(&mut self, x: u32, y: u32, glyph: &super::font::Glyph, color: Color) {
        {
            let gpu = self.gpu.lock();
            let fb = gpu.get_framebuffer();

            for gy in 0..glyph.height {
                for gx in 0..glyph.width {
                    let px = x + gx + glyph.bearing_x as u32;
                    let py = y + gy + glyph.bearing_y as u32;

                    if px < self.width && py < self.height {
                        let bitmap_idx = (gy * glyph.width + gx) as usize;
                        if bitmap_idx < glyph.bitmap.len() {
                            let alpha = glyph.bitmap[bitmap_idx];
                            if alpha > 0 {
                                // Alpha blending
                                let offset = (py * self.width + px) as usize;
                                if offset < fb.len() {
                                    let bg = Color::from_argb(fb[offset]);
                                    let blended = Self::blend_alpha(bg, color, alpha);
                                    fb[offset] = blended.to_argb();
                                }
                            }
                        }
                    }
                }
            }
        }

        self.mark_dirty(x, y, glyph.width, glyph.height);
    }

    /// Alpha blend two colors
    fn blend_alpha(bg: Color, fg: Color, alpha: u8) -> Color {
        let alpha_f = alpha as u32;
        let inv_alpha = 255 - alpha_f;

        let r = ((fg.r as u32 * alpha_f + bg.r as u32 * inv_alpha) / 255) as u8;
        let g = ((fg.g as u32 * alpha_f + bg.g as u32 * inv_alpha) / 255) as u8;
        let b = ((fg.b as u32 * alpha_f + bg.b as u32 * inv_alpha) / 255) as u8;
        let a = fg.a.max(bg.a);

        Color { r, g, b, a }
    }

    /// Measure text dimensions
    pub fn measure_text(&self, text: &str, font: &Font) -> (u32, u32) {
        font.measure_text(text)
    }

    /// Mark region as dirty (needs flushing)
    fn mark_dirty(&mut self, x: u32, y: u32, w: u32, h: u32) {
        let new_rect = Rect::new(x, y, w, h);

        self.dirty_rect = Some(match self.dirty_rect {
            None => new_rect,
            Some(existing) => {
                // Merge rectangles
                let x1 = existing.x.min(new_rect.x);
                let y1 = existing.y.min(new_rect.y);
                let x2 = (existing.x + existing.width).max(new_rect.x + new_rect.width);
                let y2 = (existing.y + existing.height).max(new_rect.y + new_rect.height);
                Rect::new(x1, y1, x2 - x1, y2 - y1)
            }
        });
    }

    /// Flush dirty regions to display
    pub fn flush(&mut self) -> crate::lib::error::Result<()> {
        if let Some(rect) = self.dirty_rect.take() {
            let mut gpu = self.gpu.lock();
            gpu.flush(rect.x, rect.y, rect.width, rect.height)?;
        }
        Ok(())
    }

    /// Flush entire screen to display
    pub fn flush_all(&mut self) -> crate::lib::error::Result<()> {
        let mut gpu = self.gpu.lock();
        gpu.flush(0, 0, self.width, self.height)?;
        self.dirty_rect = None;
        Ok(())
    }

    /// Clear screen with color
    pub fn clear(&mut self, color: Color) {
        self.fill_rect(Rect::new(0, 0, self.width, self.height), color);
    }
}
