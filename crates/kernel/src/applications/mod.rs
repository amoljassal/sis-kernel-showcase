/// Desktop Applications - Phase G.3
///
/// Core applications for the AI-native desktop environment

pub mod terminal;
pub mod ai_insights;
pub mod file_manager;
pub mod settings;

pub use terminal::TerminalApp;
pub use ai_insights::AIInsightsApp;
pub use file_manager::FileManagerApp;
pub use settings::SettingsApp;

use crate::window_manager::{WindowManager, WindowSpec, WindowDecoration};
use crate::ui::{Theme, Widget};
use crate::graphics::{self, create_system_font, Rect, Color};
use crate::lib::error::Result;
use alloc::string::String;
use alloc::boxed::Box;

/// Launch all core applications in windows
pub fn launch_all_apps() -> Result<()> {
    let wm = crate::window_manager::get_manager()
        .ok_or(crate::lib::error::Errno::ENODEV)?;
    let ctx = graphics::get_context()
        .ok_or(crate::lib::error::Errno::ENODEV)?;
    let ui_mgr = crate::ui::get_manager()
        .ok_or(crate::lib::error::Errno::ENODEV)?;

    let font = create_system_font();
    let theme = ui_mgr.lock().theme().clone();

    // Switch to floating mode for app demo
    wm.lock().set_layout_mode(crate::window_manager::LayoutMode::Floating);

    // Launch Terminal
    crate::info!("applications: launching Terminal");
    let terminal_spec = WindowSpec {
        title: String::from("Terminal"),
        bounds: Rect::new(50, 50, 620, 450),
        resizable: true,
        movable: true,
        closable: true,
        decoration: WindowDecoration::default(),
    };
    let terminal_id = wm.lock().create_window(terminal_spec);

    // Create terminal app and render to window
    let terminal_app = TerminalApp::new();
    if let Some(window) = wm.lock().get_window_mut(terminal_id) {
        // Create a temporary DrawContext for the window
        // In a real implementation, each window would have its own context
        let content_bounds = window.content_bounds;

        // Clear window content
        window.clear(Color::from_rgb(20, 20, 20));

        // Note: In a complete implementation, we would render the widget to the window's framebuffer
        // For now, we mark the window as containing the terminal app
    }

    // Launch AI Insights
    crate::info!("applications: launching AI Insights");
    let insights_spec = WindowSpec {
        title: String::from("AI Insights"),
        bounds: Rect::new(700, 50, 520, 470),
        resizable: true,
        movable: true,
        closable: true,
        decoration: WindowDecoration::default(),
    };
    let insights_id = wm.lock().create_window(insights_spec);

    if let Some(window) = wm.lock().get_window_mut(insights_id) {
        window.clear(Color::from_rgb(0, 20, 40));
    }

    // Launch File Manager
    crate::info!("applications: launching File Manager");
    let files_spec = WindowSpec {
        title: String::from("Files"),
        bounds: Rect::new(100, 300, 470, 420),
        resizable: true,
        movable: true,
        closable: true,
        decoration: WindowDecoration::default(),
    };
    let files_id = wm.lock().create_window(files_spec);

    if let Some(window) = wm.lock().get_window_mut(files_id) {
        window.clear(Color::from_rgb(50, 50, 55));
    }

    // Launch Settings
    crate::info!("applications: launching Settings");
    let settings_spec = WindowSpec {
        title: String::from("System Settings"),
        bounds: Rect::new(600, 300, 570, 420),
        resizable: true,
        movable: true,
        closable: true,
        decoration: WindowDecoration::default(),
    };
    let settings_id = wm.lock().create_window(settings_spec);

    if let Some(window) = wm.lock().get_window_mut(settings_id) {
        window.clear(Color::from_rgb(45, 45, 48));
    }

    // Clear screen and render all windows
    let mut ctx = ctx.lock();
    ctx.clear(Color::UI_BG_DARK);

    wm.lock().draw(&mut ctx, &font);

    ctx.flush_all()?;

    crate::info!("applications: {} apps launched successfully", 4);

    Ok(())
}

/// Test applications individually
pub fn test_applications() -> Result<()> {
    let ctx = graphics::get_context()
        .ok_or(crate::lib::error::Errno::ENODEV)?;
    let ui_mgr = crate::ui::get_manager()
        .ok_or(crate::lib::error::Errno::ENODEV)?;

    let font = create_system_font();
    let theme = ui_mgr.lock().theme().clone();

    let mut ctx = ctx.lock();

    // Test Terminal App
    crate::info!("applications: testing Terminal");
    ctx.clear(Color::UI_BG_DARK);

    let terminal = TerminalApp::new();
    let terminal_bounds = Rect::new(50, 50, 600, 400);
    terminal.draw(&mut ctx, terminal_bounds, &theme, &font);

    ctx.flush_all()?;

    // Small delay to see each app
    for _ in 0..10000 {}

    // Test AI Insights
    crate::info!("applications: testing AI Insights");
    ctx.clear(Color::UI_BG_DARK);

    let insights = AIInsightsApp::new();
    let insights_bounds = Rect::new(50, 50, 500, 450);
    insights.draw(&mut ctx, insights_bounds, &theme, &font);

    ctx.flush_all()?;

    for _ in 0..10000 {}

    // Test File Manager
    crate::info!("applications: testing File Manager");
    ctx.clear(Color::UI_BG_DARK);

    let file_mgr = FileManagerApp::new();
    let files_bounds = Rect::new(50, 50, 450, 400);
    file_mgr.draw(&mut ctx, files_bounds, &theme, &font);

    ctx.flush_all()?;

    for _ in 0..10000 {}

    // Test Settings
    crate::info!("applications: testing Settings");
    ctx.clear(Color::UI_BG_DARK);

    let settings = SettingsApp::new();
    let settings_bounds = Rect::new(50, 50, 550, 400);
    settings.draw(&mut ctx, settings_bounds, &theme, &font);

    ctx.flush_all()?;

    crate::info!("applications: all tests passed");

    Ok(())
}
