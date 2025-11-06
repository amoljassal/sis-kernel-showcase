/// Window structure and properties - Phase G.1
///
/// Represents a single window with its content, decorations, and state

use crate::graphics::{Color, Rect};
use alloc::vec::Vec;
use alloc::string::String;
use core::any::Any;

/// Window identifier
pub type WindowId = u32;

/// Window structure
pub struct Window {
    pub id: WindowId,
    pub title: String,
    pub bounds: Rect,                   // Total bounds including decorations
    pub content_bounds: Rect,           // Content area (excludes decorations)
    pub framebuffer: Vec<u32>,          // Window's own framebuffer (ARGB)
    pub visible: bool,
    pub focused: bool,
    pub minimized: bool,
    pub fullscreen: bool,
    pub resizable: bool,
    pub movable: bool,
    pub closable: bool,
    pub decoration: WindowDecoration,
    pub z_order: u32,                   // Higher = on top
    pub app_data: Option<Box<dyn Any + Send + Sync>>,
}

/// Window decoration settings
#[derive(Debug, Clone, Copy)]
pub struct WindowDecoration {
    pub title_bar_height: u32,
    pub border_width: u32,
    pub show_title_bar: bool,
    pub show_close_button: bool,
    pub show_minimize_button: bool,
    pub show_maximize_button: bool,
}

impl Default for WindowDecoration {
    fn default() -> Self {
        Self {
            title_bar_height: 30,
            border_width: 2,
            show_title_bar: true,
            show_close_button: true,
            show_minimize_button: true,
            show_maximize_button: true,
        }
    }
}

impl WindowDecoration {
    pub fn none() -> Self {
        Self {
            title_bar_height: 0,
            border_width: 0,
            show_title_bar: false,
            show_close_button: false,
            show_minimize_button: false,
            show_maximize_button: false,
        }
    }
}

/// Window creation specification
pub struct WindowSpec {
    pub title: String,
    pub bounds: Rect,
    pub resizable: bool,
    pub movable: bool,
    pub closable: bool,
    pub decoration: WindowDecoration,
}

impl Default for WindowSpec {
    fn default() -> Self {
        Self {
            title: String::from("Untitled"),
            bounds: Rect::new(100, 100, 400, 300),
            resizable: true,
            movable: true,
            closable: true,
            decoration: WindowDecoration::default(),
        }
    }
}

impl Window {
    /// Create a new window
    pub fn new(id: WindowId, spec: WindowSpec) -> Self {
        let content_bounds = Self::calculate_content_bounds(&spec.bounds, &spec.decoration);

        let fb_size = (spec.bounds.width * spec.bounds.height) as usize;
        let mut framebuffer = Vec::with_capacity(fb_size);
        framebuffer.resize(fb_size, Color::UI_BG_MEDIUM.to_argb());

        Self {
            id,
            title: spec.title,
            bounds: spec.bounds,
            content_bounds,
            framebuffer,
            visible: true,
            focused: false,
            minimized: false,
            fullscreen: false,
            resizable: spec.resizable,
            movable: spec.movable,
            closable: spec.closable,
            decoration: spec.decoration,
            z_order: 0,
            app_data: None,
        }
    }

    /// Calculate content bounds from total bounds and decoration
    fn calculate_content_bounds(bounds: &Rect, decoration: &WindowDecoration) -> Rect {
        let x = bounds.x + decoration.border_width;
        let y = bounds.y + decoration.title_bar_height + decoration.border_width;
        let width = bounds.width.saturating_sub(decoration.border_width * 2);
        let height = bounds.height.saturating_sub(
            decoration.title_bar_height + decoration.border_width * 2
        );

        Rect::new(x, y, width, height)
    }

    /// Update content bounds when window is resized or moved
    pub fn update_content_bounds(&mut self) {
        self.content_bounds = Self::calculate_content_bounds(&self.bounds, &self.decoration);
    }

    /// Get title bar rectangle
    pub fn title_bar_rect(&self) -> Rect {
        Rect::new(
            self.bounds.x,
            self.bounds.y,
            self.bounds.width,
            self.decoration.title_bar_height
        )
    }

