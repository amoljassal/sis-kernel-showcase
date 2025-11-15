/// Loading Indicators - Phase G.6
///
/// Visual loading indicators with animations

use crate::ui::{Widget, EventResponse, InputEvent, SizeConstraints, Size, Theme};
use crate::graphics::{DrawContext, Rect, Font, Color};

/// Loading indicator type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadingType {
    /// Spinning circle
    Spinner,
    /// Linear progress bar
    ProgressBar,
    /// Dots animation
    Dots,
    /// Pulsing circle
    Pulse,
}

/// Spinner widget
pub struct Spinner {
    angle: f32,          // Current rotation angle (degrees)
    speed: f32,          // Rotation speed (degrees per update)
    radius: u32,         // Spinner radius
    thickness: u32,      // Line thickness
    color: Color,
}

impl Spinner {
    /// Create a new spinner
    pub fn new() -> Self {
        Self {
            angle: 0.0,
            speed: 10.0,  // 10 degrees per frame
            radius: 20,
            thickness: 3,
            color: Color::from_rgb(0, 122, 204),
        }
    }

    /// Set spinner size
    pub fn with_radius(mut self, radius: u32) -> Self {
        self.radius = radius;
        self
    }

    /// Set spinner color
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Set rotation speed
    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    /// Update spinner (call this every frame)
    pub fn update(&mut self) {
        self.angle += self.speed;
        if self.angle >= 360.0 {
            self.angle -= 360.0;
        }
    }
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Spinner {
    fn draw(&self, ctx: &mut DrawContext, bounds: Rect, theme: &Theme, font: &Font) {
        let center_x = bounds.x + bounds.width / 2;
        let center_y = bounds.y + bounds.height / 2;

        // Draw partial circle (arc) to create spinner effect
        let arc_length = 270.0; // 3/4 circle
        let start_angle = self.angle;
        let end_angle = self.angle + arc_length;

        // Draw arc using line segments
        let segments = 30;
        for i in 0..segments {
            let t = i as f32 / segments as f32;
            let angle1 = start_angle + arc_length * t;
            let angle2 = start_angle + arc_length * (t + 1.0 / segments as f32);

            let rad1 = angle1 * core::f32::consts::PI / 180.0;
            let rad2 = angle2 * core::f32::consts::PI / 180.0;

            let x1 = center_x as i32 + (self.radius as f32 * libm::cosf(rad1)) as i32;
            let y1 = center_y as i32 + (self.radius as f32 * libm::sinf(rad1)) as i32;
            let x2 = center_x as i32 + (self.radius as f32 * libm::cosf(rad2)) as i32;
            let y2 = center_y as i32 + (self.radius as f32 * libm::sinf(rad2)) as i32;

            // Draw line with gradient effect (fade towards end)
            let alpha = (255.0 * (1.0 - t * 0.5)) as u8;
            let color = Color::from_rgba(self.color.r, self.color.g, self.color.b, alpha);

            ctx.draw_line(x1, y1, x2, y2, color);
        }
    }

    fn handle_event(&mut self, event: &InputEvent, bounds: Rect) -> EventResponse {
        EventResponse::Ignored
    }

    fn preferred_size(&self, constraints: &SizeConstraints, font: &Font) -> Size {
        let size = (self.radius * 2 + self.thickness * 2) as u32;
        Size::new(size, size)
    }
}

/// Progress bar widget
pub struct ProgressBar {
    progress: f32,       // 0.0 - 1.0
    width: u32,
    height: u32,
    show_percentage: bool,
    color: Color,
    background_color: Color,
}

impl ProgressBar {
    /// Create a new progress bar
    pub fn new() -> Self {
        Self {
            progress: 0.0,
            width: 300,
            height: 20,
            show_percentage: true,
            color: Color::from_rgb(0, 200, 100),
            background_color: Color::from_rgb(60, 60, 65),
        }
    }

    /// Set progress (0.0 - 1.0)
    pub fn set_progress(&mut self, progress: f32) {
        self.progress = progress.clamp(0.0, 1.0);
    }

    /// Get progress
    pub fn progress(&self) -> f32 {
        self.progress
    }

    /// Set size
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set color
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Show/hide percentage text
    pub fn with_percentage(mut self, show: bool) -> Self {
        self.show_percentage = show;
        self
    }
}

impl Default for ProgressBar {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for ProgressBar {
    fn draw(&self, ctx: &mut DrawContext, bounds: Rect, theme: &Theme, font: &Font) {
        // Draw background
        ctx.fill_rect(bounds, self.background_color);
        ctx.draw_rect_outline(bounds, theme.border, 1);

        // Draw progress fill
        let fill_width = (bounds.width as f32 * self.progress) as u32;
        if fill_width > 0 {
            ctx.fill_rect(
                Rect::new(bounds.x, bounds.y, fill_width, bounds.height),
                self.color,
            );
        }

        // Draw percentage text
        if self.show_percentage {
            let percentage = (self.progress * 100.0) as u32;
            let text = alloc::format!("{}%", percentage);
            let (text_width, text_height) = font.measure_text(&text);

            let text_x = bounds.x + (bounds.width - text_width) / 2;
            let text_y = bounds.y + (bounds.height - text_height) / 2;

            // Draw text with shadow for better visibility
            ctx.draw_text(text_x + 1, text_y + 1, &text, font, Color::from_rgb(0, 0, 0));
            ctx.draw_text(text_x, text_y, &text, font, Color::from_rgb(255, 255, 255));
        }
    }

