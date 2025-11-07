/// AI Insights Dashboard - Phase G.3
///
/// Displays kernel AI metrics and decision visualization

use crate::ui::{Widget, EventResponse, InputEvent, SizeConstraints, Size, Theme};
use crate::ui::{VStack, Label, Panel, Padding, TextAlignment};
use crate::graphics::{DrawContext, Rect, Font, Color};
use alloc::string::String;
use alloc::boxed::Box;

/// AI Insights dashboard application
pub struct AIInsightsApp {
    last_update: u64,
}

impl AIInsightsApp {
    /// Create a new AI insights dashboard
    pub fn new() -> Self {
        Self {
            last_update: 0,
        }
    }

    /// Get current CPU count (from SMP subsystem)
    fn get_cpu_count(&self) -> usize {
        crate::smp::num_cpus()
    }

    /// Get memory statistics
    fn get_memory_stats(&self) -> (usize, usize) {
        let stats = crate::mm::get_stats().unwrap_or_default();
        (stats.total_pages, stats.free_pages)
    }

    /// Get process count
    fn get_process_count(&self) -> usize {
        // Placeholder - would get from process table
        crate::process::count_processes()
    }

    /// Draw a metric bar
    fn draw_metric_bar(&self, ctx: &mut DrawContext, bounds: Rect, label: &str, value: usize, max: usize, color: Color, font: &Font, theme: &Theme) {
        // Draw label
        ctx.draw_text(bounds.x, bounds.y, label, font, theme.text_primary);

        // Draw bar background
        let bar_y = bounds.y + 20;
        let bar_height = 20;
        ctx.fill_rect(
            Rect::new(bounds.x, bar_y, bounds.width, bar_height),
            theme.bg_tertiary
        );

        // Draw bar fill
        let fill_width = if max > 0 {
            ((value as f32 / max as f32) * bounds.width as f32) as u32
        } else {
            0
        };

        ctx.fill_rect(
            Rect::new(bounds.x, bar_y, fill_width, bar_height),
            color
        );

        // Draw value text
        let value_text = alloc::format!("{} / {}", value, max);
        let (text_width, _) = ctx.measure_text(&value_text, font);
        let text_x = bounds.x + (bounds.width - text_width) / 2;
        ctx.draw_text(text_x, bar_y + 5, &value_text, font, Color::WHITE);
    }
}

impl Default for AIInsightsApp {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for AIInsightsApp {
    fn draw(&self, ctx: &mut DrawContext, bounds: Rect, theme: &Theme, font: &Font) {
        // Draw background with gradient
        for y in 0..bounds.height {
            let intensity = (y as f32 / bounds.height as f32 * 60.0) as u8;
            let color = Color::from_rgb(intensity / 3, intensity / 2, intensity);
            ctx.fill_rect(
                Rect::new(bounds.x, bounds.y + y, bounds.width, 1),
                color
            );
        }

        // Draw title
        let title = "AI Insights Dashboard";
        let (title_width, _) = ctx.measure_text(title, font);
        let title_x = bounds.x + (bounds.width - title_width) / 2;
        ctx.draw_text(title_x, bounds.y + 20, title, font, theme.accent);

        // Draw subtitle
        let subtitle = "Real-time Kernel Metrics";
        let (subtitle_width, _) = ctx.measure_text(subtitle, font);
        let subtitle_x = bounds.x + (bounds.width - subtitle_width) / 2;
        ctx.draw_text(subtitle_x, bounds.y + 45, subtitle, font, theme.text_secondary);

        // Get metrics
        let cpu_count = self.get_cpu_count();
        let (total_mem, free_mem) = self.get_memory_stats();
        let used_mem = total_mem.saturating_sub(free_mem);
        let process_count = self.get_process_count();

        // Draw metrics section
        let metrics_y = bounds.y + 80;
        let metric_height = 50;
        let padding = 20;

        // CPU metric
        self.draw_metric_bar(
            ctx,
            Rect::new(bounds.x + padding, metrics_y, bounds.width - padding * 2, metric_height),
            "CPU Cores",
            cpu_count,
            8, // Max 8 cores
            Color::from_rgb(0, 200, 100),
            font,
            theme
        );

        // Memory metric
        self.draw_metric_bar(
            ctx,
            Rect::new(bounds.x + padding, metrics_y + metric_height + 20, bounds.width - padding * 2, metric_height),
            "Memory Usage (Pages)",
            used_mem,
            total_mem,
            Color::from_rgb(0, 150, 255),
            font,
            theme
        );

        // Process metric
        self.draw_metric_bar(
            ctx,
            Rect::new(bounds.x + padding, metrics_y + (metric_height + 20) * 2, bounds.width - padding * 2, metric_height),
            "Active Processes",
            process_count,
            64, // Max 64 processes
            Color::from_rgb(255, 150, 0),
            font,
            theme
        );

        // Draw AI decision info
        let info_y = metrics_y + (metric_height + 20) * 3 + 20;

        ctx.draw_text(
            bounds.x + padding,
            info_y,
            "AI Features Active:",
            font,
            theme.text_primary
        );

        ctx.draw_text(
            bounds.x + padding,
            info_y + 25,
            "  - Predictive Memory Management",
            font,
            Color::from_rgb(0, 255, 100)
        );

        ctx.draw_text(
            bounds.x + padding,
            info_y + 45,
            "  - SMP Load Balancing",
            font,
            Color::from_rgb(0, 255, 100)
        );

        ctx.draw_text(
            bounds.x + padding,
            info_y + 65,
            "  - Adaptive Scheduling",
            font,
            Color::from_rgb(0, 255, 100)
        );

        // Draw border
        ctx.draw_rect_outline(bounds, theme.accent, 2);
    }

    fn handle_event(&mut self, event: &InputEvent, bounds: Rect) -> EventResponse {
        EventResponse::Ignored
    }

    fn preferred_size(&self, constraints: &SizeConstraints, font: &Font) -> Size {
        Size::new(500, 450)
    }
}
