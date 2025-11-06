/// AI Dashboard Application - Phase G.4
///
/// Comprehensive dashboard for AI subsystem monitoring and control

use crate::ui::{Widget, EventResponse, InputEvent, SizeConstraints, Size, Theme};
use crate::ui::layout::VStack;
use crate::ui::widgets::{Label, TextAlignment, Panel};
use crate::ui::widget::Padding;
use crate::graphics::{DrawContext, Rect, Font, Color};
use crate::ai_ui::{DecisionViewer, MemoryPredictorViewer, SchedulingViewer, AIControls};
use alloc::boxed::Box;

/// AI Dashboard application
pub struct AIDashboardApp {
    layout: VStack,
    last_update: u64,
    update_interval: u64,
}

impl AIDashboardApp {
    /// Create a new AI dashboard
    pub fn new() -> Self {
        let mut layout = VStack::new();
        layout = layout.with_spacing(10).with_padding(Padding::all(10));

        // Add title
        let title = Label::new("AI Kernel Intelligence Dashboard")
            .with_alignment(TextAlignment::Center)
            .with_color(Color::from_rgb(100, 200, 255));
        layout.add_child(Box::new(title));

        // Add subtitle
        let subtitle = Label::new("Real-time AI decision monitoring and control")
            .with_alignment(TextAlignment::Center)
            .with_color(Color::from_rgb(150, 150, 150));
        layout.add_child(Box::new(subtitle));

        // Add spacing panel
        let spacer = Panel::new().with_min_size(1, 10);
        layout.add_child(Box::new(spacer));

        // Add AI controls widget
        let controls = AIControls::new();
        let controls_panel = Panel::new()
            .with_bg_color(Color::from_rgb(30, 30, 35))
            .with_border(Color::from_rgb(60, 60, 65), 1)
            .with_padding(Padding::all(0));
        layout.add_child(Box::new(controls));

        // Add memory predictor viewer
        let memory_viewer = MemoryPredictorViewer::new();
        layout.add_child(Box::new(memory_viewer));

        // Add scheduling viewer
        let scheduling_viewer = SchedulingViewer::new(4);
        layout.add_child(Box::new(scheduling_viewer));

        // Add decision log viewer
        let decision_viewer = DecisionViewer::new(6);
        layout.add_child(Box::new(decision_viewer));

        Self {
            layout,
            last_update: 0,
            update_interval: 1000, // Update every 1000ms
        }
    }

    /// Update the dashboard
    pub fn update(&mut self) {
        // Get current time from kernel
        let current_time = crate::time::get_uptime_ms();

        if current_time - self.last_update >= self.update_interval {
            self.last_update = current_time;
            // Trigger redraw by returning NeedsRedraw from handle_event
        }
    }
}

impl Default for AIDashboardApp {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for AIDashboardApp {
    fn draw(&self, ctx: &mut DrawContext, bounds: Rect, theme: &Theme, font: &Font) {
        // Draw dark background
        ctx.fill_rect(bounds, Color::from_rgb(20, 20, 25));

        // Draw the layout
        self.layout.draw(ctx, bounds, theme, font);

        // Draw status bar at the bottom
        let status_y = bounds.y + bounds.height - 25;
        let status_rect = Rect::new(bounds.x, status_y, bounds.width, 25);

        ctx.fill_rect(status_rect, Color::from_rgb(15, 15, 20));

        // Draw status line separator
        ctx.fill_rect(
            Rect::new(bounds.x, status_y, bounds.width, 1),
            Color::from_rgb(60, 60, 65)
        );

        // Draw status text
        let uptime = crate::time::get_uptime_ms();
        let status_text = alloc::format!(
            "Uptime: {}ms | AI Systems: Active | Updates: Auto",
            uptime
        );
        ctx.draw_text(
            bounds.x + 15,
            status_y + 8,
            &status_text,
            font,
            Color::from_rgb(100, 200, 255)
        );

        // Draw refresh indicator
        let refresh_text = "●"; // Active indicator
        let (text_width, _) = font.measure_text(&status_text);
        ctx.draw_text(
            bounds.x + 20 + text_width,
            status_y + 8,
            refresh_text,
            font,
            Color::from_rgb(0, 255, 100)
        );
    }

    fn handle_event(&mut self, event: &InputEvent, bounds: Rect) -> EventResponse {
        // Adjust bounds to exclude status bar
        let content_bounds = Rect::new(
            bounds.x,
            bounds.y,
            bounds.width,
            bounds.height.saturating_sub(25)
        );

        // Forward events to layout
        let response = self.layout.handle_event(event, content_bounds);

        // Check if we need to update
        self.update();

        response
    }

    fn preferred_size(&self, constraints: &SizeConstraints, font: &Font) -> Size {
        // Dashboard prefers large size to show all widgets
        let layout_size = self.layout.preferred_size(constraints, font);

        // Add space for status bar
        Size::new(
            layout_size.width.max(800),
            layout_size.height + 25
        )
    }
}

/// Initialize the AI dashboard demo
pub fn demo_ai_dashboard() {
    use crate::window_manager::{get_window_manager, WindowSpec, WindowDecoration};
    use crate::graphics::Rect;

    let mut wm = get_window_manager();

    // Create AI dashboard window
    let dashboard_spec = WindowSpec {
        title: "AI Intelligence Dashboard".into(),
        bounds: Rect::new(50, 50, 850, 700),
        resizable: true,
        movable: true,
        closable: true,
        decoration: WindowDecoration::default(),
    };

    let window_id = wm.create_window(dashboard_spec);

    // TODO: Attach AIDashboardApp as window content
    // For now, we just create the window
    // Full integration will happen when we add content rendering to window manager

    log!("AI Dashboard created in window {}", window_id);
}
