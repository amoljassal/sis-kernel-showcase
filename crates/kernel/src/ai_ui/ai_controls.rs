/// AI Controls Widget - Phase G.4
///
/// Interactive controls for configuring kernel AI subsystems

use crate::ui::{Widget, EventResponse, InputEvent, SizeConstraints, Size, Theme};
use crate::ui::widget::is_point_in_bounds;
use crate::graphics::{DrawContext, Rect, Font, Color};
use spin::Mutex;

/// AI subsystem configuration
#[derive(Debug, Clone)]
pub struct AIConfig {
    pub memory_prediction_enabled: bool,
    pub scheduling_optimization_enabled: bool,
    pub load_balancing_enabled: bool,
    pub cache_optimization_enabled: bool,
    pub prefetch_enabled: bool,
    pub confidence_threshold: u8,  // 0-100
}

impl Default for AIConfig {
    fn default() -> Self {
        Self {
            memory_prediction_enabled: true,
            scheduling_optimization_enabled: true,
            load_balancing_enabled: true,
            cache_optimization_enabled: true,
            prefetch_enabled: true,
            confidence_threshold: 50,
        }
    }
}

/// Global AI configuration
static AI_CONFIG: Mutex<AIConfig> = Mutex::new(AIConfig {
    memory_prediction_enabled: true,
    scheduling_optimization_enabled: true,
    load_balancing_enabled: true,
    cache_optimization_enabled: true,
    prefetch_enabled: true,
    confidence_threshold: 50,
});

/// Get current AI configuration
pub fn get_ai_config() -> AIConfig {
    AI_CONFIG.lock().clone()
}

/// Update AI configuration
pub fn set_ai_config(config: AIConfig) {
    *AI_CONFIG.lock() = config;
}

/// Toggle a specific AI feature
pub fn toggle_ai_feature(feature: &str) {
    let mut config = AI_CONFIG.lock();
    match feature {
        "memory_prediction" => config.memory_prediction_enabled = !config.memory_prediction_enabled,
        "scheduling" => config.scheduling_optimization_enabled = !config.scheduling_optimization_enabled,
        "load_balancing" => config.load_balancing_enabled = !config.load_balancing_enabled,
        "cache_optimization" => config.cache_optimization_enabled = !config.cache_optimization_enabled,
        "prefetch" => config.prefetch_enabled = !config.prefetch_enabled,
        _ => {}
    }
}

/// Set confidence threshold
pub fn set_confidence_threshold(threshold: u8) {
    AI_CONFIG.lock().confidence_threshold = threshold.min(100);
}

/// AI Controls widget
pub struct AIControls {
    hover_item: Option<usize>,
    threshold_dragging: bool,
}

impl AIControls {
    /// Create a new AI controls widget
    pub fn new() -> Self {
        Self {
            hover_item: None,
            threshold_dragging: false,
        }
    }

    /// Draw a checkbox toggle
    fn draw_toggle(
        &self,
        ctx: &mut DrawContext,
        x: u32,
        y: u32,
        label: &str,
        enabled: bool,
        hovered: bool,
        font: &Font,
        theme: &Theme,
    ) -> Rect {
        let box_size = 16u32;
        let box_rect = Rect::new(x, y, box_size, box_size);

        // Draw checkbox background
        let bg_color = if hovered {
            Color::from_rgb(60, 60, 65)
        } else {
            theme.bg_tertiary
        };
        ctx.fill_rect(box_rect, bg_color);

        // Draw checkbox border
        ctx.draw_rect_outline(box_rect, theme.border, 1);

        // Draw checkmark if enabled
        if enabled {
            ctx.fill_rect(
                Rect::new(x + 3, y + 3, box_size - 6, box_size - 6),
                theme.accent
            );
        }

        // Draw label
        let text_color = if enabled {
            theme.text_primary
        } else {
            theme.text_secondary
        };
        ctx.draw_text(x + box_size + 10, y + 2, label, font, text_color);

        box_rect
    }

    /// Draw a slider control
    fn draw_slider(
        &self,
        ctx: &mut DrawContext,
        x: u32,
        y: u32,
        width: u32,
        label: &str,
        value: u8,
        font: &Font,
        theme: &Theme,
    ) -> Rect {
        // Draw label
        ctx.draw_text(x, y, label, font, theme.text_primary);

        let slider_y = y + 20;
        let slider_height = 8u32;
        let slider_rect = Rect::new(x, slider_y, width, slider_height);

        // Draw slider track
        ctx.fill_rect(slider_rect, theme.bg_tertiary);
        ctx.draw_rect_outline(slider_rect, theme.border, 1);

        // Draw slider fill
        let fill_width = (width as f32 * (value as f32 / 100.0)) as u32;
        ctx.fill_rect(
            Rect::new(x, slider_y, fill_width, slider_height),
            theme.accent
        );

        // Draw slider handle
        let handle_x = x + fill_width - 4;
        let handle_rect = Rect::new(handle_x, slider_y - 4, 8, slider_height + 8);
        ctx.fill_rect(handle_rect, Color::from_rgb(255, 255, 255));
        ctx.draw_rect_outline(handle_rect, theme.border, 1);

        // Draw value
        let value_text = alloc::format!("{}%", value);
        ctx.draw_text(x + width + 10, slider_y - 2, &value_text, font, theme.text_secondary);

        Rect::new(x, y, width + 60, 30)
    }

