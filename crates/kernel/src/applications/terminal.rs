/// Terminal Application - Phase G.3
///
/// Interactive terminal with command input and output display

use crate::ui::{Widget, EventResponse, InputEvent, SizeConstraints, Size, Theme};
use crate::ui::{VStack, Label, TextBox, Panel, Padding};
use crate::graphics::{DrawContext, Rect, Font, Color};
use alloc::vec::Vec;
use alloc::string::String;
use alloc::boxed::Box;

/// Terminal application
pub struct TerminalApp {
    output_lines: Vec<String>,
    input_box: TextBox,
    max_lines: usize,
    prompt: String,
}

impl TerminalApp {
    /// Create a new terminal application
    pub fn new() -> Self {
        let mut app = Self {
            output_lines: Vec::new(),
            input_box: TextBox::new()
                .with_placeholder(String::from("Enter command...")),
            max_lines: 20,
            prompt: String::from("sis> "),
        };

        // Add welcome message
        app.output_lines.push(String::from("SIS OS Terminal v1.0"));
        app.output_lines.push(String::from("Type 'help' for available commands"));
        app.output_lines.push(String::from(""));

        app
    }

    /// Add output line
    pub fn add_output(&mut self, line: String) {
        self.output_lines.push(line);

        // Keep only last max_lines
        if self.output_lines.len() > self.max_lines {
            self.output_lines.remove(0);
        }
    }

    /// Execute command
    fn execute_command(&mut self, command: &str) {
        // Echo command with prompt
        self.add_output(alloc::format!("{}{}", self.prompt, command));

        // Parse and execute command
        let parts: Vec<&str> = command.trim().split_whitespace().collect();
        if parts.is_empty() {
            return;
        }

        match parts[0] {
            "help" => {
                self.add_output(String::from("Available commands:"));
                self.add_output(String::from("  help       - Show this help"));
                self.add_output(String::from("  clear      - Clear terminal"));
                self.add_output(String::from("  echo <msg> - Echo message"));
                self.add_output(String::from("  info       - System information"));
                self.add_output(String::from("  uptime     - System uptime"));
            }

            "clear" => {
                self.output_lines.clear();
                self.add_output(String::from("SIS OS Terminal v1.0"));
            }

            "echo" => {
                let message = parts[1..].join(" ");
                self.add_output(message);
            }

            "info" => {
                self.add_output(String::from("System: SIS OS"));
                self.add_output(String::from("Architecture: ARM64"));
                self.add_output(String::from("Version: Phase G.3"));
                self.add_output(String::from("AI-Native Desktop Environment"));
            }

            "uptime" => {
                // Placeholder - would get real uptime from kernel
                self.add_output(String::from("Uptime: Running"));
            }

            _ => {
                self.add_output(alloc::format!("Unknown command: {}", parts[0]));
                self.add_output(String::from("Type 'help' for available commands"));
            }
        }

        // Add blank line after output
        self.add_output(String::from(""));
    }

    /// Handle key press (check for Enter to execute command)
    fn handle_key(&mut self, event: &InputEvent) -> EventResponse {
        if let InputEvent::KeyPress { key: crate::ui::KeyCode::Enter, .. } = event {
            let command = String::from(self.input_box.text());
            if !command.is_empty() {
                self.execute_command(&command);
                self.input_box.clear();
                return EventResponse::NeedsRedraw;
            }
        }
        EventResponse::Ignored
    }
}

impl Widget for TerminalApp {
    fn draw(&self, ctx: &mut DrawContext, bounds: Rect, theme: &Theme, font: &Font) {
        // Draw background
        ctx.fill_rect(bounds, Color::from_rgb(20, 20, 20));

        // Draw output area
        let mut y = bounds.y + 10;
        let x = bounds.x + 10;
        let line_height = font.line_height() + 4;

        for line in &self.output_lines {
            if y + line_height > bounds.y + bounds.height - 40 {
                break; // Don't overflow into input area
            }

            ctx.draw_text(x, y, line, font, Color::from_rgb(0, 255, 0));
            y += line_height;
        }

        // Draw input box at bottom
        let input_bounds = Rect::new(
            bounds.x + 10,
            bounds.y + bounds.height - 35,
            bounds.width - 20,
            30
        );

        self.input_box.draw(ctx, input_bounds, theme, font);
    }

    fn handle_event(&mut self, event: &InputEvent, bounds: Rect) -> EventResponse {
        // Check for Enter key first
        let key_response = self.handle_key(event);
        if key_response != EventResponse::Ignored {
            return key_response;
        }

        // Forward to input box
        let input_bounds = Rect::new(
            bounds.x + 10,
            bounds.y + bounds.height - 35,
            bounds.width - 20,
            30
        );

        self.input_box.handle_event(event, input_bounds)
    }

    fn preferred_size(&self, constraints: &SizeConstraints, font: &Font) -> Size {
        Size::new(600, 400)
    }

    fn can_focus(&self) -> bool {
        true
    }

    fn on_focus(&mut self) {
        self.input_box.on_focus();
    }

    fn on_blur(&mut self) {
        self.input_box.on_blur();
    }
}

impl Default for TerminalApp {
    fn default() -> Self {
        Self::new()
    }
}
