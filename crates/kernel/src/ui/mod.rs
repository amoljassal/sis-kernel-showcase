/// UI Toolkit - Phase G.2
///
/// Provides reusable widgets, layouts, and event handling for building UIs

pub mod event;
pub mod widget;
pub mod theme;
pub mod widgets;
pub mod layout;

pub use event::{InputEvent, EventResponse, SizeConstraints, Size, MouseButton, KeyCode, KeyModifiers};
pub use widget::{Widget, WidgetState, Padding};
pub use theme::{Theme, ThemeMode};
pub use widgets::{Label, TextAlignment, Button, Panel, TextBox, Spinner, ProgressBar, DotsIndicator, PulseIndicator};
pub use layout::{VStack, HStack};

use crate::graphics::{self, DrawContext, create_system_font, Color, Rect};
use crate::lib::error::Result;
use alloc::sync::Arc;
use alloc::boxed::Box;
use alloc::string::String;
use spin::Mutex;

/// UI Manager - coordinates UI rendering and event handling
pub struct UIManager {
    theme: Theme,
}

impl UIManager {
    /// Create a new UI manager
    pub fn new() -> Self {
        Self {
            theme: Theme::dark(),
        }
    }

    /// Get current theme
    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    /// Set theme
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    /// Toggle between dark and light themes
    pub fn toggle_theme(&mut self) {
        // Check if current theme is dark (by checking bg color)
        let is_dark = self.theme.bg_primary.r < 128;
        self.theme = if is_dark {
            Theme::light()
        } else {
            Theme::dark()
        };
    }
}

/// Global UI manager instance
static UI_MANAGER: Mutex<Option<Arc<Mutex<UIManager>>>> = Mutex::new(None);

/// Initialize UI toolkit
pub fn init() -> Result<()> {
    let manager = UIManager::new();
    *UI_MANAGER.lock() = Some(Arc::new(Mutex::new(manager)));

    crate::info!("ui: toolkit initialized");
    Ok(())
}

/// Get global UI manager
pub fn get_manager() -> Option<Arc<Mutex<UIManager>>> {
    UI_MANAGER.lock().clone()
}

/// Test UI toolkit with sample widgets
pub fn test_ui_toolkit() -> Result<()> {
    let ui_mgr = get_manager().ok_or(crate::lib::error::Errno::ENODEV)?;
    let ctx = graphics::get_context().ok_or(crate::lib::error::Errno::ENODEV)?;

    let font = create_system_font();
    let theme = ui_mgr.lock().theme().clone();

    // Create a test UI with various widgets
    let mut root = VStack::new()
        .with_spacing(8)
        .with_padding(Padding::all(16));

    // Title label
    root.add_child(Box::new(
        Label::new(String::from("UI Toolkit Demo - Phase G.2"))
            .with_color(theme.accent)
    ));

    // Subtitle
    root.add_child(Box::new(
        Label::new(String::from("Showcasing widgets and layouts"))
            .with_color(theme.text_secondary)
    ));

    // Horizontal button row
    let mut button_row = HStack::new().with_spacing(8);

    button_row.add_child(Box::new(
        Button::new(String::from("Click Me"))
    ));

    button_row.add_child(Box::new(
        Button::new(String::from("Button 2"))
    ));

    button_row.add_child(Box::new(
        Button::new(String::from("Button 3"))
    ));

    root.add_child(Box::new(button_row));

    // Text input label
    root.add_child(Box::new(
        Label::new(String::from("Text Input:"))
    ));

    // Text input box
    root.add_child(Box::new(
        TextBox::new()
            .with_placeholder(String::from("Type something..."))
    ));

    // Panel with content
    let mut panel = Panel::new()
        .with_bg_color(theme.bg_tertiary)
        .with_border(theme.border, 1)
        .with_padding(Padding::all(12))
        .with_min_size(300, 80);

    root.add_child(Box::new(panel));

    // Information label
    root.add_child(Box::new(
        Label::new(String::from("Features: VStack, HStack, Buttons, Labels, TextBox, Panel"))
            .with_alignment(TextAlignment::Center)
    ));

    // Clear screen and draw UI
    let mut ctx = ctx.lock();
    ctx.clear(theme.bg_primary);

    // Draw the root widget
    let bounds = Rect::new(50, 50, 700, 500);
    root.draw(&mut ctx, bounds, &theme, &font);

    // Flush to display
    ctx.flush_all()?;

    crate::info!("ui: test completed - widgets rendered successfully");

    Ok(())
}
