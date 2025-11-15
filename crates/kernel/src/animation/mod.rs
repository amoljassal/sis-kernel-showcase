/// Animation System - Phase G.6
///
/// Professional animations and transitions for desktop environment

pub mod easing;
pub mod animator;
pub mod transitions;

pub use easing::{EasingFunction, apply_easing, lerp};
pub use animator::{Animator, Animation, AnimationTarget, AnimationProperty, AnimationId};
pub use transitions::{TransitionType, Transition};

use spin::Mutex;

/// Global animator instance
static GLOBAL_ANIMATOR: Mutex<Option<Animator>> = Mutex::new(None);

/// Initialize the animation system
pub fn init() {
    *GLOBAL_ANIMATOR.lock() = Some(Animator::new());
    crate::info!("animation: animation system initialized");
}

/// Get global animator
pub fn get_animator() -> &'static Mutex<Option<Animator>> {
    &GLOBAL_ANIMATOR
}

/// Start an animation
pub fn animate(
    target: AnimationTarget,
    property: AnimationProperty,
    from: f32,
    to: f32,
    duration_ms: u32,
    easing: EasingFunction,
) -> Option<AnimationId> {
    if let Some(ref mut animator) = *GLOBAL_ANIMATOR.lock() {
        Some(animator.animate(target, property, from, to, duration_ms, easing))
    } else {
        None
    }
}

/// Update all animations (call this from main loop)
pub fn update(delta_ms: u32) {
    if let Some(ref mut animator) = *GLOBAL_ANIMATOR.lock() {
        animator.update(delta_ms);
    }
}

/// Get current animated value
pub fn get_value(target: &AnimationTarget, property: &AnimationProperty) -> Option<f32> {
    if let Some(ref animator) = *GLOBAL_ANIMATOR.lock() {
        animator.get_value(target, property)
    } else {
        None
    }
}

/// Stop an animation
pub fn stop(animation_id: AnimationId) {
    if let Some(ref mut animator) = *GLOBAL_ANIMATOR.lock() {
        animator.stop(animation_id);
    }
}

/// Stop all animations for a target
pub fn stop_all_for_target(target: &AnimationTarget) {
    if let Some(ref mut animator) = *GLOBAL_ANIMATOR.lock() {
        animator.stop_all_for_target(target);
    }
}
