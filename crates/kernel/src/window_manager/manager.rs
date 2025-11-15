/// Window Manager - Phase G.1
///
/// Manages multiple windows with tiling/floating layouts, focus, and composition

use super::window::{Window, WindowId, WindowSpec, WindowDecoration, ResizeEdge};
use crate::graphics::{Color, DrawContext, Rect, Font};
use crate::lib::error::{Result, Errno};
use alloc::vec::Vec;
use alloc::string::String;
use spin::Mutex;

/// Layout mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    Tiling,         // Auto-arrange windows (like i3wm)
    Floating,       // Manual positioning (like Windows)
    Fullscreen,     // Single fullscreen window
}

/// Window manager
pub struct WindowManager {
    windows: Vec<Window>,
    focused_id: Option<WindowId>,
    layout_mode: LayoutMode,
    screen_rect: Rect,
    next_window_id: u32,
    next_z_order: u32,
    drag_state: Option<DragState>,
    resize_state: Option<ResizeState>,
}

/// Window drag state
struct DragState {
    window_id: WindowId,
    start_x: u32,
    start_y: u32,
    window_start_x: u32,
    window_start_y: u32,
}

/// Window resize state
struct ResizeState {
    window_id: WindowId,
    edge: ResizeEdge,
    start_x: u32,
    start_y: u32,
    start_bounds: Rect,
}

impl WindowManager {
    /// Create a new window manager
    pub fn new(screen_width: u32, screen_height: u32) -> Self {
        Self {
            windows: Vec::new(),
            focused_id: None,
            layout_mode: LayoutMode::Tiling,
            screen_rect: Rect::new(0, 0, screen_width, screen_height),
            next_window_id: 1,
            next_z_order: 1,
            drag_state: None,
            resize_state: None,
        }
    }

    /// Create a new window
    pub fn create_window(&mut self, spec: WindowSpec) -> WindowId {
        let id = self.next_window_id;
        self.next_window_id += 1;

        // Adjust bounds based on layout mode
        let bounds = match self.layout_mode {
            LayoutMode::Tiling => self.calculate_tiling_bounds(),
            LayoutMode::Floating => spec.bounds,
            LayoutMode::Fullscreen => self.screen_rect,
        };

        let mut window_spec = spec;
        window_spec.bounds = bounds;

        let mut window = Window::new(id, window_spec);
        window.z_order = self.next_z_order;
        self.next_z_order += 1;

        self.windows.push(window);

        // Relayout if in tiling mode
        if self.layout_mode == LayoutMode::Tiling {
            self.relayout_tiling();
        }

        // Focus new window
        self.focus_window(id);

        id
    }

    /// Destroy a window
    pub fn destroy_window(&mut self, id: WindowId) -> Result<()> {
        let index = self.windows.iter().position(|w| w.id == id)
            .ok_or(Errno::ENOENT)?;

        self.windows.remove(index);

        // Update focus
        if self.focused_id == Some(id) {
            self.focus_next();
        }

        // Relayout if in tiling mode
        if self.layout_mode == LayoutMode::Tiling {
            self.relayout_tiling();
        }

        Ok(())
    }

    /// Get window by ID
    pub fn get_window(&self, id: WindowId) -> Option<&Window> {
        self.windows.iter().find(|w| w.id == id)
    }

    /// Get mutable window by ID
    pub fn get_window_mut(&mut self, id: WindowId) -> Option<&mut Window> {
        self.windows.iter_mut().find(|w| w.id == id)
    }

    /// Focus a window
    pub fn focus_window(&mut self, id: WindowId) {
        // Unfocus all windows
        for window in &mut self.windows {
            window.focused = false;
        }

        // Focus target window and bring to front
        if let Some(window) = self.windows.iter_mut().find(|w| w.id == id) {
            window.focused = true;
            window.z_order = self.next_z_order;
            self.next_z_order += 1;
        }

        self.focused_id = Some(id);
    }

