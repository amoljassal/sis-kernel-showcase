/// Icon System - Phase G.6
///
/// Simple icon rendering using geometric shapes and Unicode symbols

use super::{DrawContext, Rect, Color, Font};

/// Icon type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconType {
    // File/Folder icons
    File,
    Folder,
    FolderOpen,
    Document,
    Image,
    Audio,
    Video,
    Archive,

    // Application icons
    Terminal,
    Settings,
    Browser,
    Editor,

    // UI icons
    Close,
    Minimize,
    Maximize,
    Menu,
    Search,
    User,

    // Status icons
    Info,
    Warning,
    Error,
    Success,

    // Arrow icons
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,

    // Media icons
    Play,
    Pause,
    Stop,
    VolumeUp,
    VolumeDown,
    VolumeMute,

    // Misc
    Star,
    Heart,
    Lock,
    Unlock,
    Download,
    Upload,
}

/// Icon rendering size
#[derive(Debug, Clone, Copy)]
pub enum IconSize {
    Small,    // 16x16
    Medium,   // 24x24
    Large,    // 32x32
    XLarge,   // 48x48
}

impl IconSize {
    pub fn pixels(&self) -> u32 {
        match self {
            IconSize::Small => 16,
            IconSize::Medium => 24,
            IconSize::Large => 32,
            IconSize::XLarge => 48,
        }
    }
}

/// Icon renderer
pub struct Icon {
    icon_type: IconType,
    size: IconSize,
    color: Color,
}

impl Icon {
    /// Create a new icon
    pub fn new(icon_type: IconType) -> Self {
        Self {
            icon_type,
            size: IconSize::Medium,
            color: Color::from_rgb(255, 255, 255),
        }
    }

    /// Set icon size
    pub fn with_size(mut self, size: IconSize) -> Self {
        self.size = size;
        self
    }

    /// Set icon color
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Draw the icon
    pub fn draw(&self, ctx: &mut DrawContext, x: u32, y: u32) {
        let size = self.size.pixels();
        let bounds = Rect::new(x, y, size, size);

        match self.icon_type {
            IconType::File => self.draw_file(ctx, bounds),
            IconType::Folder => self.draw_folder(ctx, bounds),
            IconType::FolderOpen => self.draw_folder_open(ctx, bounds),
            IconType::Document => self.draw_document(ctx, bounds),
            IconType::Terminal => self.draw_terminal(ctx, bounds),
            IconType::Settings => self.draw_settings(ctx, bounds),
            IconType::Close => self.draw_close(ctx, bounds),
            IconType::Minimize => self.draw_minimize(ctx, bounds),
            IconType::Maximize => self.draw_maximize(ctx, bounds),
            IconType::Menu => self.draw_menu(ctx, bounds),
            IconType::Search => self.draw_search(ctx, bounds),
            IconType::Info => self.draw_info(ctx, bounds),
            IconType::Warning => self.draw_warning(ctx, bounds),
            IconType::Error => self.draw_error(ctx, bounds),
            IconType::Success => self.draw_success(ctx, bounds),
            IconType::Play => self.draw_play(ctx, bounds),
            IconType::Pause => self.draw_pause(ctx, bounds),
            IconType::Star => self.draw_star(ctx, bounds),
            _ => self.draw_placeholder(ctx, bounds),
        }
    }

    /// Draw file icon
    fn draw_file(&self, ctx: &mut DrawContext, bounds: Rect) {
        let margin = bounds.width / 8;
        let corner = bounds.width / 4;

        // Main rectangle
        ctx.draw_rect_outline(
            Rect::new(
                bounds.x + margin,
                bounds.y + margin,
                bounds.width - margin * 2,
                bounds.height - margin * 2,
            ),
            self.color,
            2,
        );

        // Corner fold
        ctx.draw_line(
            (bounds.x + bounds.width - margin - corner) as i32,
            (bounds.y + margin) as i32,
            (bounds.x + bounds.width - margin - corner) as i32,
            (bounds.y + margin + corner) as i32,
            self.color,
        );
        ctx.draw_line(
            (bounds.x + bounds.width - margin - corner) as i32,
            (bounds.y + margin + corner) as i32,
            (bounds.x + bounds.width - margin) as i32,
            (bounds.y + margin + corner) as i32,
            self.color,
        );
    }

