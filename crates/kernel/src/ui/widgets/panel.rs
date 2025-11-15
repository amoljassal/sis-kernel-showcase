/// Panel Widget - Phase G.2
///
/// Container widget that can hold other widgets

use crate::ui::widget::{Widget, Padding};
use crate::ui::event::{InputEvent, EventResponse, SizeConstraints, Size};
use crate::ui::theme::Theme;
use crate::graphics::{DrawContext, Rect, Font, Color};

/// Panel widget - simple container with background
pub struct Panel {
    bg_color: Option<Color>,
    border_color: Option<Color>,
    border_width: u32,
    padding: Padding,
    min_width: u32,
    min_height: u32,
}

impl Panel {
    /// Create a new panel
    pub fn new() -> Self {
        Self {
            bg_color: None,
            border_color: None,
            border_width: 0,
            padding: Padding::all(8),
            min_width: 50,
            min_height: 50,
        }
    }

    /// Set background color
    pub fn with_bg_color(mut self, color: Color) -> Self {
        self.bg_color = Some(color);
        self
    }

    /// Set border
    pub fn with_border(mut self, color: Color, width: u32) -> Self {
        self.border_color = Some(color);
        self.border_width = width;
        self
    }

    /// Set padding
    pub fn with_padding(mut self, padding: Padding) -> Self {
        self.padding = padding;
        self
    }

    /// Set minimum size
    pub fn with_min_size(mut self, width: u32, height: u32) -> Self {
        self.min_width = width;
        self.min_height = height;
        self
    }

    /// Get content bounds (excluding padding)
    pub fn content_bounds(&self, bounds: Rect) -> Rect {
        Rect::new(
            bounds.x + self.padding.left,
            bounds.y + self.padding.top,
            bounds.width.saturating_sub(self.padding.horizontal()),
            bounds.height.saturating_sub(self.padding.vertical()),
        )
    }
}

impl Default for Panel {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Panel {
    fn draw(&self, ctx: &mut DrawContext, bounds: Rect, theme: &Theme, font: &Font) {
        // Draw background
        let bg_color = self.bg_color.unwrap_or(theme.bg_secondary);
        ctx.fill_rect(bounds, bg_color);

        // Draw border if specified
        if let Some(border_color) = self.border_color {
            if self.border_width > 0 {
                ctx.draw_rect_outline(bounds, border_color, self.border_width);
            }
        }
    }

    fn handle_event(&mut self, event: &InputEvent, bounds: Rect) -> EventResponse {
        // Panel doesn't handle events itself, children will handle
        EventResponse::Ignored
    }

    fn preferred_size(&self, constraints: &SizeConstraints, font: &Font) -> Size {
        let width = self.min_width + self.padding.horizontal();
        let height = self.min_height + self.padding.vertical();

        let (width, height) = constraints.constrain(width, height);
        Size::new(width, height)
    }
}
