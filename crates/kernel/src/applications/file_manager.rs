/// File Manager Application - Phase G.3
///
/// Browse and navigate filesystem

use crate::ui::{Widget, EventResponse, InputEvent, SizeConstraints, Size, Theme, MouseButton};
use crate::ui::{VStack, Label, Panel, Padding, TextAlignment};
use crate::graphics::{DrawContext, Rect, Font, Color};
use alloc::vec::{self, Vec};
use alloc::string::String;
use alloc::boxed::Box;

/// File entry in directory listing
#[derive(Debug, Clone)]
struct FileEntry {
    name: String,
    is_dir: bool,
    size: u64,
}

/// File Manager application
pub struct FileManagerApp {
    current_path: String,
    entries: Vec<FileEntry>,
    selected_index: Option<usize>,
    scroll_offset: usize,
}

impl FileManagerApp {
    /// Create a new file manager
    pub fn new() -> Self {
        let mut app = Self {
            current_path: String::from("/"),
            entries: Vec::new(),
            selected_index: None,
            scroll_offset: 0,
        };

        // Load initial directory
        app.load_directory();
        app
    }

    /// Load directory listing
    fn load_directory(&mut self) {
        self.entries.clear();
        self.selected_index = None;
        self.scroll_offset = 0;

        // Try to open directory via VFS
        match crate::vfs::open(&self.current_path, crate::vfs::OpenFlags::O_RDONLY | crate::vfs::OpenFlags::O_DIRECTORY) {
            Ok(file) => {
                // Read directory entries
                if let Ok(dir_entries) = file.readdir() {
                    for entry in dir_entries {
                        let is_dir = entry.itype == crate::vfs::InodeType::Directory;
                        self.entries.push(FileEntry {
                            name: entry.name.clone(),
                            is_dir,
                            size: 0, // Would get from stat
                        });
                    }
                }
            }
            Err(_) => {
                // Fallback to mock data for demo
                self.load_mock_data();
            }
        }

        // Sort: directories first, then files
        self.entries.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => core::cmp::Ordering::Less,
                (false, true) => core::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
        });
    }

    /// Load mock data for demo
    fn load_mock_data(&mut self) {
        self.entries = vec![
            FileEntry { name: String::from(".."), is_dir: true, size: 0 },
            FileEntry { name: String::from("bin"), is_dir: true, size: 0 },
            FileEntry { name: String::from("dev"), is_dir: true, size: 0 },
            FileEntry { name: String::from("etc"), is_dir: true, size: 0 },
            FileEntry { name: String::from("home"), is_dir: true, size: 0 },
            FileEntry { name: String::from("proc"), is_dir: true, size: 0 },
            FileEntry { name: String::from("tmp"), is_dir: true, size: 0 },
            FileEntry { name: String::from("README.md"), is_dir: false, size: 1024 },
            FileEntry { name: String::from("kernel.elf"), is_dir: false, size: 2048576 },
        ];
    }

    /// Navigate to directory
    fn navigate_to(&mut self, entry: &FileEntry) {
        if !entry.is_dir {
            return; // Can't navigate to files
        }

        if entry.name == ".." {
            // Go to parent
            if let Some(pos) = self.current_path.rfind('/') {
                if pos == 0 {
                    self.current_path = String::from("/");
                } else {
                    self.current_path.truncate(pos);
                }
            }
        } else {
            // Go to subdirectory
            if self.current_path == "/" {
                self.current_path = alloc::format!("/{}", entry.name);
            } else {
                self.current_path = alloc::format!("{}/{}", self.current_path, entry.name);
            }
        }

        self.load_directory();
    }

    /// Format file size
    fn format_size(size: u64) -> String {
        if size < 1024 {
            alloc::format!("{} B", size)
        } else if size < 1024 * 1024 {
            alloc::format!("{} KB", size / 1024)
        } else {
            alloc::format!("{} MB", size / (1024 * 1024))
        }
    }
}

