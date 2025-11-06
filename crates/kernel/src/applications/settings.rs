/// System Settings Application - Phase G.3
///
/// Configuration UI for system preferences

use crate::ui::{Widget, EventResponse, InputEvent, SizeConstraints, Size, Theme, MouseButton};
use crate::ui::{VStack, Label, Button, Panel, Padding, TextAlignment};
use crate::graphics::{DrawContext, Rect, Font, Color};
use alloc::string::String;
use alloc::boxed::Box;

/// Settings category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SettingsCategory {
    Display,
    System,
    About,
}

/// System Settings application
pub struct SettingsApp {
    current_category: SettingsCategory,
    theme_is_dark: bool,
}

impl SettingsApp {
    /// Create a new settings app
    pub fn new() -> Self {
        Self {
            current_category: SettingsCategory::Display,
            theme_is_dark: true,
        }
    }

    /// Draw category buttons
    fn draw_categories(&self, ctx: &mut DrawContext, bounds: Rect, theme: &Theme, font: &Font) {
        let button_height = 35;
        let button_spacing = 5;
        let mut y = bounds.y + 10;

        let categories = [
            ("Display", SettingsCategory::Display),
            ("System", SettingsCategory::System),
            ("About", SettingsCategory::About),
        ];

        for (name, category) in categories.iter() {
            let is_active = self.current_category == *category;
            let bg_color = if is_active {
                theme.accent
            } else {
                theme.button_bg
            };

            let button_rect = Rect::new(bounds.x + 10, y, 120, button_height);

            // Draw button
            ctx.fill_rect(button_rect, bg_color);
            ctx.draw_rect_outline(button_rect, theme.border, 1);

            // Draw text
            let (text_width, _) = ctx.measure_text(name, font);
            let text_x = button_rect.x + (button_rect.width - text_width) / 2;
            let text_y = button_rect.y + 10;
            ctx.draw_text(text_x, text_y, name, font, Color::WHITE);

            y += button_height + button_spacing;
        }
    }

    /// Draw display settings
    fn draw_display_settings(&self, ctx: &mut DrawContext, bounds: Rect, theme: &Theme, font: &Font) {
        let mut y = bounds.y + 10;

        ctx.draw_text(bounds.x, y, "Display Settings", font, theme.accent);
        y += 30;

        // Theme setting
        ctx.draw_text(bounds.x, y, "Theme:", font, theme.text_primary);
        y += 25;

        let theme_name = if self.theme_is_dark { "Dark" } else { "Light" };
        let toggle_rect = Rect::new(bounds.x + 20, y, 100, 30);

        ctx.fill_rect(toggle_rect, theme.button_bg);
        ctx.draw_rect_outline(toggle_rect, theme.border, 1);

        let (text_width, _) = ctx.measure_text(theme_name, font);
        ctx.draw_text(
            toggle_rect.x + (toggle_rect.width - text_width) / 2,
            toggle_rect.y + 8,
            theme_name,
            font,
            theme.text_primary
        );

        y += 50;

        // Resolution (read-only)
        ctx.draw_text(bounds.x, y, "Resolution: 1280 x 720", font, theme.text_secondary);
        y += 25;

        // Refresh rate (read-only)
        ctx.draw_text(bounds.x, y, "Refresh: 60 Hz", font, theme.text_secondary);
    }

    /// Draw system settings
    fn draw_system_settings(&self, ctx: &mut DrawContext, bounds: Rect, theme: &Theme, font: &Font) {
        let mut y = bounds.y + 10;

        ctx.draw_text(bounds.x, y, "System Settings", font, theme.accent);
        y += 30;

        // CPU info
        let cpu_count = crate::smp::num_cpus();
        ctx.draw_text(
            bounds.x,
            y,
            &alloc::format!("CPU Cores: {}", cpu_count),
            font,
            theme.text_primary
        );
        y += 25;

        // Memory info
        let stats = crate::mm::get_stats();
        let total_mb = (stats.total_pages * 4) / 1024; // 4KB pages to MB
        ctx.draw_text(
            bounds.x,
            y,
            &alloc::format!("Total Memory: {} MB", total_mb),
            font,
            theme.text_primary
        );
        y += 25;

        let free_mb = (stats.free_pages * 4) / 1024;
        ctx.draw_text(
            bounds.x,
            y,
            &alloc::format!("Free Memory: {} MB", free_mb),
            font,
            theme.text_primary
        );
        y += 35;

        // AI Features
        ctx.draw_text(bounds.x, y, "AI Features:", font, theme.accent);
        y += 25;

        ctx.draw_text(bounds.x + 10, y, "✓ Predictive Memory", font, Color::from_rgb(0, 255, 100));
        y += 20;

        ctx.draw_text(bounds.x + 10, y, "✓ SMP Load Balancing", font, Color::from_rgb(0, 255, 100));
        y += 20;

        ctx.draw_text(bounds.x + 10, y, "✓ Adaptive Scheduling", font, Color::from_rgb(0, 255, 100));
    }