    /// Draw folder icon
    fn draw_folder(&self, ctx: &mut DrawContext, bounds: Rect) {
        let margin = bounds.width / 8;

        // Main folder body
        ctx.fill_rect(
            Rect::new(
                bounds.x + margin,
                bounds.y + bounds.height / 3,
                bounds.width - margin * 2,
                bounds.height / 2,
            ),
            self.color,
        );

        // Folder tab
        ctx.fill_rect(
            Rect::new(
                bounds.x + margin,
                bounds.y + margin,
                bounds.width / 3,
                bounds.height / 4,
            ),
            self.color,
        );
    }

    /// Draw folder open icon
    fn draw_folder_open(&self, ctx: &mut DrawContext, bounds: Rect) {
        self.draw_folder(ctx, bounds);
        // Add opening effect (slanted rectangle)
        let margin = bounds.width / 8;
        ctx.draw_rect_outline(
            Rect::new(
                bounds.x + margin * 2,
                bounds.y + bounds.height / 2,
                bounds.width - margin * 4,
                bounds.height / 3,
            ),
            Color::from_rgb(self.color.r / 2, self.color.g / 2, self.color.b / 2),
            1,
        );
    }

    /// Draw document icon
    fn draw_document(&self, ctx: &mut DrawContext, bounds: Rect) {
        self.draw_file(ctx, bounds);

        // Add lines
        let margin = bounds.width / 4;
        for i in 0..3 {
            let y = bounds.y + bounds.height / 3 + i * (bounds.height / 12);
            ctx.fill_rect(
                Rect::new(bounds.x + margin, y, bounds.width - margin * 2, 2),
                self.color,
            );
        }
    }

    /// Draw terminal icon
    fn draw_terminal(&self, ctx: &mut DrawContext, bounds: Rect) {
        let margin = bounds.width / 8;

        // Terminal window
        ctx.draw_rect_outline(
            Rect::new(
                bounds.x + margin,
                bounds.y + margin,
                bounds.width - margin * 2,
                bounds.height - margin * 2,
            ),
            self.color,
            2,
        );

        // Prompt symbol ">"
        let prompt_x = bounds.x + margin * 2;
        let prompt_y = bounds.y + bounds.height / 2;
        ctx.draw_line(
            prompt_x as i32,
            (prompt_y - margin) as i32,
            (prompt_x + margin * 2) as i32,
            prompt_y as i32,
            self.color,
        );
        ctx.draw_line(
            (prompt_x + margin * 2) as i32,
            prompt_y as i32,
            prompt_x as i32,
            (prompt_y + margin) as i32,
            self.color,
        );
    }

    /// Draw settings icon (gear)
    fn draw_settings(&self, ctx: &mut DrawContext, bounds: Rect) {
        let center_x = bounds.x + bounds.width / 2;
        let center_y = bounds.y + bounds.height / 2;
        let radius = bounds.width / 3;

        // Draw gear (simplified with circle and lines)
        ctx.draw_circle(center_x as i32, center_y as i32, radius as i32, self.color);

        // Draw gear teeth (8 lines radiating out)
        for i in 0..8 {
            let angle = (i as f32 / 8.0) * 2.0 * core::f32::consts::PI;
            let x1 = center_x as i32 + (radius as f32 * angle.cos()) as i32;
            let y1 = center_y as i32 + (radius as f32 * angle.sin()) as i32;
            let x2 = center_x as i32 + ((radius + 6) as f32 * angle.cos()) as i32;
            let y2 = center_y as i32 + ((radius + 6) as f32 * angle.sin()) as i32;
            ctx.draw_line(x1, y1, x2, y2, self.color);
        }
    }

    /// Draw close icon (X)
    fn draw_close(&self, ctx: &mut DrawContext, bounds: Rect) {
        let margin = bounds.width / 4;
        ctx.draw_line(
            (bounds.x + margin) as i32,
            (bounds.y + margin) as i32,
            (bounds.x + bounds.width - margin) as i32,
            (bounds.y + bounds.height - margin) as i32,
            self.color,
        );
        ctx.draw_line(
            (bounds.x + bounds.width - margin) as i32,
            (bounds.y + margin) as i32,
            (bounds.x + margin) as i32,
            (bounds.y + bounds.height - margin) as i32,
            self.color,
        );
    }

