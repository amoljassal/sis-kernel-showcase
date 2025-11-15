/// Label Widget - Phase G.2
///
/// Displays static text

use crate::ui::widget::{Widget, Padding};
use crate::ui::event::{InputEvent, EventResponse, SizeConstraints, Size};
use crate::ui::theme::Theme;
use crate::graphics::{DrawContext, Rect, Font, Color};
use alloc::string::String;

/// Label widget for displaying text
pub struct Label {
    text: String,
    color: Option<Color>,
    padding: Padding,
    alignment: TextAlignment,
}

/// Text alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
}

impl Label {
    /// Create a new label
    pub fn new(text: String) -> Self {
        Self {
            text,
            color: None,
            padding: Padding::all(4),
            alignment: TextAlignment::Left,
        }
    }

    /// Set text color
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Set padding
    pub fn with_padding(mut self, padding: Padding) -> Self {
        self.padding = padding;
        self
    }

    /// Set alignment
    pub fn with_alignment(mut self, alignment: TextAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Set text content
    pub fn set_text(&mut self, text: String) {
        self.text = text;
    }

    /// Get text content
    pub fn text(&self) -> &str {
        &self.text
    }
}

impl Widget for Label {
    fn draw(&self, ctx: &mut DrawContext, bounds: Rect, theme: &Theme, font: &Font) {
        let color = self.color.unwrap_or(theme.text_primary);

        // Measure text
        let (text_width, text_height) = ctx.measure_text(&self.text, font);

        // Calculate text position based on alignment
        let x = match self.alignment {
            TextAlignment::Left => bounds.x + self.padding.left,
            TextAlignment::Center => {
                let available_width = bounds.width.saturating_sub(self.padding.horizontal());
                bounds.x + self.padding.left + (available_width.saturating_sub(text_width)) / 2
            }
            TextAlignment::Right => {
                let available_width = bounds.width.saturating_sub(self.padding.horizontal());
                bounds.x + self.padding.left + available_width.saturating_sub(text_width)
            }
        };

        let y = bounds.y + self.padding.top + (bounds.height.saturating_sub(self.padding.vertical() + text_height)) / 2;

        // Draw text
        ctx.draw_text(x, y, &self.text, font, color);
    }

    fn handle_event(&mut self, event: &InputEvent, bounds: Rect) -> EventResponse {
        EventResponse::Ignored
    }

    fn preferred_size(&self, constraints: &SizeConstraints, font: &Font) -> Size {
        // Measure text size
        let (text_width, text_height) = font.measure_text(&self.text);

        let width = text_width + self.padding.horizontal();
        let height = text_height + self.padding.vertical();

        let (width, height) = constraints.constrain(width, height);
        Size::new(width, height)
    }
}