    /// Focus next window (Alt+Tab)
    pub fn focus_next(&mut self) {
        let visible: Vec<WindowId> = self.windows.iter()
            .filter(|w| w.visible && !w.minimized)
            .map(|w| w.id)
            .collect();

        if visible.is_empty() {
            self.focused_id = None;
            return;
        }

        if let Some(current_id) = self.focused_id {
            if let Some(index) = visible.iter().position(|&id| id == current_id) {
                let next_index = (index + 1) % visible.len();
                self.focus_window(visible[next_index]);
                return;
            }
        }

        // Focus first window if no current focus
        self.focus_window(visible[0]);
    }

    /// Focus previous window (Alt+Shift+Tab)
    pub fn focus_prev(&mut self) {
        let visible: Vec<WindowId> = self.windows.iter()
            .filter(|w| w.visible && !w.minimized)
            .map(|w| w.id)
            .collect();

        if visible.is_empty() {
            self.focused_id = None;
            return;
        }

        if let Some(current_id) = self.focused_id {
            if let Some(index) = visible.iter().position(|&id| id == current_id) {
                let prev_index = if index == 0 {
                    visible.len() - 1
                } else {
                    index - 1
                };
                self.focus_window(visible[prev_index]);
                return;
            }
        }

        // Focus first window if no current focus
        self.focus_window(visible[0]);
    }

    /// Handle mouse click
    pub fn handle_click(&mut self, x: u32, y: u32) -> Option<WindowAction> {
        // Find topmost window at click position
        let mut clicked_window = None;
        let mut max_z = 0;

        for window in &self.windows {
            if window.visible && window.bounds.contains(x, y) && window.z_order >= max_z {
                clicked_window = Some(window.id);
                max_z = window.z_order;
            }
        }

        if let Some(id) = clicked_window {
            let window = self.get_window(id).unwrap();

            // Check if clicked on close button
            if window.decoration.show_close_button && window.close_button_rect().contains(x, y) {
                return Some(WindowAction::Close(id));
            }

            // Check if clicked on minimize button
            if window.decoration.show_minimize_button && window.minimize_button_rect().contains(x, y) {
                return Some(WindowAction::Minimize(id));
            }

            // Check if clicked on maximize button
            if window.decoration.show_maximize_button && window.maximize_button_rect().contains(x, y) {
                return Some(WindowAction::Maximize(id));
            }

            // Check if clicked in title bar (start drag)
            if window.is_in_title_bar(x, y) && window.movable {
                self.drag_state = Some(DragState {
                    window_id: id,
                    start_x: x,
                    start_y: y,
                    window_start_x: window.bounds.x,
                    window_start_y: window.bounds.y,
                });
                self.focus_window(id);
                return Some(WindowAction::StartDrag(id));
            }

            // Check if clicked on resize edge
            if let Some(edge) = window.is_on_resize_border(x, y) {
                self.resize_state = Some(ResizeState {
                    window_id: id,
                    edge,
                    start_x: x,
                    start_y: y,
                    start_bounds: window.bounds,
                });
                self.focus_window(id);
                return Some(WindowAction::StartResize(id, edge));
            }

            // Focus window
            self.focus_window(id);
            return Some(WindowAction::Focus(id));
        }

        None
    }