    /// Draw about screen
    fn draw_about(&self, ctx: &mut DrawContext, bounds: Rect, theme: &Theme, font: &Font) {
        let mut y = bounds.y + 10;

        // Title
        let title = "SIS OS";
        let (title_width, _) = ctx.measure_text(title, font);
        ctx.draw_text((bounds.x + bounds.width / 2).saturating_sub(title_width / 2), y, title, font, theme.accent);
        y += 30;

        // Version
        let version = "Version: Phase G.3";
        let (ver_width, _) = ctx.measure_text(version, font);
        ctx.draw_text((bounds.x + bounds.width / 2).saturating_sub(ver_width / 2), y, version, font, theme.text_primary);
        y += 25;

        // Description
        let desc = "AI-Native Desktop Environment";
        let (desc_width, _) = ctx.measure_text(desc, font);
        ctx.draw_text((bounds.x + bounds.width / 2).saturating_sub(desc_width / 2), y, desc, font, theme.text_secondary);
        y += 40;

        // Features
        ctx.draw_text(bounds.x, y, "Features:", font, theme.accent);
        y += 25;

        let features = [
            "• Multi-core SMP support (Phase E)",
            "• ext4 journaling filesystem (Phase F)",
            "• virtio-gpu graphics (Phase G.0)",
            "• Window manager (Phase G.1)",
            "• UI toolkit (Phase G.2)",
            "• Core applications (Phase G.3)",
        ];

        for feature in features.iter() {
            ctx.draw_text(bounds.x + 10, y, feature, font, theme.text_secondary);
            y += 20;
        }

        y += 20;

        // Copyright
        let copyright = "© 2025 SIS OS Project";
        let (copy_width, _) = ctx.measure_text(copyright, font);
        ctx.draw_text((bounds.x + bounds.width / 2).saturating_sub(copy_width / 2), y, copyright, font, theme.text_disabled);
    }
}

impl Default for SettingsApp {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for SettingsApp {
    fn draw(&self, ctx: &mut DrawContext, bounds: Rect, theme: &Theme, font: &Font) {
        // Draw background
        ctx.fill_rect(bounds, theme.bg_secondary);

        // Draw sidebar
        let sidebar_width = 150;
        let sidebar = Rect::new(bounds.x, bounds.y, sidebar_width, bounds.height);
        ctx.fill_rect(sidebar, theme.bg_tertiary);

        self.draw_categories(ctx, sidebar, theme, font);

        // Draw content area
        let content = Rect::new(
            bounds.x + sidebar_width + 20,
            bounds.y + 20,
            bounds.width - sidebar_width - 40,
            bounds.height - 40
        );

        match self.current_category {
            SettingsCategory::Display => self.draw_display_settings(ctx, content, theme, font),
            SettingsCategory::System => self.draw_system_settings(ctx, content, theme, font),
            SettingsCategory::About => self.draw_about(ctx, content, theme, font),
        }

        // Draw border
        ctx.draw_rect_outline(bounds, theme.border, 1);
    }

    fn handle_event(&mut self, event: &InputEvent, bounds: Rect) -> EventResponse {
        match event {
            InputEvent::MouseButton { button: MouseButton::Left, pressed: true, x, y } => {
                // Check category buttons
                let button_height = 35;
                let button_spacing = 5;
                let mut button_y = bounds.y + 10;

                let categories = [
                    SettingsCategory::Display,
                    SettingsCategory::System,
                    SettingsCategory::About,
                ];

                for category in categories.iter() {
                    let button_rect = Rect::new(bounds.x + 10, button_y, 120, button_height);
                    if button_rect.contains(*x, *y) {
                        self.current_category = *category;
                        return EventResponse::NeedsRedraw;
                    }
                    button_y += button_height + button_spacing;
                }

                // Check theme toggle button (only in Display settings)
                if self.current_category == SettingsCategory::Display {
                    let toggle_rect = Rect::new(bounds.x + 170, bounds.y + 65, 100, 30);
                    if toggle_rect.contains(*x, *y) {
                        self.theme_is_dark = !self.theme_is_dark;
                        // Would trigger actual theme change in UI manager
                        return EventResponse::NeedsRedraw;
                    }
                }

                EventResponse::Consumed
            }
            _ => EventResponse::Ignored,
        }
    }

    fn preferred_size(&self, constraints: &SizeConstraints, font: &Font) -> Size {
        Size::new(550, 400)
    }
}