    /// Draw minimize icon (-)
    fn draw_minimize(&self, ctx: &mut DrawContext, bounds: Rect) {
        let margin = bounds.width / 4;
        ctx.fill_rect(
            Rect::new(
                bounds.x + margin,
                bounds.y + bounds.height / 2 - 1,
                bounds.width - margin * 2,
                3,
            ),
            self.color,
        );
    }

    /// Draw maximize icon (square)
    fn draw_maximize(&self, ctx: &mut DrawContext, bounds: Rect) {
        let margin = bounds.width / 4;
        ctx.draw_rect_outline(
            Rect::new(
                bounds.x + margin,
                bounds.y + margin,
                bounds.width - margin * 2,
                bounds.height - margin * 2,
            ),
            self.color,
            2,
        );
    }

    /// Draw menu icon (three lines)
    fn draw_menu(&self, ctx: &mut DrawContext, bounds: Rect) {
        let margin = bounds.width / 4;
        for i in 0..3 {
            let y = bounds.y + bounds.height / 4 + i * (bounds.height / 4);
            ctx.fill_rect(
                Rect::new(bounds.x + margin, y, bounds.width - margin * 2, 3),
                self.color,
            );
        }
    }

    /// Draw search icon (magnifying glass)
    fn draw_search(&self, ctx: &mut DrawContext, bounds: Rect) {
        let center_x = bounds.x + bounds.width / 3;
        let center_y = bounds.y + bounds.height / 3;
        let radius = bounds.width / 4;

        // Circle
        ctx.draw_circle(center_x as i32, center_y as i32, radius as i32, self.color);

        // Handle
        ctx.draw_line(
            (center_x + radius) as i32,
            (center_y + radius) as i32,
            (bounds.x + bounds.width - bounds.width / 6) as i32,
            (bounds.y + bounds.height - bounds.height / 6) as i32,
            self.color,
        );
    }

    /// Draw info icon (i)
    fn draw_info(&self, ctx: &mut DrawContext, bounds: Rect) {
        let center_x = bounds.x + bounds.width / 2;
        let center_y = bounds.y + bounds.height / 2;
        let radius = bounds.width / 2 - 2;

        // Circle outline
        ctx.draw_circle(center_x as i32, center_y as i32, radius as i32, self.color);

        // "i" dot
        ctx.fill_rect(
            Rect::new(center_x - 1, bounds.y + bounds.height / 4, 3, 3),
            self.color,
        );

        // "i" stem
        ctx.fill_rect(
            Rect::new(center_x - 1, bounds.y + bounds.height / 3, 3, bounds.height / 2),
            self.color,
        );
    }

    /// Draw warning icon (triangle with !)
    fn draw_warning(&self, ctx: &mut DrawContext, bounds: Rect) {
        let center_x = bounds.x + bounds.width / 2;
        let bottom_y = bounds.y + bounds.height - bounds.height / 8;
        let top_y = bounds.y + bounds.height / 8;

        // Triangle
        ctx.draw_line(
            center_x as i32,
            top_y as i32,
            (bounds.x + bounds.width / 8) as i32,
            bottom_y as i32,
            self.color,
        );
        ctx.draw_line(
            center_x as i32,
            top_y as i32,
            (bounds.x + bounds.width - bounds.width / 8) as i32,
            bottom_y as i32,
            self.color,
        );
        ctx.draw_line(
            (bounds.x + bounds.width / 8) as i32,
            bottom_y as i32,
            (bounds.x + bounds.width - bounds.width / 8) as i32,
            bottom_y as i32,
            self.color,
        );

        // "!" symbol
        ctx.fill_rect(
            Rect::new(center_x - 1, bounds.y + bounds.height / 3, 3, bounds.height / 4),
            self.color,
        );
        ctx.fill_rect(
            Rect::new(center_x - 1, bounds.y + bounds.height - bounds.height / 4, 3, 3),
            self.color,
        );
    }

    /// Draw error icon (X in circle)
    fn draw_error(&self, ctx: &mut DrawContext, bounds: Rect) {
        let center_x = bounds.x + bounds.width / 2;
        let center_y = bounds.y + bounds.height / 2;
        let radius = bounds.width / 2 - 2;

        // Circle
        ctx.draw_circle(center_x as i32, center_y as i32, radius as i32, Color::from_rgb(255, 0, 0));

        // X
        let margin = bounds.width / 3;
        ctx.draw_line(
            (bounds.x + margin) as i32,
            (bounds.y + margin) as i32,
            (bounds.x + bounds.width - margin) as i32,
            (bounds.y + bounds.height - margin) as i32,
            Color::from_rgb(255, 0, 0),
        );
        ctx.draw_line(
            (bounds.x + bounds.width - margin) as i32,
            (bounds.y + margin) as i32,
            (bounds.x + margin) as i32,
            (bounds.y + bounds.height - margin) as i32,
            Color::from_rgb(255, 0, 0),
        );
    }

