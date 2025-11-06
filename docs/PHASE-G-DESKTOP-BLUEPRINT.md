# Phase G — AI-Native Desktop Environment Blueprint

**Version**: 1.0.0
**Status**: Planning → Implementation
**Timeline**: 18 weeks (4.5 months)
**Target**: Revolutionary AI-first graphical desktop environment
**Last Updated**: 2025-11-06

---

## Vision

Transform SIS OS from a kernel research platform into a **complete, user-facing AI-native operating system** with a desktop environment that:

1. **Showcases AI features visibly** - Make kernel AI decisions transparent and interactive
2. **Feels intelligent** - Proactive, predictive, helpful (JARVIS-like)
3. **Looks professional** - Modern, polished, production-ready UI/UX
4. **Stays lightweight** - Runs smoothly on Raspberry Pi 5 (8GB RAM)
5. **Enables voice/vision** - Infrastructure ready for JARVIS integration

### Core Principle

> "The desktop environment is not just graphics—it's the AI's face to the world."

Traditional OS: User → Desktop → Kernel
**SIS OS**: User ↔ AI Desktop ↔ AI Kernel (bidirectional intelligence)

---

## Table of Contents

- [Architecture Overview](#architecture-overview)
- [Phase G.0: Foundation](#phase-g0-foundation-weeks-1-2)
- [Phase G.1: Window Manager](#phase-g1-window-manager-weeks-3-5)
- [Phase G.2: UI Toolkit](#phase-g2-ui-toolkit-weeks-6-8)
- [Phase G.3: Core Applications](#phase-g3-core-applications-weeks-9-11)
- [Phase G.4: AI Integration UI](#phase-g4-ai-integration-ui-weeks-12-14)
- [Phase G.5: Voice/Vision Infrastructure](#phase-g5-voicevision-infrastructure-weeks-15-16)
- [Phase G.6: Polish & Animations](#phase-g6-polish--animations-weeks-17-18)
- [Technical Stack](#technical-stack)
- [Visual Design System](#visual-design-system)
- [Testing Strategy](#testing-strategy)
- [Integration with Existing Systems](#integration-with-existing-systems)

---

## Architecture Overview

### System Layers

```
┌─────────────────────────────────────────────────────────────┐
│                    User Interaction Layer                   │
│  Mouse/Keyboard/Touch → Event System → Application          │
└──────────────────────┬──────────────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────────────┐
│                   Application Layer                         │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│  │Terminal  │  │AI Insights│  │ Files    │  │Settings  │   │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘   │
└──────────────────────┬──────────────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────────────┐
│                Window Manager Layer                         │
│  Window Composition │ Layout │ Focus │ Z-order             │
└──────────────────────┬──────────────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────────────┐
│                   UI Toolkit Layer                          │
│  Widgets │ Layout Engine │ Event Handlers │ Theming        │
└──────────────────────┬──────────────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────────────┐
│                   Graphics Layer                            │
│  2D Primitives │ Text Rendering │ Image Decoding           │
└──────────────────────┬──────────────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────────────┐
│                 virtio-gpu Driver                           │
│  Framebuffer │ Command Queue │ DMA Transfer                │
└──────────────────────┬──────────────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────────────┐
│                   SIS Kernel (ARM64)                        │
│  AI Features │ Autonomy │ Memory │ Scheduling              │
└─────────────────────────────────────────────────────────────┘
```

### Key Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Rendering** | Software 2D + virtio-gpu | Simple, portable, sufficient for desktop |
| **Window Model** | Tiling + Floating hybrid | Efficient + flexible |
| **UI Toolkit** | Custom lightweight | Full control, AI integration, lightweight |
| **Language** | Rust (no_std where possible) | Safety, performance, consistency |
| **Text Rendering** | Bitmap fonts (RustType) | Fast, simple, embedded-friendly |
| **Graphics API** | Direct framebuffer writes | No GPU complexity, works everywhere |
| **AI Integration** | Kernel syscalls + IPC | Fast, leverages existing AI |
| **Theme System** | Runtime swappable | Dark/light modes, accessibility |

---

## Phase G.0: Foundation (Weeks 1-2)

**Objective**: Get pixels on screen via virtio-gpu, implement basic drawing primitives.

### Deliverables

- ✅ virtio-gpu driver initializes and detects resolution
- ✅ Framebuffer mapped and writable
- ✅ Basic 2D primitives (rect, line, circle, text)
- ✅ Font loading and text rendering
- ✅ Double buffering (no flicker)
- ✅ Mouse cursor rendering

### Implementation

#### 1. virtio-gpu Driver

```rust
// crates/kernel/src/drivers/virtio_gpu.rs

use crate::drivers::virtio::VirtioCommonCfg;

#[repr(C)]
pub struct VirtioGpuConfig {
    events_read: u32,
    events_clear: u32,
    num_scanouts: u32,
    num_capsets: u32,
}

pub struct VirtioGpu {
    common: VirtioCommonCfg,
    control_queue: VirtQueue,
    cursor_queue: VirtQueue,
    framebuffer: *mut u32,         // ARGB8888
    framebuffer_phys: PhysAddr,
    resolution: (u32, u32),         // Width, height
    scanout_id: u32,
    resource_id: u32,
}

impl VirtioGpu {
    pub fn new(mmio_base: usize) -> Result<Self, GpuError> {
        // 1. Initialize virtio common config
        // 2. Create control and cursor queues
        // 3. Negotiate features (F_VIRGL=off, F_EDID=on)
        // 4. Allocate framebuffer memory
        // 5. Create 2D resource
        // 6. Attach backing store
        // 7. Set scanout
    }

    pub fn create_2d_resource(&mut self) -> Result<u32, GpuError> {
        let resource_id = self.next_resource_id();

        let cmd = VirtioGpuResourceCreate2D {
            hdr: VirtioGpuCtrlHdr {
                cmd_type: VIRTIO_GPU_CMD_RESOURCE_CREATE_2D,
                flags: 0,
                fence_id: 0,
                ctx_id: 0,
                padding: 0,
            },
            resource_id,
            format: VIRTIO_GPU_FORMAT_B8G8R8A8_UNORM,  // BGRA
            width: self.resolution.0,
            height: self.resolution.1,
        };

        self.submit_command(&cmd)?;
        Ok(resource_id)
    }

    pub fn attach_backing(&mut self, resource_id: u32) -> Result<(), GpuError> {
        let cmd = VirtioGpuResourceAttachBacking {
            hdr: VirtioGpuCtrlHdr {
                cmd_type: VIRTIO_GPU_CMD_RESOURCE_ATTACH_BACKING,
                flags: 0,
                fence_id: 0,
                ctx_id: 0,
                padding: 0,
            },
            resource_id,
            nr_entries: 1,
        };

        let entry = VirtioGpuMemEntry {
            addr: self.framebuffer_phys as u64,
            length: (self.resolution.0 * self.resolution.1 * 4) as u32,
            padding: 0,
        };

        self.submit_command_with_data(&cmd, &entry)?;
        Ok(())
    }

    pub fn set_scanout(&mut self, resource_id: u32) -> Result<(), GpuError> {
        let cmd = VirtioGpuSetScanout {
            hdr: VirtioGpuCtrlHdr {
                cmd_type: VIRTIO_GPU_CMD_SET_SCANOUT,
                flags: 0,
                fence_id: 0,
                ctx_id: 0,
                padding: 0,
            },
            r: VirtioGpuRect {
                x: 0,
                y: 0,
                width: self.resolution.0,
                height: self.resolution.1,
            },
            scanout_id: self.scanout_id,
            resource_id,
        };

        self.submit_command(&cmd)?;
        Ok(())
    }

    pub fn flush(&mut self, x: u32, y: u32, w: u32, h: u32) -> Result<(), GpuError> {
        let cmd = VirtioGpuResourceFlush {
            hdr: VirtioGpuCtrlHdr {
                cmd_type: VIRTIO_GPU_CMD_RESOURCE_FLUSH,
                flags: 0,
                fence_id: 0,
                ctx_id: 0,
                padding: 0,
            },
            r: VirtioGpuRect { x, y, width: w, height: h },
            resource_id: self.resource_id,
            padding: 0,
        };

        self.submit_command(&cmd)?;
        Ok(())
    }

    pub fn get_framebuffer(&self) -> &mut [u32] {
        let pixel_count = (self.resolution.0 * self.resolution.1) as usize;
        unsafe { core::slice::from_raw_parts_mut(self.framebuffer, pixel_count) }
    }
}
```

#### 2. Graphics Primitives

```rust
// crates/desktop/src/graphics/primitives.rs

pub struct DrawContext {
    framebuffer: &'static mut [u32],
    width: u32,
    height: u32,
    clip_rect: Rect,
}

impl DrawContext {
    pub fn draw_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x >= self.width || y >= self.height {
            return;
        }
        let offset = (y * self.width + x) as usize;
        self.framebuffer[offset] = color.to_argb();
    }

    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        let argb = color.to_argb();
        for y in rect.y..(rect.y + rect.height).min(self.height) {
            for x in rect.x..(rect.x + rect.width).min(self.width) {
                let offset = (y * self.width + x) as usize;
                self.framebuffer[offset] = argb;
            }
        }
    }

    pub fn draw_rect_outline(&mut self, rect: Rect, color: Color, thickness: u32) {
        // Top
        self.fill_rect(Rect { x: rect.x, y: rect.y, width: rect.width, height: thickness }, color);
        // Bottom
        self.fill_rect(Rect { x: rect.x, y: rect.y + rect.height - thickness, width: rect.width, height: thickness }, color);
        // Left
        self.fill_rect(Rect { x: rect.x, y: rect.y, width: thickness, height: rect.height }, color);
        // Right
        self.fill_rect(Rect { x: rect.x + rect.width - thickness, y: rect.y, width: thickness, height: rect.height }, color);
    }

    pub fn draw_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: Color) {
        // Bresenham's line algorithm
        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx - dy;

        let mut x = x0;
        let mut y = y0;

        loop {
            self.draw_pixel(x as u32, y as u32, color);

            if x == x1 && y == y1 {
                break;
            }

            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }
    }

    pub fn draw_circle(&mut self, cx: i32, cy: i32, radius: i32, color: Color) {
        // Midpoint circle algorithm
        let mut x = radius;
        let mut y = 0;
        let mut err = 0;

        while x >= y {
            self.draw_pixel((cx + x) as u32, (cy + y) as u32, color);
            self.draw_pixel((cx + y) as u32, (cy + x) as u32, color);
            self.draw_pixel((cx - y) as u32, (cy + x) as u32, color);
            self.draw_pixel((cx - x) as u32, (cy + y) as u32, color);
            self.draw_pixel((cx - x) as u32, (cy - y) as u32, color);
            self.draw_pixel((cx - y) as u32, (cy - x) as u32, color);
            self.draw_pixel((cx + y) as u32, (cy - x) as u32, color);
            self.draw_pixel((cx + x) as u32, (cy - y) as u32, color);

            if err <= 0 {
                y += 1;
                err += 2 * y + 1;
            }

            if err > 0 {
                x -= 1;
                err -= 2 * x + 1;
            }
        }
    }

    pub fn draw_text(&mut self, x: u32, y: u32, text: &str, font: &Font, color: Color) {
        let mut cursor_x = x;
        for ch in text.chars() {
            if let Some(glyph) = font.get_glyph(ch) {
                self.draw_glyph(cursor_x, y, glyph, color);
                cursor_x += glyph.advance;
            }
        }
    }

    fn draw_glyph(&mut self, x: u32, y: u32, glyph: &Glyph, color: Color) {
        for gy in 0..glyph.height {
            for gx in 0..glyph.width {
                let alpha = glyph.bitmap[(gy * glyph.width + gx) as usize];
                if alpha > 0 {
                    let pixel_color = color.with_alpha(alpha);
                    self.draw_pixel(x + gx, y + gy, pixel_color);
                }
            }
        }
    }
}
```

#### 3. Font Rendering

```rust
// crates/desktop/src/graphics/font.rs

use alloc::collections::BTreeMap;

pub struct Font {
    glyphs: BTreeMap<char, Glyph>,
    size: u32,
    line_height: u32,
}

pub struct Glyph {
    pub bitmap: Vec<u8>,     // Alpha channel only
    pub width: u32,
    pub height: u32,
    pub bearing_x: i32,
    pub bearing_y: i32,
    pub advance: u32,
}

impl Font {
    pub fn from_ttf(data: &[u8], size: u32) -> Result<Self, FontError> {
        // Use RustType or similar to parse TTF
        // Rasterize common glyphs (ASCII + some Unicode)
        // Store as pre-rendered bitmaps
    }

    pub fn from_bitmap(data: &[u8]) -> Result<Self, FontError> {
        // Load pre-rendered bitmap font (PSF2 format)
        // Simpler, faster, good for fixed-size console fonts
    }

    pub fn get_glyph(&self, ch: char) -> Option<&Glyph> {
        self.glyphs.get(&ch)
    }

    pub fn measure_text(&self, text: &str) -> (u32, u32) {
        let width: u32 = text.chars()
            .filter_map(|ch| self.get_glyph(ch))
            .map(|g| g.advance)
            .sum();
        (width, self.line_height)
    }
}
```

#### 4. Input Handling

```rust
// crates/desktop/src/input/mod.rs

pub struct InputManager {
    mouse_x: i32,
    mouse_y: i32,
    mouse_buttons: u8,      // Bit 0=left, 1=right, 2=middle
    keyboard_state: [bool; 256],
    event_queue: VecDeque<InputEvent>,
}

pub enum InputEvent {
    MouseMove { x: i32, y: i32 },
    MouseButton { button: MouseButton, pressed: bool },
    KeyPress { key: KeyCode, modifiers: KeyModifiers },
    KeyRelease { key: KeyCode },
}

pub enum MouseButton {
    Left = 0,
    Right = 1,
    Middle = 2,
}

impl InputManager {
    pub fn poll_events(&mut self) -> impl Iterator<Item = &InputEvent> {
        self.event_queue.iter()
    }

    pub fn clear_events(&mut self) {
        self.event_queue.clear();
    }

    pub fn handle_virtio_input(&mut self, data: &[u8]) {
        // Parse virtio-input event data
        // Convert to InputEvent
        // Add to queue
    }
}
```

### Acceptance Tests

```bash
# tests/phase_g0/test_graphics.sh

# Test 1: virtio-gpu initializes
test_gpu_init() {
    qemu-system-aarch64 ... -device virtio-gpu-pci -display gtk
    # Check: QEMU window opens
    # Check: Resolution detected (1280x720)
    # Check: Framebuffer allocated
}

# Test 2: Draw primitives
test_draw_primitives() {
    # Draw red rectangle at (100, 100), 200x150
    # Check: Pixels correctly written
    # Check: No out-of-bounds crashes
}

# Test 3: Text rendering
test_text_rendering() {
    # Render "Hello SIS OS" at (10, 10)
    # Check: Glyphs visible
    # Check: Text readable
}

# Test 4: Mouse cursor
test_mouse_cursor() {
    # Move mouse around
    # Check: Cursor follows mouse
    # Check: No lag or flicker
}
```

### Exit Criteria

- ✅ QEMU window opens showing SIS OS graphics
- ✅ Can draw colored rectangles, lines, circles
- ✅ Text renders correctly with bitmap font
- ✅ Mouse cursor visible and responsive
- ✅ No visual artifacts or flicker
- ✅ Framerate: 60 FPS

---

## Phase G.1: Window Manager (Weeks 3-5)

**Objective**: Implement tiling + floating window manager with decorations, focus, and Z-ordering.

### Deliverables

- ✅ Window creation, destruction, and lifecycle
- ✅ Window decorations (title bar, borders, close button)
- ✅ Focus management (click to focus, Alt+Tab)
- ✅ Tiling layout (auto-arrange windows)
- ✅ Floating mode (manual positioning)
- ✅ Window resizing and dragging
- ✅ Z-ordering (bring to front)

### Implementation

#### Window Manager Architecture

```rust
// crates/desktop/src/window_manager/mod.rs

pub struct WindowManager {
    windows: Vec<Window>,
    focused_id: Option<WindowId>,
    layout_mode: LayoutMode,
    screen_rect: Rect,
    next_window_id: u32,
}

pub struct Window {
    pub id: WindowId,
    pub title: String,
    pub bounds: Rect,
    pub content_bounds: Rect,      // Excludes decorations
    pub framebuffer: Vec<u32>,
    pub visible: bool,
    pub focused: bool,
    pub minimized: bool,
    pub fullscreen: bool,
    pub resizable: bool,
    pub movable: bool,
    pub closable: bool,
    pub decoration: WindowDecoration,
    pub z_order: u32,
    pub app_data: Option<Box<dyn Any>>,
}

pub enum LayoutMode {
    Tiling,         // Auto-arrange (like i3wm)
    Floating,       // Manual positioning (like Windows)
    Fullscreen,     // Single fullscreen window
}

pub struct WindowDecoration {
    pub title_bar_height: u32,
    pub border_width: u32,
    pub show_title_bar: bool,
    pub show_close_button: bool,
    pub show_minimize_button: bool,
    pub show_maximize_button: bool,
}

impl WindowManager {
    pub fn create_window(&mut self, spec: WindowSpec) -> WindowId {
        let id = self.next_window_id();

        let (bounds, content_bounds) = match self.layout_mode {
            LayoutMode::Tiling => self.calculate_tiling_bounds(),
            LayoutMode::Floating => (spec.bounds, spec.bounds.shrink_by_decoration()),
            LayoutMode::Fullscreen => (self.screen_rect, self.screen_rect),
        };

        let window = Window {
            id,
            title: spec.title,
            bounds,
            content_bounds,
            framebuffer: vec![0; (bounds.width * bounds.height) as usize],
            visible: true,
            focused: false,
            minimized: false,
            fullscreen: false,
            resizable: spec.resizable,
            movable: spec.movable,
            closable: spec.closable,
            decoration: spec.decoration,
            z_order: self.next_z_order(),
            app_data: None,
        };

        self.windows.push(window);
        self.relayout();
        self.focus_window(id);

        id
    }

    pub fn destroy_window(&mut self, id: WindowId) {
        self.windows.retain(|w| w.id != id);
        if self.focused_id == Some(id) {
            self.focus_next();
        }
        self.relayout();
    }

    pub fn focus_window(&mut self, id: WindowId) {
        // Unfocus all windows
        for window in &mut self.windows {
            window.focused = false;
        }

        // Focus target window and bring to front
        if let Some(window) = self.windows.iter_mut().find(|w| w.id == id) {
            window.focused = true;
            window.z_order = self.next_z_order();
        }

        self.focused_id = Some(id);
    }

    pub fn handle_click(&mut self, x: u32, y: u32) -> Option<WindowId> {
        // Find topmost window at click position (highest z_order)
        let mut clicked_window = None;
        let mut max_z = 0;

        for window in &self.windows {
            if window.bounds.contains(x, y) && window.z_order >= max_z {
                clicked_window = Some(window.id);
                max_z = window.z_order;
            }
        }

        if let Some(id) = clicked_window {
            // Check if clicked on title bar close button
            if let Some(window) = self.get_window(id) {
                let close_button_rect = self.get_close_button_rect(window);
                if close_button_rect.contains(x, y) {
                    self.destroy_window(id);
                    return None;
                }
            }

            self.focus_window(id);
            return Some(id);
        }

        None
    }

    pub fn relayout(&mut self) {
        match self.layout_mode {
            LayoutMode::Tiling => self.relayout_tiling(),
            LayoutMode::Floating => { /* Manual positioning */ },
            LayoutMode::Fullscreen => self.relayout_fullscreen(),
        }
    }

    fn relayout_tiling(&mut self) {
        let visible_windows: Vec<_> = self.windows.iter().filter(|w| w.visible && !w.minimized).collect();
        let count = visible_windows.len();

        if count == 0 {
            return;
        }

        // Simple tiling: divide screen vertically
        let window_height = self.screen_rect.height / count as u32;

        for (i, window) in self.windows.iter_mut().filter(|w| w.visible && !w.minimized).enumerate() {
            window.bounds = Rect {
                x: 0,
                y: (i as u32) * window_height,
                width: self.screen_rect.width,
                height: window_height,
            };
            window.content_bounds = window.bounds.shrink_by(30, 2);  // Title bar + borders
        }
    }

    pub fn draw(&self, ctx: &mut DrawContext) {
        // Sort windows by z_order (lowest first)
        let mut sorted_windows: Vec<_> = self.windows.iter().collect();
        sorted_windows.sort_by_key(|w| w.z_order);

        for window in sorted_windows {
            if !window.visible {
                continue;
            }

            self.draw_window(window, ctx);
        }
    }

    fn draw_window(&self, window: &Window, ctx: &mut DrawContext) {
        // Draw window background
        ctx.fill_rect(window.bounds, Color::from_rgb(45, 45, 48));

        // Draw title bar
        if window.decoration.show_title_bar {
            let title_bar_color = if window.focused {
                Color::from_rgb(0, 122, 204)  // Blue for focused
            } else {
                Color::from_rgb(80, 80, 80)   // Gray for unfocused
            };

            let title_bar_rect = Rect {
                x: window.bounds.x,
                y: window.bounds.y,
                width: window.bounds.width,
                height: window.decoration.title_bar_height,
            };

            ctx.fill_rect(title_bar_rect, title_bar_color);

            // Draw title text
            let title_x = window.bounds.x + 10;
            let title_y = window.bounds.y + 8;
            ctx.draw_text(title_x, title_y, &window.title, &SYSTEM_FONT, Color::WHITE);

            // Draw close button
            if window.decoration.show_close_button {
                let close_rect = self.get_close_button_rect(window);
                ctx.fill_rect(close_rect, Color::from_rgb(200, 0, 0));
                ctx.draw_text(close_rect.x + 8, close_rect.y + 4, "X", &SYSTEM_FONT, Color::WHITE);
            }
        }

        // Draw border
        if window.decoration.border_width > 0 {
            let border_color = if window.focused {
                Color::from_rgb(0, 122, 204)
            } else {
                Color::from_rgb(60, 60, 60)
            };
            ctx.draw_rect_outline(window.bounds, border_color, window.decoration.border_width);
        }

        // Blit window content from window framebuffer
        self.blit_window_content(window, ctx);
    }

    fn blit_window_content(&self, window: &Window, ctx: &mut DrawContext) {
        let src_width = window.content_bounds.width;
        let src_height = window.content_bounds.height;

        for y in 0..src_height {
            for x in 0..src_width {
                let src_offset = (y * src_width + x) as usize;
                let dst_x = window.content_bounds.x + x;
                let dst_y = window.content_bounds.y + y;

                if src_offset < window.framebuffer.len() {
                    let color = Color::from_argb(window.framebuffer[src_offset]);
                    ctx.draw_pixel(dst_x, dst_y, color);
                }
            }
        }
    }
}
```

### Keyboard Shortcuts

```rust
// crates/desktop/src/window_manager/shortcuts.rs

pub enum Shortcut {
    FocusNext,          // Alt+Tab
    FocusPrev,          // Alt+Shift+Tab
    CloseWindow,        // Alt+F4
    FullscreenToggle,   // Alt+F11
    LayoutToggle,       // Super+T (Tiling/Floating)
    NewWindow,          // Super+Enter
}

impl WindowManager {
    pub fn handle_shortcut(&mut self, shortcut: Shortcut) {
        match shortcut {
            Shortcut::FocusNext => self.focus_next(),
            Shortcut::FocusPrev => self.focus_prev(),
            Shortcut::CloseWindow => {
                if let Some(id) = self.focused_id {
                    self.destroy_window(id);
                }
            }
            Shortcut::FullscreenToggle => self.toggle_fullscreen(),
            Shortcut::LayoutToggle => {
                self.layout_mode = match self.layout_mode {
                    LayoutMode::Tiling => LayoutMode::Floating,
                    LayoutMode::Floating => LayoutMode::Tiling,
                    LayoutMode::Fullscreen => LayoutMode::Tiling,
                };
                self.relayout();
            }
            Shortcut::NewWindow => { /* Application-specific */ }
        }
    }

    fn focus_next(&mut self) {
        let visible: Vec<_> = self.windows.iter().filter(|w| w.visible).map(|w| w.id).collect();
        if visible.is_empty() {
            return;
        }

        if let Some(current_id) = self.focused_id {
            if let Some(index) = visible.iter().position(|&id| id == current_id) {
                let next_index = (index + 1) % visible.len();
                self.focus_window(visible[next_index]);
                return;
            }
        }

        self.focus_window(visible[0]);
    }
}
```

### Acceptance Tests

```bash
# tests/phase_g1/test_window_manager.sh

# Test 1: Create window
test_create_window() {
    # Create window "Terminal"
    # Check: Window appears with title bar
    # Check: Window has borders
    # Check: Close button visible
}

# Test 2: Focus management
test_focus() {
    # Create 2 windows
    # Click on window 2
    # Check: Window 2 has focus (blue title bar)
    # Check: Window 1 loses focus (gray title bar)
}

# Test 3: Alt+Tab
test_alt_tab() {
    # Create 3 windows
    # Press Alt+Tab twice
    # Check: Focus cycles through windows
}

# Test 4: Tiling layout
test_tiling() {
    # Create 3 windows in tiling mode
    # Check: Windows divide screen evenly
    # Create 4th window
    # Check: All windows resize to fit
}

# Test 5: Close window
test_close() {
    # Create window
    # Click close button
    # Check: Window disappears
    # Check: No crash
}
```

### Exit Criteria

- ✅ Windows can be created, focused, and closed
- ✅ Window decorations render correctly
- ✅ Alt+Tab cycles through windows
- ✅ Tiling layout auto-arranges windows
- ✅ Floating mode allows manual positioning
- ✅ Mouse interactions work correctly
- ✅ No visual glitches

---

## Phase G.2: UI Toolkit (Weeks 6-8)

**Objective**: Build reusable widget library for constructing UIs.

### Deliverables

- ✅ Widget trait and base framework
- ✅ Core widgets: Button, Label, TextBox, Panel, ScrollView
- ✅ Layout system: Grid, Stack, Flex
- ✅ Event system: Click, hover, focus
- ✅ Theme support: Colors, fonts, spacing
- ✅ Basic animations: Fade, slide, scale

### Widget Architecture

```rust
// crates/desktop/src/ui/widget.rs

pub trait Widget {
    /// Draw the widget
    fn draw(&self, ctx: &mut DrawContext, bounds: Rect);

    /// Handle input event, return true if event was consumed
    fn handle_event(&mut self, event: &InputEvent, bounds: Rect) -> EventResponse;

    /// Calculate preferred size
    fn preferred_size(&self, constraints: SizeConstraints) -> Size;

    /// Called when widget gains/loses focus
    fn set_focused(&mut self, focused: bool);

    /// Called when widget becomes visible/hidden
    fn set_visible(&mut self, visible: bool);
}

pub enum EventResponse {
    Consumed,      // Event handled, don't propagate
    Ignored,       // Event not handled, continue propagating
    RequestRedraw, // Event handled, request redraw
}

pub struct SizeConstraints {
    pub min_width: u32,
    pub max_width: u32,
    pub min_height: u32,
    pub max_height: u32,
}
```

### Core Widgets

#### Button

```rust
pub struct Button {
    label: String,
    on_click: Option<Box<dyn Fn()>>,
    hovered: bool,
    pressed: bool,
    enabled: bool,
    style: ButtonStyle,
}

pub struct ButtonStyle {
    pub bg_color: Color,
    pub bg_color_hover: Color,
    pub bg_color_pressed: Color,
    pub text_color: Color,
    pub border_radius: u32,
    pub padding: u32,
}

impl Widget for Button {
    fn draw(&self, ctx: &mut DrawContext, bounds: Rect) {
        let bg_color = if self.pressed {
            self.style.bg_color_pressed
        } else if self.hovered {
            self.style.bg_color_hover
        } else {
            self.style.bg_color
        };

        ctx.fill_rect(bounds, bg_color);

        let text_size = ctx.measure_text(&self.label);
        let text_x = bounds.x + (bounds.width - text_size.0) / 2;
        let text_y = bounds.y + (bounds.height - text_size.1) / 2;

        ctx.draw_text(text_x, text_y, &self.label, &SYSTEM_FONT, self.style.text_color);
    }

    fn handle_event(&mut self, event: &InputEvent, bounds: Rect) -> EventResponse {
        match event {
            InputEvent::MouseMove { x, y } => {
                let was_hovered = self.hovered;
                self.hovered = bounds.contains(*x as u32, *y as u32);
                if was_hovered != self.hovered {
                    EventResponse::RequestRedraw
                } else {
                    EventResponse::Ignored
                }
            }
            InputEvent::MouseButton { button: MouseButton::Left, pressed: true } => {
                if self.hovered {
                    self.pressed = true;
                    EventResponse::RequestRedraw
                } else {
                    EventResponse::Ignored
                }
            }
            InputEvent::MouseButton { button: MouseButton::Left, pressed: false } => {
                if self.pressed && self.hovered {
                    if let Some(callback) = &self.on_click {
                        callback();
                    }
                }
                self.pressed = false;
                EventResponse::RequestRedraw
            }
            _ => EventResponse::Ignored,
        }
    }

    fn preferred_size(&self, _constraints: SizeConstraints) -> Size {
        let text_size = measure_text(&self.label);
        Size {
            width: text_size.0 + self.style.padding * 2,
            height: text_size.1 + self.style.padding * 2,
        }
    }

    fn set_focused(&mut self, _focused: bool) {}
    fn set_visible(&mut self, _visible: bool) {}
}
```

#### TextBox

```rust
pub struct TextBox {
    text: String,
    placeholder: String,
    cursor_pos: usize,
    selection_start: Option<usize>,
    focused: bool,
    editable: bool,
    multiline: bool,
    on_change: Option<Box<dyn Fn(&str)>>,
    style: TextBoxStyle,
}

impl Widget for TextBox {
    fn draw(&self, ctx: &mut DrawContext, bounds: Rect) {
        // Draw background
        let bg_color = if self.focused {
            self.style.bg_color_focused
        } else {
            self.style.bg_color
        };
        ctx.fill_rect(bounds, bg_color);

        // Draw border
        let border_color = if self.focused {
            self.style.border_color_focused
        } else {
            self.style.border_color
        };
        ctx.draw_rect_outline(bounds, border_color, 2);

        // Draw text or placeholder
        let display_text = if self.text.is_empty() {
            &self.placeholder
        } else {
            &self.text
        };

        let text_color = if self.text.is_empty() {
            Color::from_rgb(128, 128, 128)  // Gray for placeholder
        } else {
            self.style.text_color
        };

        ctx.draw_text(
            bounds.x + self.style.padding,
            bounds.y + self.style.padding,
            display_text,
            &SYSTEM_FONT,
            text_color,
        );

        // Draw cursor if focused
        if self.focused {
            let cursor_x = bounds.x + self.style.padding + self.calculate_cursor_offset();
            ctx.draw_line(
                cursor_x as i32,
                bounds.y as i32 + self.style.padding as i32,
                cursor_x as i32,
                bounds.y as i32 + bounds.height as i32 - self.style.padding as i32,
                Color::WHITE,
            );
        }
    }

    fn handle_event(&mut self, event: &InputEvent, bounds: Rect) -> EventResponse {
        match event {
            InputEvent::MouseButton { button: MouseButton::Left, pressed: true } => {
                // Set cursor position based on click location
                EventResponse::RequestRedraw
            }
            InputEvent::KeyPress { key, modifiers } => {
                if !self.focused || !self.editable {
                    return EventResponse::Ignored;
                }

                match key {
                    KeyCode::Backspace => {
                        if self.cursor_pos > 0 {
                            self.text.remove(self.cursor_pos - 1);
                            self.cursor_pos -= 1;
                            if let Some(callback) = &self.on_change {
                                callback(&self.text);
                            }
                        }
                    }
                    KeyCode::Delete => {
                        if self.cursor_pos < self.text.len() {
                            self.text.remove(self.cursor_pos);
                            if let Some(callback) = &self.on_change {
                                callback(&self.text);
                            }
                        }
                    }
                    KeyCode::Left => {
                        if self.cursor_pos > 0 {
                            self.cursor_pos -= 1;
                        }
                    }
                    KeyCode::Right => {
                        if self.cursor_pos < self.text.len() {
                            self.cursor_pos += 1;
                        }
                    }
                    KeyCode::Char(ch) => {
                        self.text.insert(self.cursor_pos, *ch);
                        self.cursor_pos += 1;
                        if let Some(callback) = &self.on_change {
                            callback(&self.text);
                        }
                    }
                    _ => return EventResponse::Ignored,
                }

                EventResponse::RequestRedraw
            }
            _ => EventResponse::Ignored,
        }
    }

    fn preferred_size(&self, constraints: SizeConstraints) -> Size {
        Size {
            width: constraints.max_width,
            height: if self.multiline { 100 } else { 30 },
        }
    }

    fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    fn set_visible(&mut self, _visible: bool) {}
}
```

#### Panel (Container)

```rust
pub struct Panel {
    children: Vec<Box<dyn Widget>>,
    layout: Layout,
    background: Option<Color>,
    padding: u32,
    spacing: u32,
}

pub enum Layout {
    Vertical,    // Stack vertically
    Horizontal,  // Stack horizontally
    Grid { cols: u32 },  // Grid layout
}

impl Widget for Panel {
    fn draw(&self, ctx: &mut DrawContext, bounds: Rect) {
        // Draw background if specified
        if let Some(bg_color) = self.background {
            ctx.fill_rect(bounds, bg_color);
        }

        // Calculate child bounds based on layout
        let child_bounds = self.calculate_child_bounds(bounds);

        // Draw children
        for (child, &child_rect) in self.children.iter().zip(child_bounds.iter()) {
            child.draw(ctx, child_rect);
        }
    }

    fn handle_event(&mut self, event: &InputEvent, bounds: Rect) -> EventResponse {
        let child_bounds = self.calculate_child_bounds(bounds);

        // Propagate event to children (reverse order for z-ordering)
        for (child, &child_rect) in self.children.iter_mut().zip(child_bounds.iter()).rev() {
            match child.handle_event(event, child_rect) {
                EventResponse::Consumed => return EventResponse::Consumed,
                EventResponse::RequestRedraw => return EventResponse::RequestRedraw,
                EventResponse::Ignored => continue,
            }
        }

        EventResponse::Ignored
    }

    fn preferred_size(&self, constraints: SizeConstraints) -> Size {
        match self.layout {
            Layout::Vertical => {
                let mut total_height = self.padding * 2;
                let mut max_width = 0;

                for child in &self.children {
                    let child_size = child.preferred_size(constraints);
                    total_height += child_size.height + self.spacing;
                    max_width = max_width.max(child_size.width);
                }

                Size { width: max_width + self.padding * 2, height: total_height }
            }
            Layout::Horizontal => {
                let mut total_width = self.padding * 2;
                let mut max_height = 0;

                for child in &self.children {
                    let child_size = child.preferred_size(constraints);
                    total_width += child_size.width + self.spacing;
                    max_height = max_height.max(child_size.height);
                }

                Size { width: total_width, height: max_height + self.padding * 2 }
            }
            Layout::Grid { cols } => {
                // Calculate grid size
                unimplemented!()
            }
        }
    }

    fn set_focused(&mut self, _focused: bool) {}
    fn set_visible(&mut self, _visible: bool) {}
}

impl Panel {
    fn calculate_child_bounds(&self, bounds: Rect) -> Vec<Rect> {
        let mut result = Vec::new();

        match self.layout {
            Layout::Vertical => {
                let mut y = bounds.y + self.padding;
                for child in &self.children {
                    let child_size = child.preferred_size(SizeConstraints {
                        min_width: 0,
                        max_width: bounds.width - self.padding * 2,
                        min_height: 0,
                        max_height: u32::MAX,
                    });

                    result.push(Rect {
                        x: bounds.x + self.padding,
                        y,
                        width: child_size.width,
                        height: child_size.height,
                    });

                    y += child_size.height + self.spacing;
                }
            }
            Layout::Horizontal => {
                let mut x = bounds.x + self.padding;
                for child in &self.children {
                    let child_size = child.preferred_size(SizeConstraints {
                        min_width: 0,
                        max_width: u32::MAX,
                        min_height: 0,
                        max_height: bounds.height - self.padding * 2,
                    });

                    result.push(Rect {
                        x,
                        y: bounds.y + self.padding,
                        width: child_size.width,
                        height: child_size.height,
                    });

                    x += child_size.width + self.spacing;
                }
            }
            Layout::Grid { .. } => unimplemented!(),
        }

        result
    }
}
```

### Theme System

```rust
// crates/desktop/src/ui/theme.rs

pub struct Theme {
    pub name: String,
    pub colors: ColorScheme,
    pub fonts: FontSet,
    pub spacing: Spacing,
}

pub struct ColorScheme {
    pub background: Color,
    pub foreground: Color,
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub error: Color,
    pub warning: Color,
    pub success: Color,
    pub border: Color,
    pub shadow: Color,
}

pub struct FontSet {
    pub default: Font,
    pub heading: Font,
    pub monospace: Font,
}

pub struct Spacing {
    pub xs: u32,   // 4px
    pub sm: u32,   // 8px
    pub md: u32,   // 16px
    pub lg: u32,   // 24px
    pub xl: u32,   // 32px
}

// Predefined themes
pub fn dark_theme() -> Theme {
    Theme {
        name: "SIS Dark".to_string(),
        colors: ColorScheme {
            background: Color::from_rgb(30, 30, 30),
            foreground: Color::from_rgb(255, 255, 255),
            primary: Color::from_rgb(0, 122, 204),
            secondary: Color::from_rgb(80, 80, 80),
            accent: Color::from_rgb(255, 185, 0),
            error: Color::from_rgb(232, 17, 35),
            warning: Color::from_rgb(255, 185, 0),
            success: Color::from_rgb(16, 124, 16),
            border: Color::from_rgb(60, 60, 60),
            shadow: Color::from_rgba(0, 0, 0, 128),
        },
        fonts: FontSet {
            default: Font::from_bitmap(SYSTEM_FONT_DATA).unwrap(),
            heading: Font::from_bitmap(HEADING_FONT_DATA).unwrap(),
            monospace: Font::from_bitmap(MONO_FONT_DATA).unwrap(),
        },
        spacing: Spacing {
            xs: 4,
            sm: 8,
            md: 16,
            lg: 24,
            xl: 32,
        },
    }
}

pub fn light_theme() -> Theme {
    Theme {
        name: "SIS Light".to_string(),
        colors: ColorScheme {
            background: Color::from_rgb(255, 255, 255),
            foreground: Color::from_rgb(0, 0, 0),
            primary: Color::from_rgb(0, 120, 212),
            secondary: Color::from_rgb(200, 200, 200),
            accent: Color::from_rgb(255, 140, 0),
            error: Color::from_rgb(232, 17, 35),
            warning: Color::from_rgb(255, 140, 0),
            success: Color::from_rgb(16, 124, 16),
            border: Color::from_rgb(204, 204, 204),
            shadow: Color::from_rgba(0, 0, 0, 64),
        },
        fonts: FontSet {
            default: Font::from_bitmap(SYSTEM_FONT_DATA).unwrap(),
            heading: Font::from_bitmap(HEADING_FONT_DATA).unwrap(),
            monospace: Font::from_bitmap(MONO_FONT_DATA).unwrap(),
        },
        spacing: Spacing {
            xs: 4,
            sm: 8,
            md: 16,
            lg: 24,
            xl: 32,
        },
    }
}
```

### Acceptance Tests

```bash
# tests/phase_g2/test_ui_toolkit.sh

# Test 1: Button
test_button() {
    # Render button
    # Check: Button visible with label
    # Click button
    # Check: Callback fired
    # Check: Visual feedback (hover, pressed states)
}

# Test 2: TextBox
test_textbox() {
    # Render textbox
    # Type text
    # Check: Text appears in box
    # Check: Cursor visible and moves
    # Check: Backspace works
}

# Test 3: Panel layout
test_panel() {
    # Create panel with 3 buttons (vertical layout)
    # Check: Buttons stacked vertically
    # Check: Spacing consistent
}

# Test 4: Theme switching
test_theme() {
    # Apply dark theme
    # Check: Colors match dark scheme
    # Apply light theme
    # Check: Colors match light scheme
}
```

### Exit Criteria

- ✅ All core widgets (Button, Label, TextBox, Panel) functional
- ✅ Layout system correctly arranges widgets
- ✅ Events propagate correctly through widget tree
- ✅ Themes can be switched at runtime
- ✅ UI feels responsive (no lag)

---

## Phase G.3: Core Applications (Weeks 9-11)

**Objective**: Build essential applications that showcase SIS OS capabilities.

### Deliverables

- ✅ Terminal (native version of web GUI)
- ✅ AI Insights Dashboard
- ✅ System Monitor
- ✅ File Manager
- ✅ Settings & AI Control Panel

### 1. Terminal Application

```rust
// crates/desktop/src/apps/terminal.rs

pub struct TerminalApp {
    window_id: WindowId,
    command_history: Vec<String>,
    output_buffer: Vec<String>,
    current_input: String,
    cursor_pos: usize,
    scroll_offset: usize,
}

impl TerminalApp {
    pub fn new(wm: &mut WindowManager) -> Self {
        let window_id = wm.create_window(WindowSpec {
            title: "SIS Terminal".to_string(),
            bounds: Rect { x: 100, y: 100, width: 800, height: 600 },
            resizable: true,
            movable: true,
            closable: true,
            decoration: WindowDecoration::default(),
        });

        Self {
            window_id,
            command_history: Vec::new(),
            output_buffer: vec!["=== SIS Kernel Shell ===".to_string()],
            current_input: String::new(),
            cursor_pos: 0,
            scroll_offset: 0,
        }
    }

    pub fn execute_command(&mut self, cmd: &str) {
        self.command_history.push(cmd.to_string());
        self.output_buffer.push(format!("sis> {}", cmd));

        // Execute command via kernel shell
        match kernel_shell_exec(cmd) {
            Ok(output) => self.output_buffer.push(output),
            Err(e) => self.output_buffer.push(format!("Error: {}", e)),
        }
    }

    pub fn draw(&self, ctx: &mut DrawContext, bounds: Rect) {
        // Draw terminal background (black)
        ctx.fill_rect(bounds, Color::from_rgb(0, 0, 0));

        // Draw output buffer (scrolled)
        let line_height = 20;
        let visible_lines = (bounds.height / line_height) as usize;
        let start_line = self.scroll_offset;
        let end_line = (start_line + visible_lines).min(self.output_buffer.len());

        for (i, line) in self.output_buffer[start_line..end_line].iter().enumerate() {
            let y = bounds.y + (i as u32) * line_height;
            ctx.draw_text(bounds.x + 10, y + 5, line, &MONO_FONT, Color::from_rgb(0, 255, 0));
        }

        // Draw current input line
        let input_y = bounds.y + bounds.height - 30;
        let prompt = format!("sis> {}", self.current_input);
        ctx.draw_text(bounds.x + 10, input_y, &prompt, &MONO_FONT, Color::from_rgb(0, 255, 0));

        // Draw cursor
        let cursor_x = bounds.x + 10 + measure_text(&format!("sis> {}", &self.current_input[..self.cursor_pos])).0;
        ctx.draw_line(
            cursor_x as i32,
            input_y as i32,
            cursor_x as i32,
            (input_y + line_height) as i32,
            Color::from_rgb(0, 255, 0),
        );
    }

    pub fn handle_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Enter => {
                let cmd = self.current_input.clone();
                self.current_input.clear();
                self.cursor_pos = 0;
                self.execute_command(&cmd);
                self.scroll_to_bottom();
            }
            KeyCode::Backspace => {
                if self.cursor_pos > 0 {
                    self.current_input.remove(self.cursor_pos - 1);
                    self.cursor_pos -= 1;
                }
            }
            KeyCode::Left => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                }
            }
            KeyCode::Right => {
                if self.cursor_pos < self.current_input.len() {
                    self.cursor_pos += 1;
                }
            }
            KeyCode::Up => {
                // Command history (up)
                if let Some(prev_cmd) = self.command_history.last() {
                    self.current_input = prev_cmd.clone();
                    self.cursor_pos = self.current_input.len();
                }
            }
            KeyCode::Char(ch) => {
                self.current_input.insert(self.cursor_pos, ch);
                self.cursor_pos += 1;
            }
            _ => {}
        }
    }
}
```

### 2. AI Insights Dashboard

```rust
// crates/desktop/src/apps/ai_dashboard.rs

pub struct AIDashboardApp {
    window_id: WindowId,
    decision_log: Vec<AIDecision>,
    metrics: SystemMetrics,
    selected_decision: Option<usize>,
}

pub struct AIDecision {
    timestamp: u64,
    action: String,
    reasoning: Vec<String>,
    confidence: f32,
    outcome: Option<DecisionOutcome>,
}

pub enum DecisionOutcome {
    Success,
    Failed(String),
    Pending,
}

impl AIDashboardApp {
    pub fn draw(&self, ctx: &mut DrawContext, bounds: Rect) {
        let theme = get_current_theme();

        // Draw header
        ctx.fill_rect(
            Rect { x: bounds.x, y: bounds.y, width: bounds.width, height: 60 },
            theme.colors.primary,
        );
        ctx.draw_text(bounds.x + 20, bounds.y + 20, "AI Insights Dashboard", &theme.fonts.heading, Color::WHITE);

        // Draw stats panel (top right)
        self.draw_stats_panel(ctx, Rect {
            x: bounds.x + bounds.width - 300,
            y: bounds.y + 80,
            width: 280,
            height: 200,
        });

        // Draw decision log (left side)
        self.draw_decision_log(ctx, Rect {
            x: bounds.x + 20,
            y: bounds.y + 80,
            width: bounds.width - 340,
            height: bounds.height - 100,
        });

        // Draw selected decision details (if any)
        if let Some(index) = self.selected_decision {
            self.draw_decision_details(ctx, &self.decision_log[index], Rect {
                x: bounds.x + bounds.width - 300,
                y: bounds.y + 300,
                width: 280,
                height: bounds.height - 320,
            });
        }
    }

    fn draw_stats_panel(&self, ctx: &mut DrawContext, bounds: Rect) {
        let theme = get_current_theme();

        ctx.fill_rect(bounds, theme.colors.secondary);
        ctx.draw_rect_outline(bounds, theme.colors.border, 1);

        ctx.draw_text(bounds.x + 10, bounds.y + 10, "AI Statistics", &theme.fonts.heading, theme.colors.foreground);

        let stats = [
            format!("Total Decisions: {}", self.decision_log.len()),
            format!("Today: {}", self.count_decisions_today()),
            format!("Success Rate: {:.1}%", self.calculate_success_rate()),
            format!("Avg Confidence: {:.1}%", self.calculate_avg_confidence()),
        ];

        for (i, stat) in stats.iter().enumerate() {
            ctx.draw_text(
                bounds.x + 10,
                bounds.y + 40 + (i as u32 * 30),
                stat,
                &theme.fonts.default,
                theme.colors.foreground,
            );
        }
    }

    fn draw_decision_log(&self, ctx: &mut DrawContext, bounds: Rect) {
        let theme = get_current_theme();

        ctx.fill_rect(bounds, theme.colors.background);
        ctx.draw_rect_outline(bounds, theme.colors.border, 1);

        ctx.draw_text(bounds.x + 10, bounds.y + 10, "Recent Decisions", &theme.fonts.heading, theme.colors.foreground);

        let line_height = 60;
        for (i, decision) in self.decision_log.iter().rev().take(8).enumerate() {
            let y = bounds.y + 50 + (i as u32 * line_height);
            let is_selected = self.selected_decision == Some(self.decision_log.len() - 1 - i);

            let bg_color = if is_selected {
                theme.colors.primary.with_alpha(64)
            } else {
                theme.colors.background
            };

            ctx.fill_rect(Rect { x: bounds.x + 10, y, width: bounds.width - 20, height: line_height - 5 }, bg_color);

            // Draw decision icon based on outcome
            let icon = match decision.outcome {
                Some(DecisionOutcome::Success) => "✓",
                Some(DecisionOutcome::Failed(_)) => "✗",
                None => "●",
            };
            let icon_color = match decision.outcome {
                Some(DecisionOutcome::Success) => theme.colors.success,
                Some(DecisionOutcome::Failed(_)) => theme.colors.error,
                None => theme.colors.warning,
            };
            ctx.draw_text(bounds.x + 20, y + 5, icon, &theme.fonts.heading, icon_color);

            // Draw decision summary
            ctx.draw_text(bounds.x + 50, y + 5, &decision.action, &theme.fonts.default, theme.colors.foreground);

            // Draw confidence bar
            let confidence_width = (200.0 * decision.confidence) as u32;
            ctx.fill_rect(
                Rect { x: bounds.x + 50, y: y + 30, width: confidence_width, height: 10 },
                theme.colors.accent,
            );
            ctx.draw_text(
                bounds.x + 260,
                y + 27,
                &format!("{:.0}%", decision.confidence * 100.0),
                &theme.fonts.default,
                theme.colors.foreground,
            );
        }
    }

    fn draw_decision_details(&self, ctx: &mut DrawContext, decision: &AIDecision, bounds: Rect) {
        let theme = get_current_theme();

        ctx.fill_rect(bounds, theme.colors.secondary);
        ctx.draw_rect_outline(bounds, theme.colors.border, 1);

        ctx.draw_text(bounds.x + 10, bounds.y + 10, "Decision Details", &theme.fonts.heading, theme.colors.foreground);

        // Draw reasoning
        ctx.draw_text(bounds.x + 10, bounds.y + 50, "Reasoning:", &theme.fonts.default, theme.colors.foreground);

        for (i, reason) in decision.reasoning.iter().enumerate() {
            ctx.draw_text(
                bounds.x + 20,
                bounds.y + 75 + (i as u32 * 25),
                &format!("• {}", reason),
                &theme.fonts.default,
                theme.colors.foreground,
            );
        }
    }
}
```

### 3. System Monitor

```rust
// crates/desktop/src/apps/system_monitor.rs

pub struct SystemMonitorApp {
    window_id: WindowId,
    cpu_history: VecDeque<f32>,    // Last 60 samples
    mem_history: VecDeque<f32>,
    process_list: Vec<ProcessInfo>,
    sort_by: SortColumn,
}

pub struct ProcessInfo {
    pid: u32,
    name: String,
    cpu_percent: f32,
    mem_mb: u32,
    state: ProcessState,
}

impl SystemMonitorApp {
    pub fn update(&mut self) {
        // Query kernel for metrics
        let metrics = kernel_get_system_metrics();

        self.cpu_history.push_back(metrics.cpu_usage);
        if self.cpu_history.len() > 60 {
            self.cpu_history.pop_front();
        }

        self.mem_history.push_back(metrics.memory_usage_percent);
        if self.mem_history.len() > 60 {
            self.mem_history.pop_front();
        }

        // Update process list
        self.process_list = kernel_get_process_list();
        self.sort_processes();
    }

    pub fn draw(&self, ctx: &mut DrawContext, bounds: Rect) {
        let theme = get_current_theme();

        // Draw CPU graph (top left)
        self.draw_graph(
            ctx,
            Rect { x: bounds.x + 20, y: bounds.y + 20, width: 400, height: 150 },
            "CPU Usage",
            &self.cpu_history,
            theme.colors.primary,
        );

        // Draw Memory graph (top right)
        self.draw_graph(
            ctx,
            Rect { x: bounds.x + 440, y: bounds.y + 20, width: 400, height: 150 },
            "Memory Usage",
            &self.mem_history,
            theme.colors.accent,
        );

        // Draw process table (bottom)
        self.draw_process_table(
            ctx,
            Rect { x: bounds.x + 20, y: bounds.y + 190, width: bounds.width - 40, height: bounds.height - 210 },
        );
    }

    fn draw_graph(&self, ctx: &mut DrawContext, bounds: Rect, title: &str, data: &VecDeque<f32>, color: Color) {
        let theme = get_current_theme();

        // Draw background
        ctx.fill_rect(bounds, theme.colors.secondary);
        ctx.draw_rect_outline(bounds, theme.colors.border, 1);

        // Draw title
        ctx.draw_text(bounds.x + 10, bounds.y + 10, title, &theme.fonts.heading, theme.colors.foreground);

        // Draw graph axes
        let graph_bounds = Rect {
            x: bounds.x + 10,
            y: bounds.y + 40,
            width: bounds.width - 20,
            height: bounds.height - 60,
        };

        // Draw grid lines
        for i in 0..=4 {
            let y = graph_bounds.y + (graph_bounds.height * i / 4);
            ctx.draw_line(
                graph_bounds.x as i32,
                y as i32,
                (graph_bounds.x + graph_bounds.width) as i32,
                y as i32,
                theme.colors.border,
            );

            let label = format!("{}%", 100 - (i * 25));
            ctx.draw_text(graph_bounds.x + 5, y, &label, &theme.fonts.default, theme.colors.foreground);
        }

        // Draw data points
        if data.len() < 2 {
            return;
        }

        let point_spacing = graph_bounds.width / (data.len() as u32 - 1);

        for i in 0..(data.len() - 1) {
            let x1 = graph_bounds.x + (i as u32 * point_spacing);
            let y1 = graph_bounds.y + graph_bounds.height - ((data[i] * graph_bounds.height as f32) as u32);

            let x2 = graph_bounds.x + ((i + 1) as u32 * point_spacing);
            let y2 = graph_bounds.y + graph_bounds.height - ((data[i + 1] * graph_bounds.height as f32) as u32);

            ctx.draw_line(x1 as i32, y1 as i32, x2 as i32, y2 as i32, color);
        }
    }

    fn draw_process_table(&self, ctx: &mut DrawContext, bounds: Rect) {
        let theme = get_current_theme();

        ctx.fill_rect(bounds, theme.colors.background);
        ctx.draw_rect_outline(bounds, theme.colors.border, 1);

        // Draw table header
        let headers = ["PID", "Name", "CPU %", "Memory", "State"];
        let col_widths = [80, 300, 100, 100, 100];

        let mut x = bounds.x + 10;
        for (header, width) in headers.iter().zip(col_widths.iter()) {
            ctx.draw_text(x, bounds.y + 10, header, &theme.fonts.heading, theme.colors.foreground);
            x += width;
        }

        // Draw separator line
        ctx.draw_line(
            bounds.x as i32,
            (bounds.y + 35) as i32,
            (bounds.x + bounds.width) as i32,
            (bounds.y + 35) as i32,
            theme.colors.border,
        );

        // Draw process rows
        let line_height = 25;
        for (i, process) in self.process_list.iter().take(20).enumerate() {
            let y = bounds.y + 45 + (i as u32 * line_height);

            let mut x = bounds.x + 10;

            // PID
            ctx.draw_text(x, y, &process.pid.to_string(), &theme.fonts.default, theme.colors.foreground);
            x += col_widths[0];

            // Name
            ctx.draw_text(x, y, &process.name, &theme.fonts.default, theme.colors.foreground);
            x += col_widths[1];

            // CPU %
            ctx.draw_text(x, y, &format!("{:.1}%", process.cpu_percent), &theme.fonts.default, theme.colors.foreground);
            x += col_widths[2];

            // Memory
            ctx.draw_text(x, y, &format!("{} MB", process.mem_mb), &theme.fonts.default, theme.colors.foreground);
            x += col_widths[3];

            // State
            let state_str = match process.state {
                ProcessState::Running => "Running",
                ProcessState::Sleeping => "Sleeping",
                ProcessState::Stopped => "Stopped",
                _ => "Unknown",
            };
            ctx.draw_text(x, y, state_str, &theme.fonts.default, theme.colors.foreground);
        }
    }
}
```

### Acceptance Tests

```bash
# tests/phase_g3/test_apps.sh

# Test 1: Terminal
test_terminal() {
    # Launch terminal app
    # Type "help" + Enter
    # Check: Command executes, output shown
    # Check: Command history works (up arrow)
}

# Test 2: AI Dashboard
test_ai_dashboard() {
    # Launch dashboard
    # Trigger AI decision (via kernel)
    # Check: Decision appears in log
    # Check: Confidence bar renders
    # Click decision
    # Check: Details panel shows reasoning
}

# Test 3: System Monitor
test_system_monitor() {
    # Launch monitor
    # Wait 10 seconds
    # Check: CPU/memory graphs update
    # Check: Process list populated
    # Click column header
    # Check: Processes re-sort
}
```

### Exit Criteria

- ✅ All 5 core apps functional and stable
- ✅ Apps demonstrate AI features effectively
- ✅ UI responsive and polished
- ✅ No crashes or hangs
- ✅ Apps integrate with kernel APIs correctly

---

## Phase G.4: AI Integration UI (Weeks 12-14)

**Objective**: Make kernel AI visible, understandable, and controllable from the desktop.

### Deliverables

- ✅ AI Status Bar (always visible)
- ✅ Live Explainability Widget
- ✅ AI Control Panel (configure autonomy)
- ✅ Pattern Visualization
- ✅ Decision Timeline
- ✅ "Why?" button for any AI action

### AI Status Bar

```rust
// crates/desktop/src/ai/status_bar.rs

pub struct AIStatusBar {
    bounds: Rect,
    ai_mode: AIMode,
    decision_count_today: u32,
    last_action: Option<String>,
    avg_confidence: f32,
    expanded: bool,
}

pub enum AIMode {
    Disabled,
    Learning,    // Observing but not acting
    Active,      // Autonomous decisions enabled
}

impl AIStatusBar {
    pub fn draw(&self, ctx: &mut DrawContext) {
        let theme = get_current_theme();

        // Draw background
        let bg_color = match self.ai_mode {
            AIMode::Disabled => Color::from_rgb(100, 100, 100),
            AIMode::Learning => Color::from_rgb(255, 185, 0),
            AIMode::Active => Color::from_rgb(16, 124, 16),
        };
        ctx.fill_rect(self.bounds, bg_color);

        // Draw AI icon + status
        let status_text = match self.ai_mode {
            AIMode::Disabled => "AI: OFF",
            AIMode::Learning => "AI: LEARNING",
            AIMode::Active => "AI: ACTIVE",
        };
        ctx.draw_text(self.bounds.x + 10, self.bounds.y + 5, status_text, &theme.fonts.default, Color::WHITE);

        // Draw decision count
        let count_text = format!("Decisions today: {}", self.decision_count_today);
        ctx.draw_text(self.bounds.x + 150, self.bounds.y + 5, &count_text, &theme.fonts.default, Color::WHITE);

        // Draw confidence meter
        let meter_x = self.bounds.x + 350;
        let meter_width = 100;
        ctx.draw_rect_outline(
            Rect { x: meter_x, y: self.bounds.y + 5, width: meter_width, height: 20 },
            Color::WHITE,
            1,
        );

        let fill_width = (meter_width as f32 * self.avg_confidence) as u32;
        ctx.fill_rect(
            Rect { x: meter_x + 1, y: self.bounds.y + 6, width: fill_width, height: 18 },
            Color::WHITE,
        );

        let confidence_text = format!("Confidence: {:.0}%", self.avg_confidence * 100.0);
        ctx.draw_text(meter_x + meter_width + 10, self.bounds.y + 5, &confidence_text, &theme.fonts.default, Color::WHITE);

        // Draw last action (if expanded)
        if self.expanded {
            if let Some(action) = &self.last_action {
                ctx.draw_text(
                    self.bounds.x + 10,
                    self.bounds.y + 30,
                    &format!("Last: {}", action),
                    &theme.fonts.default,
                    Color::WHITE,
                );
            }
        }
    }

    pub fn handle_click(&mut self, x: u32, y: u32) {
        if self.bounds.contains(x, y) {
            self.expanded = !self.expanded;
        }
    }

    pub fn update(&mut self) {
        // Query kernel for latest AI stats
        let stats = kernel_get_ai_stats();
        self.ai_mode = stats.mode;
        self.decision_count_today = stats.decisions_today;
        self.last_action = stats.last_action;
        self.avg_confidence = stats.avg_confidence;
    }
}
```

### Live Explainability Widget

```rust
// crates/desktop/src/ai/explainability.rs

pub struct ExplainabilityWidget {
    current_explanation: Option<AIExplanation>,
    visible: bool,
    position: (u32, u32),
}

pub struct AIExplanation {
    action: String,
    reasoning_steps: Vec<ReasoningStep>,
    confidence: f32,
    data_sources: Vec<String>,
    can_undo: bool,
}

pub struct ReasoningStep {
    description: String,
    evidence: Vec<String>,
    weight: f32,
}

impl ExplainabilityWidget {
    pub fn show_explanation(&mut self, decision_id: u64) {
        // Query kernel for explanation
        self.current_explanation = kernel_explain_decision(decision_id);
        self.visible = true;
    }

    pub fn draw(&self, ctx: &mut DrawContext) {
        if !self.visible {
            return;
        }

        let Some(exp) = &self.current_explanation else {
            return;
        };

        let theme = get_current_theme();

        let bounds = Rect {
            x: self.position.0,
            y: self.position.1,
            width: 400,
            height: 500,
        };

        // Draw modal background
        ctx.fill_rect(bounds, theme.colors.background);
        ctx.draw_rect_outline(bounds, theme.colors.primary, 2);

        // Draw shadow
        ctx.fill_rect(
            Rect {
                x: bounds.x + 5,
                y: bounds.y + 5,
                width: bounds.width,
                height: bounds.height,
            },
            theme.colors.shadow,
        );

        // Draw header
        ctx.fill_rect(
            Rect { x: bounds.x, y: bounds.y, width: bounds.width, height: 40 },
            theme.colors.primary,
        );
        ctx.draw_text(
            bounds.x + 15,
            bounds.y + 10,
            "WHY DID THE AI DO THAT?",
            &theme.fonts.heading,
            Color::WHITE,
        );

        // Draw action
        ctx.draw_text(
            bounds.x + 15,
            bounds.y + 60,
            &format!("Action: {}", exp.action),
            &theme.fonts.heading,
            theme.colors.foreground,
        );

        // Draw reasoning steps
        ctx.draw_text(
            bounds.x + 15,
            bounds.y + 100,
            "Reasoning:",
            &theme.fonts.default,
            theme.colors.foreground,
        );

        let mut y = bounds.y + 130;
        for (i, step) in exp.reasoning_steps.iter().enumerate() {
            ctx.draw_text(
                bounds.x + 25,
                y,
                &format!("{}. {}", i + 1, step.description),
                &theme.fonts.default,
                theme.colors.foreground,
            );
            y += 30;

            // Draw evidence
            for evidence in &step.evidence {
                ctx.draw_text(
                    bounds.x + 40,
                    y,
                    &format!("• {}", evidence),
                    &theme.fonts.default,
                    theme.colors.secondary,
                );
                y += 25;
            }
        }

        // Draw confidence
        ctx.draw_text(
            bounds.x + 15,
            bounds.y + bounds.height - 80,
            &format!("Confidence: {:.0}%", exp.confidence * 100.0),
            &theme.fonts.default,
            theme.colors.foreground,
        );

        // Draw buttons
        let button_y = bounds.y + bounds.height - 50;

        // OK button
        ctx.fill_rect(
            Rect { x: bounds.x + 15, y: button_y, width: 80, height: 35 },
            theme.colors.primary,
        );
        ctx.draw_text(bounds.x + 35, button_y + 10, "OK", &theme.fonts.default, Color::WHITE);

        // Undo button (if applicable)
        if exp.can_undo {
            ctx.fill_rect(
                Rect { x: bounds.x + 110, y: button_y, width: 80, height: 35 },
                theme.colors.warning,
            );
            ctx.draw_text(bounds.x + 125, button_y + 10, "UNDO", &theme.fonts.default, Color::WHITE);
        }

        // Teach AI button
        ctx.fill_rect(
            Rect { x: bounds.x + 205, y: button_y, width: 100, height: 35 },
            theme.colors.accent,
        );
        ctx.draw_text(bounds.x + 215, button_y + 10, "TEACH AI", &theme.fonts.default, Color::WHITE);
    }
}
```

### AI Control Panel

```rust
// crates/desktop/src/apps/ai_control.rs

pub struct AIControlPanel {
    window_id: WindowId,
    autonomy_level: u8,     // 0-100
    learning_enabled: bool,
    prediction_enabled: bool,
    auto_optimize: bool,
    min_confidence: u8,     // 0-100
}

impl AIControlPanel {
    pub fn draw(&self, ctx: &mut DrawContext, bounds: Rect) {
        let theme = get_current_theme();

        ctx.draw_text(bounds.x + 20, bounds.y + 20, "AI Configuration", &theme.fonts.heading, theme.colors.foreground);

        // Autonomy Level Slider
        ctx.draw_text(bounds.x + 20, bounds.y + 70, "Autonomy Level:", &theme.fonts.default, theme.colors.foreground);
        self.draw_slider(
            ctx,
            Rect { x: bounds.x + 200, y: bounds.y + 65, width: 300, height: 30 },
            self.autonomy_level,
        );
        ctx.draw_text(
            bounds.x + 520,
            bounds.y + 70,
            &format!("{}%", self.autonomy_level),
            &theme.fonts.default,
            theme.colors.foreground,
        );

        // Description based on level
        let description = match self.autonomy_level {
            0..=25 => "Low: AI observes but rarely acts",
            26..=50 => "Medium: AI makes suggestions, waits for approval",
            51..=75 => "High: AI acts autonomously, can be overridden",
            76..=100 => "Maximum: AI has full control, explains decisions",
            _ => unreachable!(),
        };
        ctx.draw_text(bounds.x + 200, bounds.y + 100, description, &theme.fonts.default, theme.colors.secondary);

        // Checkboxes
        let checkbox_y = bounds.y + 150;
        self.draw_checkbox(ctx, bounds.x + 20, checkbox_y, self.learning_enabled);
        ctx.draw_text(bounds.x + 50, checkbox_y, "Enable Pattern Learning", &theme.fonts.default, theme.colors.foreground);

        self.draw_checkbox(ctx, bounds.x + 20, checkbox_y + 40, self.prediction_enabled);
        ctx.draw_text(bounds.x + 50, checkbox_y + 40, "Enable Predictive Actions", &theme.fonts.default, theme.colors.foreground);

        self.draw_checkbox(ctx, bounds.x + 20, checkbox_y + 80, self.auto_optimize);
        ctx.draw_text(bounds.x + 50, checkbox_y + 80, "Auto-optimize Resources", &theme.fonts.default, theme.colors.foreground);

        // Minimum Confidence Threshold
        ctx.draw_text(bounds.x + 20, checkbox_y + 140, "Min Confidence Threshold:", &theme.fonts.default, theme.colors.foreground);
        self.draw_slider(
            ctx,
            Rect { x: bounds.x + 250, y: checkbox_y + 135, width: 250, height: 30 },
            self.min_confidence,
        );
        ctx.draw_text(
            bounds.x + 520,
            checkbox_y + 140,
            &format!("{}%", self.min_confidence),
            &theme.fonts.default,
            theme.colors.foreground,
        );

        // Apply button
        ctx.fill_rect(
            Rect { x: bounds.x + 20, y: bounds.y + bounds.height - 60, width: 120, height: 40 },
            theme.colors.primary,
        );
        ctx.draw_text(
            bounds.x + 45,
            bounds.y + bounds.height - 48,
            "APPLY",
            &theme.fonts.heading,
            Color::WHITE,
        );

        // Reset button
        ctx.fill_rect(
            Rect { x: bounds.x + 160, y: bounds.y + bounds.height - 60, width: 120, height: 40 },
            theme.colors.secondary,
        );
        ctx.draw_text(
            bounds.x + 180,
            bounds.y + bounds.height - 48,
            "RESET",
            &theme.fonts.heading,
            Color::WHITE,
        );
    }

    fn draw_slider(&self, ctx: &mut DrawContext, bounds: Rect, value: u8) {
        let theme = get_current_theme();

        // Draw track
        ctx.fill_rect(
            Rect { x: bounds.x, y: bounds.y + bounds.height / 2 - 2, width: bounds.width, height: 4 },
            theme.colors.border,
        );

        // Draw filled portion
        let fill_width = (bounds.width as f32 * (value as f32 / 100.0)) as u32;
        ctx.fill_rect(
            Rect { x: bounds.x, y: bounds.y + bounds.height / 2 - 2, width: fill_width, height: 4 },
            theme.colors.primary,
        );

        // Draw thumb
        let thumb_x = bounds.x + fill_width - 10;
        ctx.fill_circle(thumb_x as i32 + 10, (bounds.y + bounds.height / 2) as i32, 10, theme.colors.primary);
    }

    fn draw_checkbox(&self, ctx: &mut DrawContext, x: u32, y: u32, checked: bool) {
        let theme = get_current_theme();

        ctx.draw_rect_outline(Rect { x, y, width: 20, height: 20 }, theme.colors.border, 2);

        if checked {
            ctx.draw_text(x + 4, y + 2, "✓", &theme.fonts.heading, theme.colors.primary);
        }
    }

    pub fn apply_settings(&self) {
        // Send settings to kernel
        kernel_set_ai_config(AIConfig {
            autonomy_level: self.autonomy_level,
            learning_enabled: self.learning_enabled,
            prediction_enabled: self.prediction_enabled,
            auto_optimize: self.auto_optimize,
            min_confidence: self.min_confidence as f32 / 100.0,
        });
    }
}
```

### Acceptance Tests

```bash
# tests/phase_g4/test_ai_integration.sh

# Test 1: Status bar
test_status_bar() {
    # Launch desktop
    # Check: AI status bar visible at top
    # Trigger AI decision
    # Check: Status bar updates (decision count increments)
}

# Test 2: Explainability
test_explainability() {
    # Trigger AI decision
    # Click "Why?" button
    # Check: Explanation modal appears
    # Check: Reasoning steps shown
    # Check: Confidence displayed
}

# Test 3: Control panel
test_control_panel() {
    # Open AI control panel
    # Adjust autonomy slider to 75%
    # Click Apply
    # Check: Kernel AI config updated
    # Trigger decision
    # Check: AI behaves according to new settings
}

# Test 4: Pattern visualization
test_pattern_viz() {
    # Open AI dashboard
    # Check: Pattern graph renders
    # Check: Shows learned behaviors
}
```

### Exit Criteria

- ✅ AI status always visible
- ✅ Any AI decision can be explained with "Why?" button
- ✅ AI configuration can be changed via UI
- ✅ Explanations are clear and accurate
- ✅ Users feel in control of AI behavior

---

## Phase G.5: Voice/Vision Infrastructure (Weeks 15-16)

**Objective**: Prepare desktop for JARVIS voice assistant and computer vision integration.

### Deliverables

- ✅ Audio input pipeline (microphone)
- ✅ Audio output pipeline (speakers)
- ✅ Voice activity detection (VAD)
- ✅ Camera capture framework
- ✅ Voice UI widget (listen indicator, waveform)
- ✅ Integration points for Whisper + TTS

### Audio Infrastructure

```rust
// crates/desktop/src/audio/mod.rs

pub struct AudioManager {
    input_device: Option<Arc<Mutex<AudioInputDevice>>>,
    output_device: Option<Arc<Mutex<AudioOutputDevice>>>,
    sample_rate: u32,        // 16kHz for voice
    channels: u8,            // Mono for voice
    buffer_size: usize,      // 4096 samples
}

pub struct AudioInputDevice {
    device_id: u32,
    buffer: RingBuffer<i16, 16384>,
    vad_active: bool,
}

pub struct AudioOutputDevice {
    device_id: u32,
    buffer: RingBuffer<i16, 16384>,
}

impl AudioManager {
    pub fn init() -> Result<Self, AudioError> {
        // Detect USB audio device (microphone)
        let input_device = detect_usb_audio_input()?;

        // Detect audio output (virtio-snd or USB)
        let output_device = detect_audio_output()?;

        Ok(Self {
            input_device: Some(Arc::new(Mutex::new(input_device))),
            output_device: Some(Arc::new(Mutex::new(output_device))),
            sample_rate: 16000,  // 16kHz
            channels: 1,         // Mono
            buffer_size: 4096,
        })
    }

    pub fn start_recording(&mut self, callback: impl Fn(&[i16]) + 'static) {
        // Start capturing audio from microphone
        // Call callback with audio chunks
    }

    pub fn stop_recording(&mut self) {
        // Stop capture
    }

    pub fn play_audio(&mut self, samples: &[i16]) {
        // Send audio to output device
    }
}
```

### Voice Activity Detection

```rust
// crates/desktop/src/audio/vad.rs

pub struct VAD {
    energy_threshold: f32,
    zero_crossing_threshold: f32,
    silence_duration_ms: u32,
    voice_detected: bool,
}

impl VAD {
    pub fn new() -> Self {
        Self {
            energy_threshold: 0.01,
            zero_crossing_threshold: 0.3,
            silence_duration_ms: 500,
            voice_detected: false,
        }
    }

    pub fn process_frame(&mut self, samples: &[i16]) -> VADResult {
        let energy = self.calculate_energy(samples);
        let zcr = self.calculate_zero_crossing_rate(samples);

        if energy > self.energy_threshold && zcr < self.zero_crossing_threshold {
            self.voice_detected = true;
            VADResult::VoiceActive
        } else {
            if self.voice_detected {
                // Silence detected, but wait for silence_duration_ms before marking as ended
                VADResult::Silence
            } else {
                VADResult::NoVoice
            }
        }
    }

    fn calculate_energy(&self, samples: &[i16]) -> f32 {
        let sum_squares: f32 = samples.iter()
            .map(|&s| (s as f32 / 32768.0).powi(2))
            .sum();
        sum_squares / samples.len() as f32
    }

    fn calculate_zero_crossing_rate(&self, samples: &[i16]) -> f32 {
        let mut crossings = 0;
        for i in 1..samples.len() {
            if (samples[i] >= 0 && samples[i-1] < 0) || (samples[i] < 0 && samples[i-1] >= 0) {
                crossings += 1;
            }
        }
        crossings as f32 / samples.len() as f32
    }
}

pub enum VADResult {
    VoiceActive,
    Silence,
    NoVoice,
}
```

### Voice UI Widget

```rust
// crates/desktop/src/voice/ui.rs

pub struct VoiceUIWidget {
    bounds: Rect,
    listening: bool,
    audio_samples: Vec<f32>,  // Last 100 samples for waveform
    transcript: Option<String>,
    response: Option<String>,
}

impl VoiceUIWidget {
    pub fn draw(&self, ctx: &mut DrawContext) {
        let theme = get_current_theme();

        // Draw background
        ctx.fill_rect(self.bounds, theme.colors.background);
        ctx.draw_rect_outline(self.bounds, theme.colors.border, 2);

        if self.listening {
            // Draw "Listening..." indicator
            ctx.draw_text(
                self.bounds.x + 20,
                self.bounds.y + 20,
                "🎤 Listening...",
                &theme.fonts.heading,
                theme.colors.primary,
            );

            // Draw waveform
            self.draw_waveform(ctx, Rect {
                x: self.bounds.x + 20,
                y: self.bounds.y + 60,
                width: self.bounds.width - 40,
                height: 80,
            });
        } else {
            ctx.draw_text(
                self.bounds.x + 20,
                self.bounds.y + 20,
                "Say 'Hey JARVIS' to activate",
                &theme.fonts.default,
                theme.colors.foreground,
            );
        }

        // Draw transcript (what user said)
        if let Some(transcript) = &self.transcript {
            ctx.draw_text(
                self.bounds.x + 20,
                self.bounds.y + 160,
                &format!("You: {}", transcript),
                &theme.fonts.default,
                theme.colors.foreground,
            );
        }

        // Draw response (what JARVIS said)
        if let Some(response) = &self.response {
            ctx.draw_text(
                self.bounds.x + 20,
                self.bounds.y + 200,
                &format!("JARVIS: {}", response),
                &theme.fonts.default,
                theme.colors.accent,
            );
        }
    }

    fn draw_waveform(&self, ctx: &mut DrawContext, bounds: Rect) {
        let theme = get_current_theme();

        // Draw center line
        let center_y = bounds.y + bounds.height / 2;
        ctx.draw_line(
            bounds.x as i32,
            center_y as i32,
            (bounds.x + bounds.width) as i32,
            center_y as i32,
            theme.colors.border,
        );

        // Draw waveform
        if self.audio_samples.is_empty() {
            return;
        }

        let sample_spacing = bounds.width / self.audio_samples.len() as u32;

        for (i, &sample) in self.audio_samples.iter().enumerate() {
            let x = bounds.x + (i as u32 * sample_spacing);
            let height = (sample * bounds.height as f32 / 2.0) as i32;
            let y = center_y as i32;

            ctx.draw_line(x as i32, y, x as i32, y + height, theme.colors.primary);
        }
    }
}
```

### Camera Integration

```rust
// crates/desktop/src/vision/camera.rs

pub struct CameraManager {
    device: Option<Arc<Mutex<USBCamera>>>,
    resolution: (u32, u32),     // 640x480
    fps: u32,                   // 30 FPS
    frame_buffer: Vec<u8>,      // Raw RGB data
}

pub struct USBCamera {
    device_path: String,
    fd: i32,
}

impl CameraManager {
    pub fn init() -> Result<Self, CameraError> {
        // Detect USB camera (via Video4Linux2 or similar)
        let device = detect_usb_camera()?;

        Ok(Self {
            device: Some(Arc::new(Mutex::new(device))),
            resolution: (640, 480),
            fps: 30,
            frame_buffer: vec![0; 640 * 480 * 3],
        })
    }

    pub fn start_capture(&mut self, callback: impl Fn(&[u8], u32, u32) + 'static) {
        // Start capturing frames
        // Call callback with RGB data
    }

    pub fn stop_capture(&mut self) {
        // Stop capture
    }
}
```

### Integration Stubs for Future JARVIS

```rust
// crates/desktop/src/jarvis/mod.rs

pub struct JARVISIntegration {
    audio_manager: AudioManager,
    camera_manager: CameraManager,
    vad: VAD,
    wake_word: &'static str,  // "Hey JARVIS"
    listening: bool,
}

impl JARVISIntegration {
    pub fn init() -> Result<Self, JARVISError> {
        Ok(Self {
            audio_manager: AudioManager::init()?,
            camera_manager: CameraManager::init()?,
            vad: VAD::new(),
            wake_word: "hey jarvis",
            listening: false,
        })
    }

    pub fn start_listening(&mut self) {
        self.audio_manager.start_recording(|samples| {
            // TODO: Integrate Whisper for speech-to-text
            // For now, just detect wake word via VAD

            let vad_result = self.vad.process_frame(samples);

            match vad_result {
                VADResult::VoiceActive => {
                    self.listening = true;
                }
                VADResult::Silence => {
                    if self.listening {
                        // Voice ended, process command
                        self.process_voice_command();
                    }
                }
                VADResult::NoVoice => {}
            }
        });
    }

    fn process_voice_command(&mut self) {
        // TODO: Phase H (post-G) will integrate:
        // 1. Whisper for speech-to-text
        // 2. Kernel LLM or Cloud API for intent understanding
        // 3. Execute command
        // 4. TTS for response

        // For now, just placeholder
        println!("[JARVIS] Voice command detected (integration pending)");
    }

    pub fn take_photo(&mut self) -> Option<Vec<u8>> {
        // Capture single frame from camera
        // TODO: Phase H will add computer vision (object detection, etc.)
        None
    }
}
```

### Acceptance Tests

```bash
# tests/phase_g5/test_voice_vision.sh

# Test 1: Audio input
test_audio_input() {
    # Plug in USB microphone
    # Start recording
    # Check: Audio samples captured
    # Check: VAD detects voice vs. silence
}

# Test 2: Audio output
test_audio_output() {
    # Play test tone
    # Check: Sound plays through speakers
    # Check: No distortion or clipping
}

# Test 3: Voice UI
test_voice_ui() {
    # Launch voice UI widget
    # Start recording
    # Check: Waveform visualizes audio
    # Check: "Listening..." indicator shows
}

# Test 4: Camera
test_camera() {
    # Plug in USB camera
    # Start capture
    # Check: Frames captured at 30 FPS
    # Display frame on screen
    # Check: Image visible
}
```

### Exit Criteria

- ✅ USB microphone detected and captures audio
- ✅ VAD correctly detects voice activity
- ✅ Audio output works (can play sounds)
- ✅ Voice UI widget visualizes microphone input
- ✅ USB camera captures frames
- ✅ Infrastructure ready for Whisper + TTS integration

---

## Phase G.6: Polish & Animations (Weeks 17-18)

**Objective**: Add professional polish, animations, and final touches.

### Deliverables

- ✅ Window fade in/out animations
- ✅ Smooth transitions between apps
- ✅ Loading indicators
- ✅ Icon design
- ✅ Dark/light theme toggle
- ✅ Keyboard navigation polish
- ✅ Performance optimization (60 FPS)

### Animation System

```rust
// crates/desktop/src/animation/mod.rs

pub struct Animator {
    animations: Vec<Animation>,
}

pub struct Animation {
    id: u64,
    target: AnimationTarget,
    property: AnimationProperty,
    start_value: f32,
    end_value: f32,
    duration_ms: u32,
    elapsed_ms: u32,
    easing: EasingFunction,
    on_complete: Option<Box<dyn Fn()>>,
}

pub enum AnimationTarget {
    Window(WindowId),
    Widget(WidgetId),
    Global,
}

pub enum AnimationProperty {
    Opacity,
    X,
    Y,
    Width,
    Height,
    Scale,
    Rotation,
}

pub enum EasingFunction {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Bounce,
}

impl Animator {
    pub fn animate(
        &mut self,
        target: AnimationTarget,
        property: AnimationProperty,
        from: f32,
        to: f32,
        duration_ms: u32,
        easing: EasingFunction,
    ) -> u64 {
        let id = self.next_animation_id();

        self.animations.push(Animation {
            id,
            target,
            property,
            start_value: from,
            end_value: to,
            duration_ms,
            elapsed_ms: 0,
            easing,
            on_complete: None,
        });

        id
    }

    pub fn update(&mut self, delta_ms: u32) {
        let mut completed = Vec::new();

        for animation in &mut self.animations {
            animation.elapsed_ms += delta_ms;

            if animation.elapsed_ms >= animation.duration_ms {
                // Animation complete
                animation.elapsed_ms = animation.duration_ms;
                completed.push(animation.id);

                if let Some(callback) = &animation.on_complete {
                    callback();
                }
            }
        }

        // Remove completed animations
        self.animations.retain(|a| !completed.contains(&a.id));
    }

    pub fn get_value(&self, target: &AnimationTarget, property: &AnimationProperty) -> Option<f32> {
        for animation in &self.animations {
            if animation.target == *target && animation.property == *property {
                let progress = animation.elapsed_ms as f32 / animation.duration_ms as f32;
                let eased = apply_easing(progress, &animation.easing);
                return Some(lerp(animation.start_value, animation.end_value, eased));
            }
        }

        None
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn apply_easing(t: f32, easing: &EasingFunction) -> f32 {
    match easing {
        EasingFunction::Linear => t,
        EasingFunction::EaseIn => t * t,
        EasingFunction::EaseOut => t * (2.0 - t),
        EasingFunction::EaseInOut => {
            if t < 0.5 {
                2.0 * t * t
            } else {
                -1.0 + (4.0 - 2.0 * t) * t
            }
        }
        EasingFunction::Bounce => {
            // Simplified bounce
            if t < 0.5 {
                8.0 * t * t
            } else {
                1.0 - 8.0 * (1.0 - t) * (1.0 - t)
            }
        }
    }
}
```

### Window Animations

```rust
// Add to WindowManager
impl WindowManager {
    pub fn create_window_animated(&mut self, spec: WindowSpec) -> WindowId {
        let id = self.create_window(spec);

        // Animate window fade-in
        GLOBAL_ANIMATOR.lock().animate(
            AnimationTarget::Window(id),
            AnimationProperty::Opacity,
            0.0,    // Start transparent
            1.0,    // End opaque
            300,    // 300ms duration
            EasingFunction::EaseOut,
        );

        id
    }

    pub fn close_window_animated(&mut self, id: WindowId) {
        // Animate window fade-out
        let animation_id = GLOBAL_ANIMATOR.lock().animate(
            AnimationTarget::Window(id),
            AnimationProperty::Opacity,
            1.0,
            0.0,
            200,
            EasingFunction::EaseIn,
        );

        // Set callback to actually destroy window after animation
        GLOBAL_ANIMATOR.lock().set_on_complete(animation_id, move || {
            // Destroy window
            self.destroy_window(id);
        });
    }
}
```

### Loading Indicators

```rust
// crates/desktop/src/ui/widgets/spinner.rs

pub struct Spinner {
    angle: f32,
    speed: f32,  // Degrees per frame
    radius: u32,
    thickness: u32,
    color: Color,
}

impl Spinner {
    pub fn update(&mut self) {
        self.angle += self.speed;
        if self.angle >= 360.0 {
            self.angle -= 360.0;
        }
    }

    pub fn draw(&self, ctx: &mut DrawContext, center: (u32, u32)) {
        // Draw circular spinner
        let segments = 12;
        let arc_length = 270.0;  // 3/4 of circle

        for i in 0..segments {
            let segment_angle = self.angle + (i as f32 * (arc_length / segments as f32));
            let alpha = (i as f32 / segments as f32 * 255.0) as u8;

            let start_angle = segment_angle.to_radians();
            let end_angle = (segment_angle + (arc_length / segments as f32)).to_radians();

            let x1 = center.0 as i32 + (self.radius as f32 * start_angle.cos()) as i32;
            let y1 = center.1 as i32 + (self.radius as f32 * start_angle.sin()) as i32;

            let x2 = center.0 as i32 + (self.radius as f32 * end_angle.cos()) as i32;
            let y2 = center.1 as i32 + (self.radius as f32 * end_angle.sin()) as i32;

            ctx.draw_line(x1, y1, x2, y2, self.color.with_alpha(alpha));
        }
    }
}
```

### Icon Design

```rust
// crates/desktop/src/ui/icons.rs

pub struct Icon {
    name: String,
    size: u32,
    pixels: Vec<u8>,  // RGBA
}

// Pre-rendered icons (48x48, 32x32, 24x24 variants)
pub fn load_system_icons() -> HashMap<String, Icon> {
    let mut icons = HashMap::new();

    icons.insert("terminal".to_string(), render_terminal_icon());
    icons.insert("settings".to_string(), render_settings_icon());
    icons.insert("files".to_string(), render_files_icon());
    icons.insert("ai-brain".to_string(), render_ai_icon());
    icons.insert("close".to_string(), render_close_icon());
    icons.insert("minimize".to_string(), render_minimize_icon());
    icons.insert("maximize".to_string(), render_maximize_icon());

    icons
}

fn render_terminal_icon() -> Icon {
    // Draw terminal icon ($ prompt in window)
    let mut pixels = vec![0u8; 48 * 48 * 4];

    // Window background
    fill_rect(&mut pixels, 48, Rect { x: 4, y: 4, width: 40, height: 40 }, Color::BLACK);

    // $ prompt
    draw_text(&mut pixels, 48, 12, 20, "$", Color::GREEN);
    draw_text(&mut pixels, 48, 20, 20, "_", Color::GREEN);

    Icon {
        name: "terminal".to_string(),
        size: 48,
        pixels,
    }
}

// Similar functions for other icons...
```

### Performance Optimization

```rust
// crates/desktop/src/performance.rs

pub struct PerformanceMonitor {
    frame_times: VecDeque<u64>,  // Last 60 frame times (µs)
    draw_calls: u64,
    pixels_drawn: u64,
    target_fps: u32,
}

impl PerformanceMonitor {
    pub fn begin_frame(&mut self) {
        self.frame_start = read_timestamp_us();
        self.draw_calls = 0;
        self.pixels_drawn = 0;
    }

    pub fn end_frame(&mut self) {
        let frame_time = read_timestamp_us() - self.frame_start;
        self.frame_times.push_back(frame_time);

        if self.frame_times.len() > 60 {
            self.frame_times.pop_front();
        }
    }

    pub fn get_fps(&self) -> f32 {
        if self.frame_times.is_empty() {
            return 0.0;
        }

        let avg_frame_time = self.frame_times.iter().sum::<u64>() / self.frame_times.len() as u64;
        1_000_000.0 / avg_frame_time as f32
    }

    pub fn should_skip_frame(&self) -> bool {
        // If we're behind target FPS, skip non-critical rendering
        self.get_fps() < (self.target_fps as f32 * 0.9)
    }
}

// Dirty region optimization
pub struct DirtyRegionTracker {
    dirty_rects: Vec<Rect>,
    full_redraw: bool,
}

impl DirtyRegionTracker {
    pub fn mark_dirty(&mut self, rect: Rect) {
        // Merge with existing dirty rects if overlapping
        self.dirty_rects.push(rect);
    }

    pub fn get_dirty_regions(&self) -> &[Rect] {
        if self.full_redraw {
            // Return entire screen
            &[SCREEN_RECT]
        } else {
            &self.dirty_rects
        }
    }

    pub fn clear(&mut self) {
        self.dirty_rects.clear();
        self.full_redraw = false;
    }
}
```

### Acceptance Tests

```bash
# tests/phase_g6/test_polish.sh

# Test 1: Animations
test_animations() {
    # Create window
    # Check: Window fades in smoothly
    # Close window
    # Check: Window fades out smoothly
}

# Test 2: Performance
test_performance() {
    # Open 5 windows with animations
    # Measure FPS
    # Check: Maintains 60 FPS
}

# Test 3: Icons
test_icons() {
    # Check: All system icons load
    # Check: Icons render correctly at all sizes
}

# Test 4: Theme toggle
test_theme_toggle() {
    # Switch from dark to light theme
    # Check: All colors update
    # Check: Smooth transition
}
```

### Exit Criteria

- ✅ All animations smooth (60 FPS)
- ✅ Window operations feel polished
- ✅ Icons look professional
- ✅ Light/dark themes work correctly
- ✅ No performance regressions
- ✅ Desktop feels production-ready

---

## Visual Design System

### Color Palette

**Dark Theme (Default)**:
```
Background:  #1E1E1E (30, 30, 30)
Foreground:  #FFFFFF (255, 255, 255)
Primary:     #007ACC (0, 122, 204) - Blue
Secondary:   #505050 (80, 80, 80) - Gray
Accent:      #FFB900 (255, 185, 0) - Gold
Error:       #E81123 (232, 17, 35) - Red
Warning:     #FFB900 (255, 185, 0) - Orange
Success:     #107C10 (16, 124, 16) - Green
Border:      #3C3C3C (60, 60, 60)
Shadow:      #00000080 (0, 0, 0, 128)
```

**Light Theme**:
```
Background:  #FFFFFF (255, 255, 255)
Foreground:  #000000 (0, 0, 0)
Primary:     #0078D4 (0, 120, 212) - Blue
Secondary:   #C8C8C8 (200, 200, 200) - Gray
Accent:      #FF8C00 (255, 140, 0) - Orange
Error:       #E81123 (232, 17, 35) - Red
Warning:     #FF8C00 (255, 140, 0) - Orange
Success:     #107C10 (16, 124, 16) - Green
Border:      #CCCCCC (204, 204, 204)
Shadow:      #00000040 (0, 0, 0, 64)
```

### Typography

```
Heading Font: 24px, Bold
Default Font: 14px, Regular
Monospace:    12px, Monospace (for code/terminal)
```

### Spacing

```
XS:  4px
SM:  8px
MD:  16px
LG:  24px
XL:  32px
```

---

## Testing Strategy

### Unit Tests
- Widget rendering
- Event propagation
- Animation calculations
- Theme switching

### Integration Tests
- Window manager operations
- Application lifecycle
- AI integration hooks
- Input handling

### Performance Tests
- FPS benchmarks (target: 60 FPS)
- Memory usage (target: <100MB for desktop)
- Latency measurements

### User Acceptance Tests
- All apps functional
- Keyboard shortcuts work
- Mouse interactions responsive
- Visual polish acceptable

---

## Integration with Existing Systems

### Kernel Integration

```rust
// Desktop calls kernel for AI features
pub fn kernel_get_ai_stats() -> AIStats {
    // Syscall to query kernel autonomous control state
    syscall(SYS_AI_GET_STATS, ...)
}

pub fn kernel_explain_decision(id: u64) -> Option<AIExplanation> {
    // Syscall to get explanation for specific decision
    syscall(SYS_AI_EXPLAIN, id, ...)
}

pub fn kernel_set_ai_config(config: AIConfig) {
    // Syscall to configure AI behavior
    syscall(SYS_AI_SET_CONFIG, &config as *const _ as usize, ...)
}
```

### Web GUI Integration

**Current Web GUI** → **Native Desktop App**

The existing React web GUI (gui/desktop) will be ported to a native desktop application:

```
gui/desktop/ (React, TypeScript, Tauri)
     ↓
crates/desktop/src/apps/terminal.rs (Native Rust)
```

Components to port:
- Terminal emulator → Native terminal app
- System monitor → Native system monitor
- Shell executor → Direct kernel integration

---

## Summary Timeline

| Phase | Duration | Deliverables |
|-------|----------|--------------|
| **G.0** | 2 weeks | virtio-gpu driver, 2D primitives, font rendering |
| **G.1** | 3 weeks | Window manager, tiling/floating, keyboard shortcuts |
| **G.2** | 3 weeks | UI toolkit, widgets, themes |
| **G.3** | 3 weeks | Core apps (Terminal, AI Dashboard, System Monitor, Files, Settings) |
| **G.4** | 3 weeks | AI integration UI (status bar, explainability, control panel) |
| **G.5** | 2 weeks | Voice/vision infrastructure (audio, camera, VAD) |
| **G.6** | 2 weeks | Polish, animations, icons, performance |
| **TOTAL** | **18 weeks** | **Production-ready AI-native desktop environment** |

---

## Success Metrics

- ✅ Desktop boots and displays properly in QEMU
- ✅ All 5 core applications functional
- ✅ AI features visible and interactive
- ✅ Voice/vision infrastructure operational
- ✅ Performance: 60 FPS on Raspberry Pi 5
- ✅ Memory: <150MB RAM usage for desktop
- ✅ User feedback: "This feels professional and unique"

---

## Next Steps After Phase G

1. **JARVIS Integration** (Phase H, 6-8 weeks):
   - Integrate Whisper (speech-to-text)
   - Integrate Coqui TTS (text-to-speech)
   - Local LLM (Llama 3 8B) or Cloud API
   - Computer vision (object detection, face recognition)

2. **Raspberry Pi 5 Port** (8-12 weeks):
   - Boot on real hardware
   - GPIO control via kernel
   - Hardware-specific drivers

3. **Advanced Desktop Features** (ongoing):
   - More applications
   - Better animations
   - Advanced themes
   - Accessibility features

---

**Document Status**: ✅ Complete and Ready for Implementation
**Approval Required**: Review + Start Date Confirmation
**Estimated Start**: After Phase F completion (Now!)
**Estimated Completion**: 18 weeks from start

---

*This blueprint is a living document and will be updated as implementation progresses.*
