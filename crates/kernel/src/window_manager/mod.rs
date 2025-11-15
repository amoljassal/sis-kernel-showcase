/// Window Manager - Phase G.1
///
/// Provides window management with tiling/floating layouts, decorations, and focus

pub mod window;
pub mod manager;

pub use window::{Window, WindowId, WindowSpec, WindowDecoration, ResizeEdge};
pub use manager::{WindowManager, LayoutMode, WindowAction};

use crate::graphics::{self, DrawContext, Font, create_system_font, Color, Rect};
use crate::lib::error::Result;
use alloc::sync::Arc;
use alloc::string::String;
use spin::Mutex;

/// Global window manager instance
static WINDOW_MANAGER: Mutex<Option<Arc<Mutex<WindowManager>>>> = Mutex::new(None);

/// Initialize window manager
pub fn init() -> Result<()> {
    // Get screen dimensions from graphics context
    let ctx = graphics::get_context().ok_or(crate::lib::error::Errno::ENODEV)?;
    let (width, height) = ctx.lock().dimensions();

    // Create window manager
    let wm = WindowManager::new(width, height);
    *WINDOW_MANAGER.lock() = Some(Arc::new(Mutex::new(wm)));

    crate::info!("window_manager: initialized {}x{}", width, height);
    Ok(())
}

/// Get global window manager
pub fn get_manager() -> Option<Arc<Mutex<WindowManager>>> {
    WINDOW_MANAGER.lock().clone()
}

/// Test window manager with sample windows
pub fn test_window_manager() -> Result<()> {
    let wm = get_manager().ok_or(crate::lib::error::Errno::ENODEV)?;
    let ctx = graphics::get_context().ok_or(crate::lib::error::Errno::ENODEV)?;

    // Create system font
    let font = create_system_font();

    // Create three test windows
    let mut wm = wm.lock();

    // Window 1: Terminal
    let win1_spec = WindowSpec {
        title: String::from("Terminal"),
        bounds: Rect::new(50, 50, 400, 300),
        resizable: true,
        movable: true,
        closable: true,
        decoration: WindowDecoration::default(),
    };
    let win1_id = wm.create_window(win1_spec);

    // Fill window 1 with some content
    if let Some(win1) = wm.get_window_mut(win1_id) {
        win1.clear(Color::from_rgb(20, 20, 20));
        // Draw some "terminal" content
        for i in 0..10 {
            let y = i * 20;
            for x in 0..win1.content_bounds.width {
                let offset = ((y + 10) * win1.bounds.width + win1.content_bounds.x - win1.bounds.x + x) as usize;
                if offset < win1.framebuffer.len() && i % 2 == 0 {
                    win1.framebuffer[offset] = Color::GREEN.to_argb();
                }
            }
        }
    }

    // Window 2: AI Insights
    let win2_spec = WindowSpec {
        title: String::from("AI Insights"),
        bounds: Rect::new(500, 100, 350, 250),
        resizable: true,
        movable: true,
        closable: true,
        decoration: WindowDecoration::default(),
    };
    let win2_id = wm.create_window(win2_spec);

    // Fill window 2 with blue gradient
    if let Some(win2) = wm.get_window_mut(win2_id) {
        for y in 0..win2.bounds.height {
            let intensity = (y as f32 / win2.bounds.height as f32 * 128.0) as u8;
            let color = Color::from_rgb(0, intensity / 2, intensity);
            for x in 0..win2.bounds.width {
                let offset = (y * win2.bounds.width + x) as usize;
                if offset < win2.framebuffer.len() {
                    win2.framebuffer[offset] = color.to_argb();
                }
            }
        }
    }

    // Window 3: File Manager
    let win3_spec = WindowSpec {
        title: String::from("Files"),
        bounds: Rect::new(200, 400, 380, 200),
        resizable: true,
        movable: true,
        closable: true,
        decoration: WindowDecoration::default(),
    };
    let win3_id = wm.create_window(win3_spec);

    // Fill window 3 with gray background
    if let Some(win3) = wm.get_window_mut(win3_id) {
        win3.clear(Color::from_rgb(50, 50, 55));
    }

    // Switch to floating mode for demo
    wm.set_layout_mode(LayoutMode::Floating);

    // Clear screen
    let mut ctx = ctx.lock();
    ctx.clear(Color::UI_BG_DARK);

    // Draw all windows
    wm.draw(&mut ctx, &font);

    // Flush to display
    ctx.flush_all()?;

    crate::info!("window_manager: test completed - {} windows created", wm.window_count());

    // Test focus cycling
    crate::info!("window_manager: testing focus cycling");
    wm.focus_next();
    wm.draw(&mut ctx, &font);
    ctx.flush_all()?;

    wm.focus_next();
    wm.draw(&mut ctx, &font);
    ctx.flush_all()?;

    // Test close button (simulate click on win1 close button)
    crate::info!("window_manager: testing window close");
    if let Some(win1) = wm.get_window(win1_id) {
        let close_rect = win1.close_button_rect();
        let click_x = close_rect.x + close_rect.width / 2;
        let click_y = close_rect.y + close_rect.height / 2;

        if let Some(WindowAction::Close(id)) = wm.handle_click(click_x, click_y) {
            crate::info!("window_manager: closing window {}", id);
            wm.destroy_window(id)?;
        }
    }

    // Redraw after close
    ctx.clear(Color::UI_BG_DARK);
    wm.draw(&mut ctx, &font);
    ctx.flush_all()?;

    crate::info!("window_manager: test passed - {} windows remaining", wm.window_count());

    Ok(())
}
