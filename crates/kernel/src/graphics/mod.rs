/// Graphics Layer - Phase G.0
///
/// Provides 2D graphics primitives, color management, and font rendering

pub mod color;
pub mod primitives;
pub mod font;

pub use color::Color;
pub use primitives::{DrawContext, Rect};
pub use font::{Font, Glyph, create_system_font, create_display_font};

use crate::drivers::virtio_gpu;
use crate::lib::error::Result;
use alloc::sync::Arc;
use spin::Mutex;

/// Global graphics context
static GRAPHICS_CTX: Mutex<Option<Arc<Mutex<DrawContext>>>> = Mutex::new(None);

/// Initialize graphics subsystem
pub fn init() -> Result<()> {
    // Get GPU device
    let gpu = virtio_gpu::get_gpu().ok_or(crate::lib::error::Errno::ENODEV)?;

    // Create drawing context
    let ctx = DrawContext::new(gpu);
    *GRAPHICS_CTX.lock() = Some(Arc::new(Mutex::new(ctx)));

    crate::info!("graphics: initialized 2D rendering");
    Ok(())
}

/// Get global graphics context
pub fn get_context() -> Option<Arc<Mutex<DrawContext>>> {
    GRAPHICS_CTX.lock().clone()
}

/// Run a graphics test (draw some shapes and text)
pub fn test_graphics() -> Result<()> {
    let ctx = get_context().ok_or(crate::lib::error::Errno::ENODEV)?;
    let mut ctx = ctx.lock();

    // Clear screen to dark background
    ctx.clear(Color::UI_BG_DARK);

    // Draw title bar
    ctx.fill_rect(Rect::new(0, 0, 1280, 40), Color::UI_ACCENT);

    // Draw some test shapes
    ctx.fill_rect(Rect::new(100, 100, 200, 150), Color::RED);
    ctx.fill_rect(Rect::new(350, 100, 200, 150), Color::GREEN);
    ctx.fill_rect(Rect::new(600, 100, 200, 150), Color::BLUE);

    // Draw circles
    ctx.fill_circle(200, 350, 50, Color::YELLOW);
    ctx.fill_circle(450, 350, 50, Color::CYAN);
    ctx.fill_circle(700, 350, 50, Color::MAGENTA);

    // Draw lines
    ctx.draw_line(100, 500, 800, 500, Color::WHITE);
    ctx.draw_line(100, 520, 800, 580, Color::LIGHT_GRAY);

    // Create system font and draw text
    let font = create_system_font();
    ctx.draw_text(20, 10, "SIS OS - AI-Native Desktop Environment", &font, Color::WHITE);
    ctx.draw_text(100, 280, "Phase G.0: Foundation - Graphics Test", &font, Color::UI_TEXT);

    // Draw rectangles with outlines
    ctx.draw_rect_outline(Rect::new(100, 100, 200, 150), Color::WHITE, 2);
    ctx.draw_rect_outline(Rect::new(350, 100, 200, 150), Color::WHITE, 2);
    ctx.draw_rect_outline(Rect::new(600, 100, 200, 150), Color::WHITE, 2);

    // Flush to display
    ctx.flush_all()?;

    crate::info!("graphics: test completed successfully");
    Ok(())
}
