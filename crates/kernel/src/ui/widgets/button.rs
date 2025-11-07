/// Button Widget - Phase G.2
///
/// Interactive button with click handling

use crate::ui::widget::{Widget, WidgetState, Padding, is_point_in_bounds};
use crate::ui::event::{InputEvent, EventResponse, SizeConstraints, Size, MouseButton};
use crate::ui::theme::Theme;
use crate::graphics::{DrawContext, Rect, Font, Color};
use alloc::string::String;
use alloc::boxed::Box;

/// Button widget
pub struct Button {
    text: String,
    state: WidgetState,
    padding: Padding,
    on_click: Option<Box<dyn Fn() + Send + Sync>>,
    enabled: bool,
}

impl Button {
    /// Create a new button
    pub fn new(text: String) -> Self {
        Self {
            text,
            state: WidgetState::Normal,
            padding: Padding::symmetric(16, 8),
            on_click: None,
            enabled: true,
        }
    }

    /// Set click handler
    pub fn on_click<F>(mut self, handler: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_click = Some(Box::new(handler));
        self
    }

    /// Set enabled state
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.state = WidgetState::Disabled;
        } else if self.state == WidgetState::Disabled {
            self.state = WidgetState::Normal;
        }
    }

    /// Get text
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Set text
    pub fn set_text(&mut self, text: String) {
        self.text = text;
    }
}

impl Widget for Button {
    fn draw(&self, ctx: &mut DrawContext, bounds: Rect, theme: &Theme, font: &Font) {
        // Choose background color based on state
        let bg_color = if !self.enabled {
            theme.bg_tertiary
        } else {
            match self.state {
                WidgetState::Normal => theme.button_bg,
                WidgetState::Hovered => theme.button_bg_hover,
                WidgetState::Pressed => theme.button_bg_pressed,
                WidgetState::Focused => theme.button_bg_hover,
                WidgetState::Disabled => theme.bg_tertiary,
            }
        };

        // Draw background
        ctx.fill_rect(bounds, bg_color);

        // Draw border
        let border_color = if self.state == WidgetState::Focused {
            theme.border_focus
        } else {
            theme.border
        };
        ctx.draw_rect_outline(bounds, border_color, theme.border_width);

        // Draw text
        let text_color = if self.enabled {
            theme.button_text
        } else {
            theme.text_disabled
        };

        let (text_width, text_height) = ctx.measure_text(&self.text, font);
        let x = bounds.x + (bounds.width.saturating_sub(text_width)) / 2;
        let y = bounds.y + (bounds.height.saturating_sub(text_height)) / 2;

        ctx.draw_text(x, y, &self.text, font, text_color);
    }

    fn handle_event(&mut self, event: &InputEvent, bounds: Rect) -> EventResponse {
        if !self.enabled {
            return EventResponse::Ignored;
        }

        match event {
            InputEvent::MouseMove { x, y } => {
                let in_bounds = is_point_in_bounds(*x, *y, &bounds);

                let old_state = self.state;
                if in_bounds && self.state != WidgetState::Pressed {
                    self.state = WidgetState::Hovered;
                } else if !in_bounds && self.state == WidgetState::Hovered {
                    self.state = WidgetState::Normal;
                }

                if old_state != self.state {
                    EventResponse::NeedsRedraw
                } else {
                    EventResponse::Ignored
                }
            }

            InputEvent::MouseButton { button: MouseButton::Left, pressed: true, x, y } => {
                if is_point_in_bounds(*x, *y, &bounds) {
                    self.state = WidgetState::Pressed;
                    EventResponse::NeedsRedraw
                } else {
                    EventResponse::Ignored
                }
            }

            InputEvent::MouseButton { button: MouseButton::Left, pressed: false, x, y } => {
                if self.state == WidgetState::Pressed {
                    self.state = if is_point_in_bounds(*x, *y, &bounds) {
                        WidgetState::Hovered
                    } else {
                        WidgetState::Normal
                    };

                    // Trigger click handler if released over button
                    if is_point_in_bounds(*x, *y, &bounds) {
                        if let Some(ref handler) = self.on_click {
                            handler();
                        }
                        return EventResponse::Consumed;
                    }

                    EventResponse::NeedsRedraw
                } else {
                    EventResponse::Ignored
                }
            }

            _ => EventResponse::Ignored,
        }
    }

    fn preferred_size(&self, constraints: &SizeConstraints, font: &Font) -> Size {
        let (text_width, text_height) = font.measure_text(&self.text);

        let width = text_width + self.padding.horizontal();
        let height = text_height + self.padding.vertical();

        let (width, height) = constraints.constrain(width, height);
        Size::new(width, height)
    }

    fn can_focus(&self) -> bool {
        self.enabled
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn on_focus(&mut self) {
        if self.enabled {
            self.state = WidgetState::Focused;
        }
    }

    fn on_blur(&mut self) {
        if self.state == WidgetState::Focused {
            self.state = WidgetState::Normal;
        }
    }
}
