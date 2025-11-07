/// Layout System - Phase G.2
///
/// Provides layout managers for arranging widgets

use crate::ui::widget::{Widget, Padding};
use crate::ui::event::{InputEvent, EventResponse, SizeConstraints, Size};
use crate::ui::theme::Theme;
use crate::graphics::{DrawContext, Rect, Font};
use alloc::vec::Vec;
use alloc::boxed::Box;

/// Vertical stack layout - arranges children vertically
pub struct VStack {
    children: Vec<Box<dyn Widget + Send + Sync>>,
    spacing: u32,
    padding: Padding,
}

impl VStack {
    /// Create a new vertical stack
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            spacing: 4,
            padding: Padding::zero(),
        }
    }

    /// Set spacing between children
    pub fn with_spacing(mut self, spacing: u32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Set padding
    pub fn with_padding(mut self, padding: Padding) -> Self {
        self.padding = padding;
        self
    }

    /// Add a child widget
    pub fn add_child(&mut self, child: Box<dyn Widget + Send + Sync>) {
        self.children.push(child);
    }

    /// Calculate child bounds for drawing
    fn calculate_child_bounds(&self, bounds: Rect, font: &Font) -> Vec<Rect> {
        let mut child_bounds = Vec::new();

        let content_width = bounds.width.saturating_sub(self.padding.horizontal());
        let content_height = bounds.height.saturating_sub(self.padding.vertical());

        let mut y = bounds.y + self.padding.top;

        for child in &self.children {
            let constraints = SizeConstraints::loose(content_width, content_height);
            let size = child.preferred_size(&constraints, font);

            let child_rect = Rect::new(
                bounds.x + self.padding.left,
                y,
                size.width.min(content_width),
                size.height
            );

            child_bounds.push(child_rect);
            y += size.height + self.spacing;
        }

        child_bounds
    }
}

impl Default for VStack {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for VStack {
    fn draw(&self, ctx: &mut DrawContext, bounds: Rect, theme: &Theme, font: &Font) {
        let child_bounds = self.calculate_child_bounds(bounds, font);

        for (child, child_bound) in self.children.iter().zip(child_bounds.iter()) {
            child.draw(ctx, *child_bound, theme, font);
        }
    }

    fn handle_event(&mut self, event: &InputEvent, bounds: Rect) -> EventResponse {
        let child_bounds = self.calculate_child_bounds(bounds, &crate::graphics::create_system_font());

        // Propagate event to children in reverse order (top-most first)
        for (child, child_bound) in self.children.iter_mut().zip(child_bounds.iter()).rev() {
            let response = child.handle_event(event, *child_bound);
            if response != EventResponse::Ignored {
                return response;
            }
        }

        EventResponse::Ignored
    }

    fn preferred_size(&self, constraints: &SizeConstraints, font: &Font) -> Size {
        let mut total_height = self.padding.vertical();
        let mut max_width = 0u32;

        for child in &self.children {
            let child_size = child.preferred_size(constraints, font);
            total_height += child_size.height + self.spacing;
            max_width = max_width.max(child_size.width);
        }

        // Remove last spacing
        if !self.children.is_empty() {
            total_height = total_height.saturating_sub(self.spacing);
        }

        let width = max_width + self.padding.horizontal();
        let (width, height) = constraints.constrain(width, total_height);

        Size::new(width, height)
    }
}

/// Horizontal stack layout - arranges children horizontally
pub struct HStack {
    children: Vec<Box<dyn Widget + Send + Sync>>,
    spacing: u32,
    padding: Padding,
}

impl HStack {
    /// Create a new horizontal stack
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            spacing: 4,
            padding: Padding::zero(),
        }
    }

    /// Set spacing between children
    pub fn with_spacing(mut self, spacing: u32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Set padding
    pub fn with_padding(mut self, padding: Padding) -> Self {
        self.padding = padding;
        self
    }

    /// Add a child widget
    pub fn add_child(&mut self, child: Box<dyn Widget + Send + Sync>) {
        self.children.push(child);
    }

    /// Calculate child bounds for drawing
    fn calculate_child_bounds(&self, bounds: Rect, font: &Font) -> Vec<Rect> {
        let mut child_bounds = Vec::new();

        let content_width = bounds.width.saturating_sub(self.padding.horizontal());
        let content_height = bounds.height.saturating_sub(self.padding.vertical());

        let mut x = bounds.x + self.padding.left;

        for child in &self.children {
            let constraints = SizeConstraints::loose(content_width, content_height);
            let size = child.preferred_size(&constraints, font);

            let child_rect = Rect::new(
                x,
                bounds.y + self.padding.top,
                size.width,
                size.height.min(content_height)
            );

            child_bounds.push(child_rect);
            x += size.width + self.spacing;
        }

        child_bounds
    }
}

impl Default for HStack {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for HStack {
    fn draw(&self, ctx: &mut DrawContext, bounds: Rect, theme: &Theme, font: &Font) {
        let child_bounds = self.calculate_child_bounds(bounds, font);

        for (child, child_bound) in self.children.iter().zip(child_bounds.iter()) {
            child.draw(ctx, *child_bound, theme, font);
        }
    }

    fn handle_event(&mut self, event: &InputEvent, bounds: Rect) -> EventResponse {
        let child_bounds = self.calculate_child_bounds(bounds, &crate::graphics::create_system_font());

        // Propagate event to children in reverse order (top-most first)
        for (child, child_bound) in self.children.iter_mut().zip(child_bounds.iter()).rev() {
            let response = child.handle_event(event, *child_bound);
            if response != EventResponse::Ignored {
                return response;
            }
        }

        EventResponse::Ignored
    }

    fn preferred_size(&self, constraints: &SizeConstraints, font: &Font) -> Size {
        let mut total_width = self.padding.horizontal();
        let mut max_height = 0u32;

        for child in &self.children {
            let child_size = child.preferred_size(constraints, font);
            total_width += child_size.width + self.spacing;
            max_height = max_height.max(child_size.height);
        }

        // Remove last spacing
        if !self.children.is_empty() {
            total_width = total_width.saturating_sub(self.spacing);
        }

        let height = max_height + self.padding.vertical();
        let (width, height) = constraints.constrain(total_width, height);

        Size::new(width, height)
    }
}