    /// Handle mouse drag
    pub fn handle_drag(&mut self, x: u32, y: u32) {
        // Copy drag state fields to avoid immutable borrow during mutation
        if let Some((win_id, start_x, start_y, win_start_x, win_start_y)) = self
            .drag_state
            .as_ref()
            .map(|d| (d.window_id, d.start_x, d.start_y, d.window_start_x, d.window_start_y))
        {
            let dx = x.saturating_sub(start_x);
            let dy = y.saturating_sub(start_y);

            if let Some(window) = self.get_window_mut(win_id) {
                let new_x = win_start_x.saturating_add(dx);
                let new_y = win_start_y.saturating_add(dy);
                window.move_to(new_x, new_y);
            }
        }

        if let Some((win_id, edge, start_x, start_y, start_bounds)) = self
            .resize_state
            .as_ref()
            .map(|r| (r.window_id, r.edge, r.start_x, r.start_y, r.start_bounds))
        {
            let dx = x as i32 - start_x as i32;
            let dy = y as i32 - start_y as i32;

            if let Some(window) = self.get_window_mut(win_id) {
                let mut new_bounds = start_bounds;

                match edge {
                    ResizeEdge::Right => {
                        new_bounds.width = (new_bounds.width as i32 + dx).max(100) as u32;
                    }
                    ResizeEdge::Bottom => {
                        new_bounds.height = (new_bounds.height as i32 + dy).max(80) as u32;
                    }
                    ResizeEdge::Left => {
                        let new_width = (new_bounds.width as i32 - dx).max(100) as u32;
                        new_bounds.x = (new_bounds.x as i32 + dx) as u32;
                        new_bounds.width = new_width;
                    }
                    ResizeEdge::Top => {
                        let new_height = (new_bounds.height as i32 - dy).max(80) as u32;
                        new_bounds.y = (new_bounds.y as i32 + dy) as u32;
                        new_bounds.height = new_height;
                    }
                    ResizeEdge::TopLeft => {
                        let new_width = (new_bounds.width as i32 - dx).max(100) as u32;
                        let new_height = (new_bounds.height as i32 - dy).max(80) as u32;
                        new_bounds.x = (new_bounds.x as i32 + dx) as u32;
                        new_bounds.y = (new_bounds.y as i32 + dy) as u32;
                        new_bounds.width = new_width;
                        new_bounds.height = new_height;
                    }
                    ResizeEdge::TopRight => {
                        let new_height = (new_bounds.height as i32 - dy).max(80) as u32;
                        new_bounds.y = (new_bounds.y as i32 + dy) as u32;
                        new_bounds.width = (new_bounds.width as i32 + dx).max(100) as u32;
                        new_bounds.height = new_height;
                    }
                    ResizeEdge::BottomLeft => {
                        let new_width = (new_bounds.width as i32 - dx).max(100) as u32;
                        new_bounds.x = (new_bounds.x as i32 + dx) as u32;
                        new_bounds.width = new_width;
                        new_bounds.height = (new_bounds.height as i32 + dy).max(80) as u32;
                    }
                    ResizeEdge::BottomRight => {
                        new_bounds.width = (new_bounds.width as i32 + dx).max(100) as u32;
                        new_bounds.height = (new_bounds.height as i32 + dy).max(80) as u32;
                    }
                }

                window.bounds = new_bounds;
                window.update_content_bounds();

                // Reallocate framebuffer
                let new_size = (new_bounds.width * new_bounds.height) as usize;
                window.framebuffer.clear();
                window.framebuffer.resize(new_size, Color::UI_BG_MEDIUM.to_argb());
            }
        }
    }

    /// Handle mouse release (stop drag/resize)
    pub fn handle_release(&mut self) {
        self.drag_state = None;
        self.resize_state = None;
    }

    /// Toggle layout mode
    pub fn toggle_layout_mode(&mut self) {
        self.layout_mode = match self.layout_mode {
            LayoutMode::Tiling => LayoutMode::Floating,
            LayoutMode::Floating => LayoutMode::Tiling,
            LayoutMode::Fullscreen => LayoutMode::Tiling,
        };

        if self.layout_mode == LayoutMode::Tiling {
            self.relayout_tiling();
        }
    }

    /// Set layout mode
    pub fn set_layout_mode(&mut self, mode: LayoutMode) {
        self.layout_mode = mode;

        if self.layout_mode == LayoutMode::Tiling {
            self.relayout_tiling();
        }
    }

    /// Relayout windows in tiling mode
    fn relayout_tiling(&mut self) {
        let visible: Vec<&mut Window> = self.windows.iter_mut()
            .filter(|w| w.visible && !w.minimized)
            .collect();

        let count = visible.len();
        if count == 0 {
            return;
        }

        // Simple vertical tiling: divide screen into horizontal strips
        let window_height = self.screen_rect.height / count as u32;

        for (i, window) in visible.into_iter().enumerate() {
            window.bounds = Rect::new(
                0,
                (i as u32) * window_height,
                self.screen_rect.width,
                window_height
            );
            window.update_content_bounds();

            // Reallocate framebuffer if size changed
            let new_size = (window.bounds.width * window.bounds.height) as usize;
            if window.framebuffer.len() != new_size {
                window.framebuffer.clear();
                window.framebuffer.resize(new_size, Color::UI_BG_MEDIUM.to_argb());
            }
        }
    }

