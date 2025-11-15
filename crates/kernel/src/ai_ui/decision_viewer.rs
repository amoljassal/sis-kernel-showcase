/// AI Decision Viewer Widget - Phase G.4
///
/// Displays recent AI decisions in real-time

use super::decision_log::{get_recent_decisions, DecisionType};
use crate::ui::{Widget, EventResponse, InputEvent, SizeConstraints, Size, Theme};
use crate::graphics::{DrawContext, Rect, Font, Color};

/// AI Decision Viewer widget
pub struct DecisionViewer {
    max_visible: usize,
}

impl DecisionViewer {
    /// Create a new decision viewer
    pub fn new(max_visible: usize) -> Self {
        Self { max_visible }
    }

    /// Get color for decision type
    fn type_color(decision_type: DecisionType) -> Color {
        match decision_type {
            DecisionType::MemoryPrediction => Color::from_rgb(100, 200, 255),
            DecisionType::SchedulingDecision => Color::from_rgb(255, 200, 100),
            DecisionType::LoadBalancing => Color::from_rgb(100, 255, 150),
            DecisionType::CacheOptimization => Color::from_rgb(255, 150, 200),
            DecisionType::PrefetchDecision => Color::from_rgb(200, 150, 255),
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
}

impl Default for DecisionViewer {
    fn default() -> Self {
        Self::new(10)
    }
}

impl Widget for DecisionViewer {
    fn draw(&self, ctx: &mut DrawContext, bounds: Rect, theme: &Theme, font: &Font) {
        // Draw background
        ctx.fill_rect(bounds, Color::from_rgb(25, 25, 30));

        // Draw title
        ctx.draw_text(
            bounds.x + 10,
            bounds.y + 10,
            "AI Decision Log",
            font,
            theme.accent
        );

        // Draw header
        let header_y = bounds.y + 35;
        ctx.draw_text(bounds.x + 10, header_y, "Type", font, theme.text_secondary);
        ctx.draw_text(bounds.x + 150, header_y, "Description", font, theme.text_secondary);
        ctx.draw_text(bounds.x + bounds.width - 100, header_y, "Confidence", font, theme.text_secondary);

        // Draw separator line
        ctx.fill_rect(
            Rect::new(bounds.x + 5, bounds.y + 55, bounds.width - 10, 1),
            theme.border
        );

        // Get recent decisions
        let decisions = get_recent_decisions(self.max_visible);

        // Draw decisions
        let line_height = 25;
        let mut y = bounds.y + 65;

        for decision in decisions.iter().take(self.max_visible) {
            if y + line_height > bounds.y + bounds.height - 10 {
                break;
            }

            // Draw type with color coding
            let type_str = decision.decision_type.as_str();
            let type_color = Self::type_color(decision.decision_type);
            ctx.draw_text(bounds.x + 10, y, type_str, font, type_color);

            // Draw description (truncated if needed)
            let desc = if decision.description.len() > 25 {
                alloc::format!("{}...", &decision.description[..22])
            } else {
                decision.description.clone()
            };
            ctx.draw_text(bounds.x + 150, y, &desc, font, theme.text_primary);

            // Draw confidence bar
            let conf_x = bounds.x + bounds.width - 95;
            let conf_width = 80;
            let conf_height = 12;

            // Background
            ctx.fill_rect(
                Rect::new(conf_x, y + 2, conf_width, conf_height),
                theme.bg_tertiary
            );

            // Confidence fill
            let fill_width = (conf_width as f32 * (decision.confidence as f32 / 100.0)) as u32;
            ctx.fill_rect(
                Rect::new(conf_x, y + 2, fill_width, conf_height),
                Self::confidence_color(decision.confidence)
            );

            // Confidence text
            let conf_text = alloc::format!("{}%", decision.confidence);
            ctx.draw_text(conf_x + conf_width + 5, y, &conf_text, font, theme.text_secondary);

            // Draw outcome indicator if available
            if let Some(outcome) = decision.outcome {
                let indicator = if outcome { "✓" } else { "✗" };
                let indicator_color = if outcome {
                    Color::from_rgb(0, 255, 0)
                } else {
                    Color::from_rgb(255, 0, 0)
                };
                ctx.draw_text(bounds.x + 130, y, indicator, font, indicator_color);
            }

            y += line_height;
        }

        // Draw border
        ctx.draw_rect_outline(bounds, theme.border, 1);
    }

    fn handle_event(&mut self, event: &InputEvent, bounds: Rect) -> EventResponse {
        EventResponse::Ignored
    }

    fn preferred_size(&self, constraints: &SizeConstraints, font: &Font) -> Size {
        let height = 65 + (self.max_visible as u32 * 25) + 10;
        Size::new(500, height)
    }
}
