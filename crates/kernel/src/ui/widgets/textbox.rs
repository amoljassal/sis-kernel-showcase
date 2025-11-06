/// TextBox Widget - Phase G.2
///
/// Single-line text input widget

use crate::ui::widget::{Widget, WidgetState, Padding, is_point_in_bounds};
use crate::ui::event::{InputEvent, EventResponse, SizeConstraints, Size, MouseButton, KeyCode};
use crate::ui::theme::Theme;
use crate::graphics::{DrawContext, Rect, Font, Color};
use alloc::string::String;

/// TextBox widget for single-line text input
pub struct TextBox {
    text: String,
    placeholder: String,
    state: WidgetState,
    padding: Padding,
    cursor_pos: usize,
    focused: bool,
    max_length: Option<usize>,
}

impl TextBox {
    /// Create a new textbox
    pub fn new() -> Self {
        Self {
            text: String::new(),
            placeholder: String::from("Enter text..."),
            state: WidgetState::Normal,
            padding: Padding::symmetric(8, 6),
            cursor_pos: 0,
            focused: false,
            max_length: None,
        }
    }

    /// Set placeholder text
    pub fn with_placeholder(mut self, placeholder: String) -> Self {
        self.placeholder = placeholder;
        self
    }

    /// Set initial text
    pub fn with_text(mut self, text: String) -> Self {
        self.cursor_pos = text.len();
        self.text = text;
        self
    }

    /// Set maximum length
    pub fn with_max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
        self
    }

    /// Get text content
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Set text content
    pub fn set_text(&mut self, text: String) {
        self.text = text;
        self.cursor_pos = self.text.len();
    }

    /// Clear text
    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor_pos = 0;
    }
}

impl Default for TextBox {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for TextBox {
    fn draw(&self, ctx: &mut DrawContext, bounds: Rect, theme: &Theme, font: &Font) {
        // Draw background
        ctx.fill_rect(bounds, theme.input_bg);

        // Draw border
        let border_color = if self.focused {
            theme.input_border_focus
        } else {
            theme.input_border
        };
        ctx.draw_rect_outline(bounds, border_color, theme.border_width);

        // Draw text or placeholder
        let display_text = if self.text.is_empty() {
            &self.placeholder
        } else {
            &self.text
        };

        let text_color = if self.text.is_empty() {
            theme.text_secondary
        } else {
            theme.input_text
        };

        let x = bounds.x + self.padding.left;
        let y = bounds.y + self.padding.top + 2;

        ctx.draw_text(x, y, display_text, font, text_color);

        // Draw cursor if focused
        if self.focused {
            let cursor_text = &self.text[..self.cursor_pos.min(self.text.len())];
            let (cursor_x_offset, _) = font.measure_text(cursor_text);
            let cursor_x = x + cursor_x_offset;
            let cursor_y = bounds.y + self.padding.top;
            let cursor_height = font.line_height();

            // Draw cursor line
            ctx.fill_rect(
                Rect::new(cursor_x, cursor_y, 2, cursor_height),
                theme.accent
            );
        }
    }

    fn handle_event(&mut self, event: &InputEvent, bounds: Rect) -> EventResponse {
        match event {
            InputEvent::MouseButton { button: MouseButton::Left, pressed: true, x, y } => {
                if is_point_in_bounds(*x, *y, &bounds) {
                    self.focused = true;
                    self.state = WidgetState::Focused;
                    EventResponse::NeedsRedraw
                } else {
                    if self.focused {
                        self.focused = false;
                        self.state = WidgetState::Normal;
                        EventResponse::NeedsRedraw
                    } else {
                        EventResponse::Ignored
                    }
                }
            }

            InputEvent::TextInput { character } if self.focused => {
                // Check max length
                if let Some(max) = self.max_length {
                    if self.text.len() >= max {
                        return EventResponse::Consumed;
                    }
                }

                // Insert character at cursor position
                if character.is_ascii() || character.is_alphanumeric() {
                    self.text.insert(self.cursor_pos, *character);
                    self.cursor_pos += 1;
                    EventResponse::NeedsRedraw
                } else {
                    EventResponse::Ignored
                }
            }

            InputEvent::KeyPress { key, .. } if self.focused => {
                match key {
                    KeyCode::Backspace => {
                        if self.cursor_pos > 0 {
                            self.cursor_pos -= 1;
                            self.text.remove(self.cursor_pos);
                            EventResponse::NeedsRedraw
                        } else {
                            EventResponse::Consumed
                        }
                    }

                    KeyCode::Delete => {
                        if self.cursor_pos < self.text.len() {
                            self.text.remove(self.cursor_pos);
                            EventResponse::NeedsRedraw
                        } else {
                            EventResponse::Consumed
                        }
                    }

                    KeyCode::Left => {
                        if self.cursor_pos > 0 {
                            self.cursor_pos -= 1;
                            EventResponse::NeedsRedraw
                        } else {
                            EventResponse::Consumed
                        }
                    }

                    KeyCode::Right => {
                        if self.cursor_pos < self.text.len() {
                            self.cursor_pos += 1;
                            EventResponse::NeedsRedraw
                        } else {
                            EventResponse::Consumed
                        }
                    }

                    KeyCode::Home => {
                        self.cursor_pos = 0;
                        EventResponse::NeedsRedraw
                    }

                    KeyCode::End => {
                        self.cursor_pos = self.text.len();
                        EventResponse::NeedsRedraw
                    }

                    _ => EventResponse::Ignored,
                }
            }

            _ => EventResponse::Ignored,
        }
    }

    fn preferred_size(&self, constraints: &SizeConstraints, font: &Font) -> Size {
        let text_height = font.line_height();
        let width = 200; // Default width
        let height = text_height + self.padding.vertical();

        let (width, height) = constraints.constrain(width, height);
        Size::new(width, height)
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn on_focus(&mut self) {
        self.focused = true;
        self.state = WidgetState::Focused;
    }

    fn on_blur(&mut self) {
        self.focused = false;
        self.state = WidgetState::Normal;
    }
}
