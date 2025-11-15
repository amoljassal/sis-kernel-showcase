/// Scheduling Viewer Widget - Phase G.4
///
/// Visualizes AI-driven scheduling decisions and load balancing

use crate::ui::{Widget, EventResponse, InputEvent, SizeConstraints, Size, Theme};
use crate::graphics::{DrawContext, Rect, Font, Color};
use alloc::vec::Vec;
use alloc::string::String;

/// Scheduling decision entry
#[derive(Debug, Clone)]
pub struct SchedulingDecision {
    pub action: String,          // e.g., "Migrate task 42 from CPU 0 to CPU 2"
    pub reason: String,           // e.g., "Load imbalance detected"
    pub confidence: u8,           // 0-100
    pub performance_gain: i32,    // Expected performance improvement %
    pub timestamp: u64,
}

/// Scheduling statistics
#[derive(Debug, Clone, Copy)]
pub struct SchedulingStats {
    pub total_decisions: u64,
    pub migrations: u64,
    pub priority_adjustments: u64,
    pub load_balancing_events: u64,
    pub avg_cpu_utilization: u32,  // 0-100
    pub load_balance_score: u32,   // 0-100 (100 = perfectly balanced)
}

impl Default for SchedulingStats {
    fn default() -> Self {
        Self {
            total_decisions: 0,
            migrations: 0,
            priority_adjustments: 0,
            load_balancing_events: 0,
            avg_cpu_utilization: 0,
            load_balance_score: 100,
        }
    }
}

/// Get scheduling statistics from kernel
/// TODO: Integrate with actual predictive_scheduling subsystem
pub fn get_scheduling_stats() -> SchedulingStats {
    // Mock data for now - will integrate with predictive_scheduling module
    SchedulingStats {
        total_decisions: 8430,
        migrations: 234,
        priority_adjustments: 567,
        load_balancing_events: 89,
        avg_cpu_utilization: 65,
        load_balance_score: 92,
    }
}

/// Get recent scheduling decisions
/// TODO: Integrate with actual predictive_scheduling subsystem
pub fn get_recent_scheduling_decisions(count: usize) -> Vec<SchedulingDecision> {
    // Mock data for now
    let mut decisions = Vec::new();

    decisions.push(SchedulingDecision {
        action: String::from("Migrate task 42 → CPU 2"),
        reason: String::from("Load imbalance"),
        confidence: 87,
        performance_gain: 15,
        timestamp: 1000,
    });

    decisions.push(SchedulingDecision {
        action: String::from("Boost priority: task 17"),
        reason: String::from("I/O wait detected"),
        confidence: 92,
        performance_gain: 8,
        timestamp: 950,
    });

    decisions.push(SchedulingDecision {
        action: String::from("Migrate task 28 → CPU 1"),
        reason: String::from("Cache affinity"),
        confidence: 78,
        performance_gain: 12,
        timestamp: 900,
    });

    decisions.push(SchedulingDecision {
        action: String::from("Reduce priority: task 5"),
        reason: String::from("CPU hog detected"),
        confidence: 95,
        performance_gain: 20,
        timestamp: 850,
    });

    decisions.truncate(count);
    decisions
}

/// Scheduling Viewer widget
pub struct SchedulingViewer {
    max_visible: usize,
    show_stats: bool,
}

impl SchedulingViewer {
    /// Create a new scheduling viewer
    pub fn new(max_visible: usize) -> Self {
        Self {
            max_visible,
            show_stats: true,
        }
    }

    /// Get color for confidence level
    fn confidence_color(confidence: u8) -> Color {
        if confidence >= 80 {
            Color::from_rgb(0, 255, 100) // High confidence - green
        } else if confidence >= 50 {
            Color::from_rgb(255, 200, 0) // Medium confidence - yellow
        } else {
            Color::from_rgb(255, 100, 0) // Low confidence - orange
        }
    }

    /// Get color for performance gain
    fn gain_color(gain: i32) -> Color {
        if gain >= 15 {
            Color::from_rgb(0, 255, 100) // High gain - green
        } else if gain >= 5 {
            Color::from_rgb(100, 200, 255) // Medium gain - blue
        } else {
            Color::from_rgb(200, 200, 200) // Low gain - gray
        }
    }

