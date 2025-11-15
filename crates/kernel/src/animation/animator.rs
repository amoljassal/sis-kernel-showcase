/// Animator - Phase G.6
///
/// Animation manager for controlling multiple simultaneous animations

use super::easing::{EasingFunction, apply_easing, lerp};
use alloc::vec::Vec;
use alloc::boxed::Box;

/// Animation ID (unique identifier)
pub type AnimationId = u64;

/// Animation target
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationTarget {
    /// Window with specific ID
    Window(u32),
    /// Widget with specific ID
    Widget(u32),
    /// Global screen effect
    Global,
}

/// Animation property
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationProperty {
    /// Opacity (0.0 - 1.0)
    Opacity,
    /// X position
    X,
    /// Y position
    Y,
    /// Width
    Width,
    /// Height
    Height,
    /// Scale (0.0 - 1.0+)
    Scale,
    /// Rotation (degrees)
    Rotation,
    /// Custom property
    Custom(u32),
}

/// Animation
pub struct Animation {
    pub id: AnimationId,
    pub target: AnimationTarget,
    pub property: AnimationProperty,
    pub start_value: f32,
    pub end_value: f32,
    pub duration_ms: u32,
    pub elapsed_ms: u32,
    pub easing: EasingFunction,
    pub on_complete: Option<Box<dyn Fn() + Send + Sync>>,
    pub repeat: bool,
    pub reverse_on_repeat: bool,
}

impl Animation {
    /// Get current animated value
    pub fn current_value(&self) -> f32 {
        let progress = (self.elapsed_ms as f32 / self.duration_ms as f32).min(1.0);
        let eased = apply_easing(progress, &self.easing);
        lerp(self.start_value, self.end_value, eased)
    }

    /// Check if animation is complete
    pub fn is_complete(&self) -> bool {
        !self.repeat && self.elapsed_ms >= self.duration_ms
    }
}

/// Animator (manages all animations)
pub struct Animator {
    animations: Vec<Animation>,
    next_id: AnimationId,
}

impl Animator {
    /// Create a new animator
    pub fn new() -> Self {
        Self {
            animations: Vec::new(),
            next_id: 1,
        }
    }

    /// Start a new animation
    pub fn animate(
        &mut self,
        target: AnimationTarget,
        property: AnimationProperty,
        from: f32,
        to: f32,
        duration_ms: u32,
        easing: EasingFunction,
    ) -> AnimationId {
        let id = self.next_animation_id();

        // Stop any existing animation for this target/property
        self.animations.retain(|a| {
            !(a.target == target && a.property == property)
        });

        let animation = Animation {
            id,
            target,
            property,
            start_value: from,
            end_value: to,
            duration_ms,
            elapsed_ms: 0,
            easing,
            on_complete: None,
            repeat: false,
            reverse_on_repeat: false,
        };

        self.animations.push(animation);
        id
    }

    /// Start an animation with completion callback
    pub fn animate_with_callback<F>(
        &mut self,
        target: AnimationTarget,
        property: AnimationProperty,
        from: f32,
        to: f32,
        duration_ms: u32,
        easing: EasingFunction,
        on_complete: F,
    ) -> AnimationId
    where
        F: Fn() + Send + Sync + 'static,
    {
        let id = self.animate(target, property, from, to, duration_ms, easing);

        // Find the animation we just added and set callback
        if let Some(animation) = self.animations.iter_mut().find(|a| a.id == id) {
            animation.on_complete = Some(Box::new(on_complete));
        }

        id
    }

    /// Start a repeating animation
    pub fn animate_repeat(
        &mut self,
        target: AnimationTarget,
        property: AnimationProperty,
        from: f32,
        to: f32,
        duration_ms: u32,
        easing: EasingFunction,
        reverse: bool,
    ) -> AnimationId {
        let id = self.animate(target, property, from, to, duration_ms, easing);

        if let Some(animation) = self.animations.iter_mut().find(|a| a.id == id) {
            animation.repeat = true;
            animation.reverse_on_repeat = reverse;
        }

        id
    }

    /// Update all animations
    pub fn update(&mut self, delta_ms: u32) {
        let mut completed = Vec::new();

        for animation in &mut self.animations {
            animation.elapsed_ms += delta_ms;

            if animation.elapsed_ms >= animation.duration_ms {
                if animation.repeat {
                    // Reset animation
                    animation.elapsed_ms = 0;

                    if animation.reverse_on_repeat {
                        // Swap start and end for ping-pong effect
                        core::mem::swap(&mut animation.start_value, &mut animation.end_value);
                    }
                } else {
                    // Mark as complete
                    animation.elapsed_ms = animation.duration_ms;
                    completed.push(animation.id);

                    // Call completion callback
                    if let Some(ref callback) = animation.on_complete {
                        callback();
                    }
                }
            }
        }

        // Remove completed animations
        self.animations.retain(|a| !completed.contains(&a.id));
    }

    /// Get current animated value for a target/property
    pub fn get_value(&self, target: &AnimationTarget, property: &AnimationProperty) -> Option<f32> {
        for animation in &self.animations {
            if animation.target == *target && animation.property == *property {
                return Some(animation.current_value());
            }
        }
        None
    }

    /// Stop a specific animation
    pub fn stop(&mut self, animation_id: AnimationId) {
        self.animations.retain(|a| a.id != animation_id);
    }

    /// Stop all animations for a target
    pub fn stop_all_for_target(&mut self, target: &AnimationTarget) {
        self.animations.retain(|a| a.target != *target);
    }

    /// Stop all animations for a property
    pub fn stop_all_for_property(&mut self, property: &AnimationProperty) {
        self.animations.retain(|a| a.property != *property);
    }

    /// Stop all animations
    pub fn stop_all(&mut self) {
        self.animations.clear();
    }

    /// Get number of active animations
    pub fn active_count(&self) -> usize {
        self.animations.len()
    }

    /// Check if an animation is running
    pub fn is_animating(&self, target: &AnimationTarget, property: &AnimationProperty) -> bool {
        self.animations.iter().any(|a| a.target == *target && a.property == *property)
    }

    /// Get next animation ID
    fn next_animation_id(&mut self) -> AnimationId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Get all animations for a target
    pub fn get_animations_for_target(&self, target: &AnimationTarget) -> Vec<&Animation> {
        self.animations.iter().filter(|a| a.target == *target).collect()
    }
}

impl Default for Animator {
    fn default() -> Self {
        Self::new()
    }
}
