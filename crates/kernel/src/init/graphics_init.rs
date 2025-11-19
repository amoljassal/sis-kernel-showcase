//! Phase 7: Graphics and UI initialization
//!
//! This phase initializes graphics and UI subsystems:
//! - virtio-gpu devices
//! - Graphics subsystem
//! - Window manager
//! - UI toolkit
//! - Desktop applications
//!
//! All components are optional - failures are non-fatal

use super::{InitError, InitResult};

/// Initialize graphics and UI subsystems (all optional)
///
/// # Safety
/// Must be called after SMP/security init (Phase 6)
/// Must be called before AI subsystem (Phase 8)
pub unsafe fn init_graphics() -> InitResult<()> {
    // Initialize virtio-gpu devices
    init_virtio_gpu()?;

    // Initialize graphics subsystem (optional)
    if let Ok(()) = init_graphics_subsystem() {
        // Initialize window manager (optional)
        if let Ok(()) = init_window_manager() {
            // Initialize UI toolkit (optional)
            if let Ok(()) = init_ui_toolkit() {
                // Test and launch applications (optional)
                let _ = test_and_launch_applications();
            }
        }
    }

    Ok(())
}

/// Initialize virtio-gpu devices
unsafe fn init_virtio_gpu() -> InitResult<()> {
    #[cfg(target_arch = "aarch64")]
    crate::arch::aarch64::init_virtio_gpu();
    Ok(())
}

/// Initialize graphics subsystem (optional)
unsafe fn init_graphics_subsystem() -> InitResult<()> {
    crate::graphics::init()
        .map_err(|_| InitError::GraphicsFailed)?;

    // Run graphics test
    let _ = crate::graphics::test_graphics();

    Ok(())
}

/// Initialize window manager (optional)
unsafe fn init_window_manager() -> InitResult<()> {
    crate::window_manager::init()
        .map_err(|_| InitError::WindowManagerFailed)?;

    // Run window manager test
    let _ = crate::window_manager::test_window_manager();

    Ok(())
}

/// Initialize UI toolkit (optional)
unsafe fn init_ui_toolkit() -> InitResult<()> {
    crate::ui::init()
        .map_err(|_| InitError::WindowManagerFailed)?;

    // Run UI toolkit test
    let _ = crate::ui::test_ui_toolkit();

    Ok(())
}

/// Test and launch desktop applications (optional)
unsafe fn test_and_launch_applications() -> InitResult<()> {
    // Test applications
    let _ = crate::applications::test_applications();

    // Launch all applications in windows
    let _ = crate::applications::launch_all_apps();

    Ok(())
}