    /// Draw a compact stat
    fn draw_stat(
        &self,
        ctx: &mut DrawContext,
        x: u32,
        y: u32,
        label: &str,
        value: &str,
        color: Color,
        font: &Font,
        theme: &Theme,
    ) {
        ctx.draw_text(x, y, label, font, theme.text_secondary);
        ctx.draw_text(x + 150, y, value, font, color);
    }
}

impl Default for SchedulingViewer {
    fn default() -> Self {
        Self::new(5)
    }
}

impl Widget for SchedulingViewer {
    fn draw(&self, ctx: &mut DrawContext, bounds: Rect, theme: &Theme, font: &Font) {
        // Draw background
        ctx.fill_rect(bounds, Color::from_rgb(25, 25, 30));

        // Draw title
        ctx.draw_text(
            bounds.x + 10,
            bounds.y + 10,
            "AI Scheduling Decisions",
            font,
            theme.accent
        );

        let mut y = bounds.y + 35;
        let x = bounds.x + 10;

        if self.show_stats {
            let stats = get_scheduling_stats();

            // Draw key stats in compact form
            self.draw_stat(
                ctx,
                x,
                y,
                "Avg CPU Usage:",
                &alloc::format!("{}%", stats.avg_cpu_utilization),
                Color::from_rgb(100, 200, 255),
                font,
                theme,
            );

            y += 20;

            self.draw_stat(
                ctx,
                x,
                y,
                "Load Balance:",
                &alloc::format!("{}%", stats.load_balance_score),
                if stats.load_balance_score >= 80 {
                    Color::from_rgb(0, 255, 100)
                } else {
                    Color::from_rgb(255, 200, 0)
                },
                font,
                theme,
            );

            y += 20;

            self.draw_stat(
                ctx,
                x,
                y,
                "Total Decisions:",
                &alloc::format!("{}", stats.total_decisions),
                theme.text_primary,
                font,
                theme,
            );

            y += 30;

            // Draw separator
            ctx.fill_rect(
                Rect::new(bounds.x + 5, y, bounds.width - 10, 1),
                theme.border
            );

            y += 10;
        }

        // Draw recent decisions header
        ctx.draw_text(x, y, "Recent Decisions", font, theme.text_secondary);

        y += 25;

        // Get recent decisions
        let decisions = get_recent_scheduling_decisions(self.max_visible);

        // Draw decisions
        let line_height = 50;

        for decision in decisions.iter().take(self.max_visible) {
            if y + line_height > bounds.y + bounds.height - 10 {
                break;
            }

            // Draw action (main line)
            ctx.draw_text(
                x,
                y,
                &decision.action,
                font,
                theme.text_primary
            );

            // Draw reason (sub line)
            let reason_text = alloc::format!("→ {}", decision.reason);
            ctx.draw_text(
                x + 10,
                y + 16,
                &reason_text,
                font,
                theme.text_secondary
            );

            // Draw confidence indicator
            let conf_x = bounds.x + bounds.width - 85;
            let conf_width = 60;
            let conf_height = 10;

            // Background
            ctx.fill_rect(
                Rect::new(conf_x, y + 3, conf_width, conf_height),
                theme.bg_tertiary
            );

            // Fill
            let fill_width = (conf_width as f32 * (decision.confidence as f32 / 100.0)) as u32;
            ctx.fill_rect(
                Rect::new(conf_x, y + 3, fill_width, conf_height),
                Self::confidence_color(decision.confidence)
            );

            // Draw performance gain
            let gain_text = if decision.performance_gain >= 0 {
                alloc::format!("+{}%", decision.performance_gain)
            } else {
                alloc::format!("{}%", decision.performance_gain)
            };
            ctx.draw_text(
                conf_x,
                y + 18,
                &gain_text,
                font,
                Self::gain_color(decision.performance_gain)
            );

            y += line_height;
        }

        // Draw border
        ctx.draw_rect_outline(bounds, theme.border, 1);
    }

    fn handle_event(&mut self, event: &InputEvent, bounds: Rect) -> EventResponse {
        EventResponse::Ignored
    }

    fn preferred_size(&self, constraints: &SizeConstraints, font: &Font) -> Size {
        let stats_height = if self.show_stats { 85 } else { 0 };
        let header_height = 60;
        let decisions_height = (self.max_visible as u32) * 50;
        let height = stats_height + header_height + decisions_height + 20;

        Size::new(500, height)
    }
}
