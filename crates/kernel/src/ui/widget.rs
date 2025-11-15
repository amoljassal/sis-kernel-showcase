/// Widget Trait - Phase G.2
///
/// Base trait for all UI widgets

use super::event::{InputEvent, EventResponse, SizeConstraints, Size};
use super::theme::Theme;
use crate::graphics::{DrawContext, Rect, Font};

/// Base widget trait
pub trait Widget {
    /// Draw the widget to the context
    fn draw(&self, ctx: &mut DrawContext, bounds: Rect, theme: &Theme, font: &Font);

    /// Handle input event
    /// Returns EventResponse indicating if event was consumed
    fn handle_event(&mut self, event: &InputEvent, bounds: Rect) -> EventResponse {
        EventResponse::Ignored
    }

    /// Calculate preferred size given constraints
    fn preferred_size(&self, constraints: &SizeConstraints, font: &Font) -> Size;

    /// Called when widget gains focus
    fn on_focus(&mut self) {}

    /// Called when widget loses focus
    fn on_blur(&mut self) {}

    /// Check if widget can receive focus
    fn can_focus(&self) -> bool {
        false
    }

    /// Check if widget is enabled
    fn is_enabled(&self) -> bool {
        true
    }

    /// Check if widget is visible
    fn is_visible(&self) -> bool {
        true
    }
}

/// Widget state for interactive elements
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetState {
    Normal,
    Hovered,
    Pressed,
    Focused,
    Disabled,
}

/// Helper to check if point is in bounds
pub fn is_point_in_bounds(x: u32, y: u32, bounds: &Rect) -> bool {
    bounds.contains(x, y)
}

/// Padding for widget spacing
#[derive(Debug, Clone, Copy)]
pub struct Padding {
    pub left: u32,
    pub right: u32,
    pub top: u32,
    pub bottom: u32,
}

impl Padding {
    pub const fn all(value: u32) -> Self {
        Self {
            left: value,
            right: value,
            top: value,
            bottom: value,
        }
    }

    pub const fn symmetric(horizontal: u32, vertical: u32) -> Self {
        Self {
            left: horizontal,
            right: horizontal,
            top: vertical,
            bottom: vertical,
        }
    }

    pub const fn zero() -> Self {
        Self::all(0)
    }

    pub fn horizontal(&self) -> u32 {
        self.left + self.right
    }

    pub fn vertical(&self) -> u32 {
        self.top + self.bottom
    }
}