impl Default for FileManagerApp {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for FileManagerApp {
    fn draw(&self, ctx: &mut DrawContext, bounds: Rect, theme: &Theme, font: &Font) {
        // Draw background
        ctx.fill_rect(bounds, Color::from_rgb(50, 50, 55));

        // Draw title bar
        let title_bar = Rect::new(bounds.x, bounds.y, bounds.width, 30);
        ctx.fill_rect(title_bar, theme.bg_tertiary);

        ctx.draw_text(
            bounds.x + 10,
            bounds.y + 8,
            "File Manager",
            font,
            theme.accent
        );

        // Draw path bar
        let path_bar = Rect::new(bounds.x, bounds.y + 30, bounds.width, 25);
        ctx.fill_rect(path_bar, theme.bg_secondary);

        ctx.draw_text(
            bounds.x + 10,
            bounds.y + 35,
            &alloc::format!("Path: {}", self.current_path),
            font,
            theme.text_secondary
        );

        // Draw file list
        let list_y = bounds.y + 60;
        let list_height = bounds.height - 60;
        let line_height = font.line_height() + 6;
        let visible_count = (list_height / line_height).min(self.entries.len() as u32) as usize;

        for (i, entry) in self.entries.iter()
            .skip(self.scroll_offset)
            .take(visible_count)
            .enumerate()
        {
            let y = list_y + (i as u32 * line_height);
            let is_selected = self.selected_index == Some(i + self.scroll_offset);

            // Draw selection highlight
            if is_selected {
                ctx.fill_rect(
                    Rect::new(bounds.x, y, bounds.width, line_height),
                    theme.accent.darken(0.7)
                );
            }

            // Draw icon
            let icon = if entry.is_dir { "📁" } else { "📄" };
            let icon_fallback = if entry.is_dir { "[DIR]" } else { "[FILE]" };
            ctx.draw_text(bounds.x + 10, y + 2, icon_fallback, font,
                if entry.is_dir { Color::from_rgb(100, 150, 255) } else { theme.text_primary });

            // Draw name
            ctx.draw_text(
                bounds.x + 60,
                y + 2,
                &entry.name,
                font,
                if is_selected { Color::WHITE } else { theme.text_primary }
            );

            // Draw size (for files)
            if !entry.is_dir {
                let size_str = Self::format_size(entry.size);
                let (size_width, _) = ctx.measure_text(&size_str, font);
                ctx.draw_text(
                    bounds.x + bounds.width - size_width - 10,
                    y + 2,
                    &size_str,
                    font,
                    theme.text_secondary
                );
            }
        }

        // Draw border
        ctx.draw_rect_outline(bounds, theme.border, 1);
    }

    fn handle_event(&mut self, event: &InputEvent, bounds: Rect) -> EventResponse {
        match event {
            InputEvent::MouseButton { button: MouseButton::Left, pressed: true, x, y } => {
                // Check if click is in file list area
                let list_y = bounds.y + 60;
                if *y < list_y {
                    return EventResponse::Ignored;
                }

                // Calculate which entry was clicked
                let line_height = 22; // font height + 6
                let relative_y = y.saturating_sub(list_y);
                let index = (relative_y / line_height) as usize + self.scroll_offset;

                if index < self.entries.len() {
                    // Double-click detection would go here
                    // For now, single click navigates
                    if let Some(entry) = self.entries.get(index).cloned() {
                        self.navigate_to(&entry);
                        return EventResponse::NeedsRedraw;
                    }
                }

                EventResponse::Consumed
            }

            InputEvent::KeyPress { key: crate::ui::KeyCode::Enter, .. } => {
                if let Some(idx) = self.selected_index {
                    if let Some(entry) = self.entries.get(idx).cloned() {
                        self.navigate_to(&entry);
                        return EventResponse::NeedsRedraw;
                    }
                }
                EventResponse::Consumed
            }

            InputEvent::KeyPress { key: crate::ui::KeyCode::Up, .. } => {
                if let Some(idx) = self.selected_index {
                    if idx > 0 {
                        self.selected_index = Some(idx - 1);
                        return EventResponse::NeedsRedraw;
                    }
                } else if !self.entries.is_empty() {
                    self.selected_index = Some(0);
                    return EventResponse::NeedsRedraw;
                }
                EventResponse::Consumed
            }

            InputEvent::KeyPress { key: crate::ui::KeyCode::Down, .. } => {
                if let Some(idx) = self.selected_index {
                    if idx + 1 < self.entries.len() {
                        self.selected_index = Some(idx + 1);
                        return EventResponse::NeedsRedraw;
                    }
                } else if !self.entries.is_empty() {
                    self.selected_index = Some(0);
                    return EventResponse::NeedsRedraw;
                }
                EventResponse::Consumed
            }

            _ => EventResponse::Ignored,
        }
    }

    fn preferred_size(&self, constraints: &SizeConstraints, font: &Font) -> Size {
        Size::new(450, 400)
    }

    fn can_focus(&self) -> bool {
        true
    }
}