    /// Calculate bounds for new window in tiling mode
    fn calculate_tiling_bounds(&self) -> Rect {
        // Will be adjusted in relayout
        self.screen_rect
    }

    /// Get number of windows
    pub fn window_count(&self) -> usize {
        self.windows.len()
    }

    /// Get all window IDs
    pub fn window_ids(&self) -> Vec<WindowId> {
        self.windows.iter().map(|w| w.id).collect()
    }

    /// Draw all windows to context
    pub fn draw(&self, ctx: &mut DrawContext, font: &Font) {
        // Sort windows by z_order (lowest first)
        let mut sorted: Vec<&Window> = self.windows.iter().collect();
        sorted.sort_by_key(|w| w.z_order);

        for window in sorted {
            if !window.visible {
                continue;
            }

            self.draw_window(window, ctx, font);
        }
    }

    /// Draw a single window
    fn draw_window(&self, window: &Window, ctx: &mut DrawContext, font: &Font) {
        // Draw window content from window's framebuffer
        self.blit_window_content(window, ctx);

        // Draw title bar
        if window.decoration.show_title_bar {
            let title_bar_color = if window.focused {
                Color::UI_ACCENT
            } else {
                Color::from_rgb(80, 80, 80)
            };

            ctx.fill_rect(window.title_bar_rect(), title_bar_color);

            // Draw title text
            let title_x = window.bounds.x + 10;
            let title_y = window.bounds.y + 8;
            ctx.draw_text(title_x, title_y, &window.title, font, Color::WHITE);

            // Draw close button
            if window.decoration.show_close_button {
                let close_rect = window.close_button_rect();
                ctx.fill_rect(close_rect, Color::from_rgb(200, 0, 0));
                let x_x = close_rect.x + close_rect.width / 2 - 4;
                let x_y = close_rect.y + close_rect.height / 2 - 8;
                ctx.draw_text(x_x, x_y, "X", font, Color::WHITE);
            }

            // Draw minimize button
            if window.decoration.show_minimize_button {
                let min_rect = window.minimize_button_rect();
                ctx.fill_rect(min_rect, Color::from_rgb(100, 100, 100));
                let line_y = min_rect.y + min_rect.height - 4;
                ctx.fill_rect(
                    Rect::new(min_rect.x + 4, line_y, min_rect.width - 8, 2),
                    Color::WHITE
                );
            }

            // Draw maximize button
            if window.decoration.show_maximize_button {
                let max_rect = window.maximize_button_rect();
                ctx.fill_rect(max_rect, Color::from_rgb(100, 100, 100));
                ctx.draw_rect_outline(
                    Rect::new(max_rect.x + 4, max_rect.y + 4, max_rect.width - 8, max_rect.height - 8),
                    Color::WHITE,
                    2
                );
            }
        }

        // Draw border
        if window.decoration.border_width > 0 {
            let border_color = if window.focused {
                Color::UI_ACCENT
            } else {
                Color::UI_BORDER
            };
            ctx.draw_rect_outline(window.bounds, border_color, window.decoration.border_width);
        }
    }

    /// Blit window content to main framebuffer
    fn blit_window_content(&self, window: &Window, ctx: &mut DrawContext) {
        let (screen_width, _) = ctx.dimensions();

        for y in 0..window.bounds.height {
            for x in 0..window.bounds.width {
                let src_offset = (y * window.bounds.width + x) as usize;
                let dst_x = window.bounds.x + x;
                let dst_y = window.bounds.y + y;

                if src_offset < window.framebuffer.len() {
                    let color = Color::from_argb(window.framebuffer[src_offset]);
                    ctx.draw_pixel(dst_x, dst_y, color);
                }
            }
        }
    }
}

/// Window action result from user interaction
#[derive(Debug, Clone, Copy)]
pub enum WindowAction {
    Focus(WindowId),
    Close(WindowId),
    Minimize(WindowId),
    Maximize(WindowId),
    StartDrag(WindowId),
    StartResize(WindowId, ResizeEdge),
}