    fn handle_event(&mut self, event: &InputEvent, bounds: Rect) -> EventResponse {
        EventResponse::Ignored
    }

    fn preferred_size(&self, constraints: &SizeConstraints, font: &Font) -> Size {
        Size::new(self.width, self.height)
    }
}

/// Dots loading indicator
pub struct DotsIndicator {
    frame: u32,
    dot_count: usize,
    dot_radius: u32,
    spacing: u32,
    color: Color,
    animation_speed: u32,  // Frames per cycle
}

impl DotsIndicator {
    /// Create a new dots indicator
    pub fn new() -> Self {
        Self {
            frame: 0,
            dot_count: 3,
            dot_radius: 5,
            spacing: 15,
            color: Color::from_rgb(0, 122, 204),
            animation_speed: 30,
        }
    }

    /// Update animation
    pub fn update(&mut self) {
        self.frame += 1;
        if self.frame >= self.animation_speed {
            self.frame = 0;
        }
    }

    /// Set color
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

impl Default for DotsIndicator {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for DotsIndicator {
    fn draw(&self, ctx: &mut DrawContext, bounds: Rect, theme: &Theme, font: &Font) {
        let center_x = bounds.x + bounds.width / 2;
        let center_y = bounds.y + bounds.height / 2;

        let total_width = (self.dot_count - 1) as u32 * self.spacing;
        let start_x = center_x as i32 - total_width as i32 / 2;

        for i in 0..self.dot_count {
            let x = start_x + (i as u32 * self.spacing) as i32;
            let y = center_y as i32;

            // Calculate scale based on animation
            let phase = (self.frame as f32 / self.animation_speed as f32) * 2.0 * core::f32::consts::PI;
            let offset = (i as f32 / self.dot_count as f32) * 2.0 * core::f32::consts::PI;
            let scale = 0.5 + 0.5 * libm::sinf(phase + offset);

            let radius = (self.dot_radius as f32 * scale) as i32;

            // Draw dot
            ctx.draw_circle(x, y, radius, self.color);
        }
    }

    fn handle_event(&mut self, event: &InputEvent, bounds: Rect) -> EventResponse {
        EventResponse::Ignored
    }

    fn preferred_size(&self, constraints: &SizeConstraints, font: &Font) -> Size {
        let width = (self.dot_count - 1) as u32 * self.spacing + self.dot_radius * 2;
        let height = self.dot_radius * 4;
        Size::new(width, height)
    }
}

/// Pulsing circle indicator
pub struct PulseIndicator {
    phase: f32,
    radius: u32,
    speed: f32,
    color: Color,
}

impl PulseIndicator {
    /// Create a new pulse indicator
    pub fn new() -> Self {
        Self {
            phase: 0.0,
            radius: 20,
            speed: 0.1,
            color: Color::from_rgb(0, 122, 204),
        }
    }

    /// Update animation
    pub fn update(&mut self) {
        self.phase += self.speed;
        if self.phase >= 2.0 * core::f32::consts::PI {
            self.phase -= 2.0 * core::f32::consts::PI;
        }
    }

    /// Set color
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Set radius
    pub fn with_radius(mut self, radius: u32) -> Self {
        self.radius = radius;
        self
    }
}

impl Default for PulseIndicator {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for PulseIndicator {
    fn draw(&self, ctx: &mut DrawContext, bounds: Rect, theme: &Theme, font: &Font) {
        let center_x = bounds.x + bounds.width / 2;
        let center_y = bounds.y + bounds.height / 2;

        // Calculate pulsing scale
        let scale = 0.6 + 0.4 * libm::sinf(self.phase);
        let current_radius = (self.radius as f32 * scale) as i32;

        // Calculate pulsing opacity
        let alpha = (128.0 + 127.0 * libm::cosf(self.phase)) as u8;
        let color = Color::from_rgba(self.color.r, self.color.g, self.color.b, alpha);

        // Draw pulsing circle
        ctx.draw_circle(center_x as i32, center_y as i32, current_radius, color);
    }

    fn handle_event(&mut self, event: &InputEvent, bounds: Rect) -> EventResponse {
        EventResponse::Ignored
    }

    fn preferred_size(&self, constraints: &SizeConstraints, font: &Font) -> Size {
        let size = self.radius * 2;
        Size::new(size, size)
    }
}