    /// Draw success icon (checkmark)
    fn draw_success(&self, ctx: &mut DrawContext, bounds: Rect) {
        let center_x = bounds.x + bounds.width / 2;
        let center_y = bounds.y + bounds.height / 2;
        let radius = bounds.width / 2 - 2;

        // Circle
        ctx.draw_circle(center_x as i32, center_y as i32, radius as i32, Color::from_rgb(0, 255, 0));

        // Checkmark
        ctx.draw_line(
            (bounds.x + bounds.width / 4) as i32,
            (center_y) as i32,
            (bounds.x + bounds.width / 2 - 2) as i32,
            (bounds.y + bounds.height - bounds.height / 3) as i32,
            Color::from_rgb(0, 255, 0),
        );
        ctx.draw_line(
            (bounds.x + bounds.width / 2 - 2) as i32,
            (bounds.y + bounds.height - bounds.height / 3) as i32,
            (bounds.x + bounds.width - bounds.width / 4) as i32,
            (bounds.y + bounds.height / 4) as i32,
            Color::from_rgb(0, 255, 0),
        );
    }

    /// Draw play icon (triangle)
    fn draw_play(&self, ctx: &mut DrawContext, bounds: Rect) {
        let margin = bounds.width / 4;
        ctx.draw_line(
            (bounds.x + margin) as i32,
            (bounds.y + margin) as i32,
            (bounds.x + margin) as i32,
            (bounds.y + bounds.height - margin) as i32,
            self.color,
        );
        ctx.draw_line(
            (bounds.x + margin) as i32,
            (bounds.y + margin) as i32,
            (bounds.x + bounds.width - margin) as i32,
            (bounds.y + bounds.height / 2) as i32,
            self.color,
        );
        ctx.draw_line(
            (bounds.x + margin) as i32,
            (bounds.y + bounds.height - margin) as i32,
            (bounds.x + bounds.width - margin) as i32,
            (bounds.y + bounds.height / 2) as i32,
            self.color,
        );
    }

    /// Draw pause icon (two bars)
    fn draw_pause(&self, ctx: &mut DrawContext, bounds: Rect) {
        let margin = bounds.width / 4;
        let bar_width = bounds.width / 6;

        // Left bar
        ctx.fill_rect(
            Rect::new(bounds.x + margin, bounds.y + margin, bar_width, bounds.height - margin * 2),
            self.color,
        );

        // Right bar
        ctx.fill_rect(
            Rect::new(
                bounds.x + bounds.width - margin - bar_width,
                bounds.y + margin,
                bar_width,
                bounds.height - margin * 2,
            ),
            self.color,
        );
    }

    /// Draw star icon
    fn draw_star(&self, ctx: &mut DrawContext, bounds: Rect) {
        let center_x = bounds.x + bounds.width / 2;
        let center_y = bounds.y + bounds.height / 2;
        let radius = bounds.width / 2 - 2;

        // Draw 5-pointed star
        for i in 0..5 {
            let angle1 = (i as f32 / 5.0) * 2.0 * core::f32::consts::PI - core::f32::consts::PI / 2.0;
            let angle2 = ((i + 2) as f32 / 5.0) * 2.0 * core::f32::consts::PI - core::f32::consts::PI / 2.0;

            let x1 = center_x as i32 + (radius as f32 * angle1.cos()) as i32;
            let y1 = center_y as i32 + (radius as f32 * angle1.sin()) as i32;
            let x2 = center_x as i32 + (radius as f32 * angle2.cos()) as i32;
            let y2 = center_y as i32 + (radius as f32 * angle2.sin()) as i32;

            ctx.draw_line(x1, y1, x2, y2, self.color);
        }
    }

    /// Draw placeholder icon (square)
    fn draw_placeholder(&self, ctx: &mut DrawContext, bounds: Rect) {
        ctx.draw_rect_outline(bounds, self.color, 2);
    }
}