    /// Get the bounds of interactive elements
    fn get_toggle_bounds(&self, bounds: Rect, index: usize) -> Rect {
        let x = bounds.x + 15;
        let y = bounds.y + 45 + (index as u32 * 30);
        Rect::new(x, y, 200, 20)
    }

    /// Get slider bounds
    fn get_slider_bounds(&self, bounds: Rect) -> Rect {
        let x = bounds.x + 15;
        let y = bounds.y + 45 + (5 * 30) + 20;
        Rect::new(x, y, 250, 30)
    }
}

impl Default for AIControls {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for AIControls {
    fn draw(&self, ctx: &mut DrawContext, bounds: Rect, theme: &Theme, font: &Font) {
        // Draw background
        ctx.fill_rect(bounds, Color::from_rgb(25, 25, 30));

        // Draw title
        ctx.draw_text(
            bounds.x + 10,
            bounds.y + 10,
            "AI System Controls",
            font,
            theme.accent
        );

        let config = get_ai_config();

        let x = bounds.x + 15;
        let mut y = bounds.y + 45;

        // Draw toggles for each AI feature
        let features = [
            ("Memory Prediction", config.memory_prediction_enabled),
            ("Scheduling Optimization", config.scheduling_optimization_enabled),
            ("Load Balancing", config.load_balancing_enabled),
            ("Cache Optimization", config.cache_optimization_enabled),
            ("Prefetch System", config.prefetch_enabled),
        ];

        for (index, (label, enabled)) in features.iter().enumerate() {
            let hovered = self.hover_item == Some(index);
            self.draw_toggle(ctx, x, y, label, *enabled, hovered, font, theme);
            y += 30;
        }

        y += 20;

        // Draw separator
        ctx.fill_rect(
            Rect::new(bounds.x + 10, y, bounds.width - 20, 1),
            theme.border
        );

        y += 15;

        // Draw confidence threshold slider
        self.draw_slider(
            ctx,
            x,
            y,
            200,
            "Confidence Threshold",
            config.confidence_threshold,
            font,
            theme,
        );

        y += 50;

        // Draw info text
        ctx.draw_text(
            x,
            y,
            "Lower threshold = more aggressive AI",
            font,
            Color::from_rgb(150, 150, 150)
        );

        y += 20;

        ctx.draw_text(
            x,
            y,
            "Higher threshold = more conservative AI",
            font,
            Color::from_rgb(150, 150, 150)
        );

        // Draw border
        ctx.draw_rect_outline(bounds, theme.border, 1);
    }

    fn handle_event(&mut self, event: &InputEvent, bounds: Rect) -> EventResponse {
        use crate::ui::event::MouseButton;

        match event {
            InputEvent::MouseMove { x, y } => {
                // Check if hovering over any toggle
                let mut found_hover = false;
                for i in 0..5 {
                    let toggle_bounds = self.get_toggle_bounds(bounds, i);
                    if is_point_in_bounds(*x, *y, &toggle_bounds) {
                        self.hover_item = Some(i);
                        found_hover = true;
                        break;
                    }
                }

                if !found_hover {
                    if self.hover_item.is_some() {
                        self.hover_item = None;
                        return EventResponse::NeedsRedraw;
                    }
                } else {
                    return EventResponse::NeedsRedraw;
                }

                // Check if dragging threshold slider
                if self.threshold_dragging {
                    let slider_bounds = self.get_slider_bounds(bounds);
                    if *x >= slider_bounds.x && *x <= slider_bounds.x + 200 {
                        let slider_x = slider_bounds.x;
                        let offset = *x - slider_x;
                        let threshold = ((offset as f32 / 200.0) * 100.0).min(100.0).max(0.0) as u8;
                        set_confidence_threshold(threshold);
                        return EventResponse::NeedsRedraw;
                    }
                }

                EventResponse::Ignored
            }

            InputEvent::MouseButton { button: MouseButton::Left, pressed: true, x, y } => {
                // Check toggle clicks
                for i in 0..5 {
                    let toggle_bounds = self.get_toggle_bounds(bounds, i);
                    if is_point_in_bounds(*x, *y, &toggle_bounds) {
                        let feature = match i {
                            0 => "memory_prediction",
                            1 => "scheduling",
                            2 => "load_balancing",
                            3 => "cache_optimization",
                            4 => "prefetch",
                            _ => "",
                        };
                        toggle_ai_feature(feature);
                        return EventResponse::NeedsRedraw;
                    }
                }

                // Check slider click
                let slider_bounds = self.get_slider_bounds(bounds);
                if is_point_in_bounds(*x, *y, &slider_bounds) {
                    self.threshold_dragging = true;
                    let slider_x = slider_bounds.x;
                    let offset = *x - slider_x;
                    let threshold = ((offset as f32 / 200.0) * 100.0).min(100.0).max(0.0) as u8;
                    set_confidence_threshold(threshold);
                    return EventResponse::NeedsRedraw;
                }

                EventResponse::Ignored
            }

            InputEvent::MouseButton { button: MouseButton::Left, pressed: false, .. } => {
                if self.threshold_dragging {
                    self.threshold_dragging = false;
                    return EventResponse::NeedsRedraw;
                }
                EventResponse::Ignored
            }

            _ => EventResponse::Ignored,
        }
    }

    fn preferred_size(&self, constraints: &SizeConstraints, font: &Font) -> Size {
        Size::new(400, 360)
    }
}