    /// Get close button rectangle
    pub fn close_button_rect(&self) -> Rect {
        let size = self.decoration.title_bar_height - 8;
        Rect::new(
            self.bounds.x + self.bounds.width - size - 4,
            self.bounds.y + 4,
            size,
            size
        )
    }

    /// Get minimize button rectangle
    pub fn minimize_button_rect(&self) -> Rect {
        let size = self.decoration.title_bar_height - 8;
        let close_x = self.bounds.x + self.bounds.width - size - 4;
        Rect::new(
            close_x - size - 4,
            self.bounds.y + 4,
            size,
            size
        )
    }

    /// Get maximize button rectangle
    pub fn maximize_button_rect(&self) -> Rect {
        let size = self.decoration.title_bar_height - 8;
        let close_x = self.bounds.x + self.bounds.width - size - 4;
        let minimize_x = close_x - size - 4;
        Rect::new(
            minimize_x - size - 4,
            self.bounds.y + 4,
            size,
            size
        )
    }

    /// Check if point is in title bar (for dragging)
    pub fn is_in_title_bar(&self, x: u32, y: u32) -> bool {
        if !self.decoration.show_title_bar {
            return false;
        }

        let title_bar = self.title_bar_rect();

        // Exclude button areas
        if self.decoration.show_close_button && self.close_button_rect().contains(x, y) {
            return false;
        }
        if self.decoration.show_minimize_button && self.minimize_button_rect().contains(x, y) {
            return false;
        }
        if self.decoration.show_maximize_button && self.maximize_button_rect().contains(x, y) {
            return false;
        }

        title_bar.contains(x, y)
    }

    /// Check if point is on resize border
    pub fn is_on_resize_border(&self, x: u32, y: u32) -> Option<ResizeEdge> {
        if !self.resizable {
            return None;
        }

        let border = self.decoration.border_width;
        let threshold = border.max(4); // At least 4px for easier grabbing

        let left = x >= self.bounds.x.saturating_sub(threshold) &&
                   x <= self.bounds.x + threshold;
        let right = x >= (self.bounds.x + self.bounds.width).saturating_sub(threshold) &&
                    x <= self.bounds.x + self.bounds.width + threshold;
        let top = y >= self.bounds.y.saturating_sub(threshold) &&
                  y <= self.bounds.y + threshold;
        let bottom = y >= (self.bounds.y + self.bounds.height).saturating_sub(threshold) &&
                     y <= self.bounds.y + self.bounds.height + threshold;

        // Check corners first
        if left && top {
            return Some(ResizeEdge::TopLeft);
        }
        if right && top {
            return Some(ResizeEdge::TopRight);
        }
        if left && bottom {
            return Some(ResizeEdge::BottomLeft);
        }
        if right && bottom {
            return Some(ResizeEdge::BottomRight);
        }

        // Check edges
        if left {
            return Some(ResizeEdge::Left);
        }
        if right {
            return Some(ResizeEdge::Right);
        }
        if top {
            return Some(ResizeEdge::Top);
        }
        if bottom {
            return Some(ResizeEdge::Bottom);
        }

        None
    }

    /// Resize window
    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        self.bounds.width = new_width.max(100); // Minimum width
        self.bounds.height = new_height.max(80); // Minimum height

        // Reallocate framebuffer
        let new_size = (self.bounds.width * self.bounds.height) as usize;
        self.framebuffer.clear();
        self.framebuffer.resize(new_size, Color::UI_BG_MEDIUM.to_argb());

        self.update_content_bounds();
    }

    /// Move window
    pub fn move_to(&mut self, x: u32, y: u32) {
        self.bounds.x = x;
        self.bounds.y = y;
        self.update_content_bounds();
    }

    /// Clear window content
    pub fn clear(&mut self, color: Color) {
        let argb = color.to_argb();
        for pixel in self.framebuffer.iter_mut() {
            *pixel = argb;
        }
    }
}

/// Edge/corner being resized
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeEdge {
    Top,
    Bottom,
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}
