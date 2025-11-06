/// Transitions - Phase G.6
///
/// Common animation patterns and transitions

use super::{Animator, AnimationTarget, AnimationProperty, EasingFunction, AnimationId};
use alloc::vec::Vec;

/// Transition type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionType {
    /// Fade in (opacity 0 → 1)
    FadeIn,
    /// Fade out (opacity 1 → 0)
    FadeOut,
    /// Slide in from left
    SlideInLeft,
    /// Slide in from right
    SlideInRight,
    /// Slide in from top
    SlideInTop,
    /// Slide in from bottom
    SlideInBottom,
    /// Slide out to left
    SlideOutLeft,
    /// Slide out to right
    SlideOutRight,
    /// Slide out to top
    SlideOutTop,
    /// Slide out to bottom
    SlideOutBottom,
    /// Scale up (small → normal)
    ScaleUp,
    /// Scale down (normal → small)
    ScaleDown,
    /// Zoom in (scale up + fade in)
    ZoomIn,
    /// Zoom out (scale down + fade out)
    ZoomOut,
}

/// Transition configuration
pub struct Transition {
    pub transition_type: TransitionType,
    pub duration_ms: u32,
    pub easing: EasingFunction,
}

impl Transition {
    /// Create a new transition
    pub fn new(transition_type: TransitionType, duration_ms: u32) -> Self {
        let easing = match transition_type {
            TransitionType::FadeIn | TransitionType::FadeOut => EasingFunction::EaseInOut,
            TransitionType::ScaleUp | TransitionType::ScaleDown => EasingFunction::EaseOut,
            TransitionType::ZoomIn | TransitionType::ZoomOut => EasingFunction::EaseInOutCubic,
            _ => EasingFunction::EaseOut,
        };

        Self {
            transition_type,
            duration_ms,
            easing,
        }
    }

    /// Create with custom easing
    pub fn with_easing(mut self, easing: EasingFunction) -> Self {
        self.easing = easing;
        self
    }

    /// Apply transition to a target
    pub fn apply(&self, animator: &mut Animator, target: AnimationTarget) -> Vec<AnimationId> {
        let mut animation_ids = Vec::new();

        match self.transition_type {
            TransitionType::FadeIn => {
                let id = animator.animate(
                    target,
                    AnimationProperty::Opacity,
                    0.0,
                    1.0,
                    self.duration_ms,
                    self.easing,
                );
                animation_ids.push(id);
            }

            TransitionType::FadeOut => {
                let id = animator.animate(
                    target,
                    AnimationProperty::Opacity,
                    1.0,
                    0.0,
                    self.duration_ms,
                    self.easing,
                );
                animation_ids.push(id);
            }

            TransitionType::SlideInLeft => {
                let id = animator.animate(
                    target,
                    AnimationProperty::X,
                    -800.0, // Assume screen width ~800px
                    0.0,
                    self.duration_ms,
                    self.easing,
                );
                animation_ids.push(id);
            }

            TransitionType::SlideInRight => {
                let id = animator.animate(
                    target,
                    AnimationProperty::X,
                    800.0,
                    0.0,
                    self.duration_ms,
                    self.easing,
                );
                animation_ids.push(id);
            }

            TransitionType::SlideInTop => {
                let id = animator.animate(
                    target,
                    AnimationProperty::Y,
                    -600.0, // Assume screen height ~600px
                    0.0,
                    self.duration_ms,
                    self.easing,
                );
                animation_ids.push(id);
            }

            TransitionType::SlideInBottom => {
                let id = animator.animate(
                    target,
                    AnimationProperty::Y,
                    600.0,
                    0.0,
                    self.duration_ms,
                    self.easing,
                );
                animation_ids.push(id);
            }

            TransitionType::SlideOutLeft => {
                let id = animator.animate(
                    target,
                    AnimationProperty::X,
                    0.0,
                    -800.0,
                    self.duration_ms,
                    self.easing,
                );
                animation_ids.push(id);
            }

            TransitionType::SlideOutRight => {
                let id = animator.animate(
                    target,
                    AnimationProperty::X,
                    0.0,
                    800.0,
                    self.duration_ms,
                    self.easing,
                );
                animation_ids.push(id);
            }

            TransitionType::SlideOutTop => {
                let id = animator.animate(
                    target,
                    AnimationProperty::Y,
                    0.0,
                    -600.0,
                    self.duration_ms,
                    self.easing,
                );
                animation_ids.push(id);
            }

            TransitionType::SlideOutBottom => {
                let id = animator.animate(
                    target,
                    AnimationProperty::Y,
                    0.0,
                    600.0,
                    self.duration_ms,
                    self.easing,
                );
                animation_ids.push(id);
            }

            TransitionType::ScaleUp => {
                let id = animator.animate(
                    target,
                    AnimationProperty::Scale,
                    0.0,
                    1.0,
                    self.duration_ms,
                    self.easing,
                );
                animation_ids.push(id);
            }

            TransitionType::ScaleDown => {
                let id = animator.animate(
                    target,
                    AnimationProperty::Scale,
                    1.0,
                    0.0,
                    self.duration_ms,
                    self.easing,
                );
                animation_ids.push(id);
            }

            TransitionType::ZoomIn => {
                // Scale up
                let id1 = animator.animate(
                    target,
                    AnimationProperty::Scale,
                    0.8,
                    1.0,
                    self.duration_ms,
                    self.easing,
                );
                animation_ids.push(id1);

                // Fade in
                let id2 = animator.animate(
                    target,
                    AnimationProperty::Opacity,
                    0.0,
                    1.0,
                    self.duration_ms,
                    self.easing,
                );
                animation_ids.push(id2);
            }

            TransitionType::ZoomOut => {
                // Scale down
                let id1 = animator.animate(
                    target,
                    AnimationProperty::Scale,
                    1.0,
                    0.8,
                    self.duration_ms,
                    self.easing,
                );
                animation_ids.push(id1);

                // Fade out
                let id2 = animator.animate(
                    target,
                    AnimationProperty::Opacity,
                    1.0,
                    0.0,
                    self.duration_ms,
                    self.easing,
                );
                animation_ids.push(id2);
            }
        }

        animation_ids
    }
}

/// Predefined transition presets
impl Transition {
    /// Fast fade in (200ms)
    pub fn fade_in_fast() -> Self {
        Self::new(TransitionType::FadeIn, 200)
    }

    /// Normal fade in (400ms)
    pub fn fade_in() -> Self {
        Self::new(TransitionType::FadeIn, 400)
    }

    /// Slow fade in (600ms)
    pub fn fade_in_slow() -> Self {
        Self::new(TransitionType::FadeIn, 600)
    }

    /// Fast fade out (200ms)
    pub fn fade_out_fast() -> Self {
        Self::new(TransitionType::FadeOut, 200)
    }

    /// Normal fade out (400ms)
    pub fn fade_out() -> Self {
        Self::new(TransitionType::FadeOut, 400)
    }

    /// Slow fade out (600ms)
    pub fn fade_out_slow() -> Self {
        Self::new(TransitionType::FadeOut, 600)
    }

    /// Slide in from left
    pub fn slide_in_left() -> Self {
        Self::new(TransitionType::SlideInLeft, 300)
    }

    /// Slide out to right
    pub fn slide_out_right() -> Self {
        Self::new(TransitionType::SlideOutRight, 300)
    }

    /// Zoom in effect
    pub fn zoom_in() -> Self {
        Self::new(TransitionType::ZoomIn, 400)
    }

    /// Zoom out effect
    pub fn zoom_out() -> Self {
        Self::new(TransitionType::ZoomOut, 400)
    }

    /// Scale up (pop in)
    pub fn scale_up() -> Self {
        Self::new(TransitionType::ScaleUp, 250)
            .with_easing(EasingFunction::Back)
    }

    /// Scale down (pop out)
    pub fn scale_down() -> Self {
        Self::new(TransitionType::ScaleDown, 200)
            .with_easing(EasingFunction::EaseInCubic)
    }
}

/// Convenience functions for common transitions
pub fn fade_in_window(animator: &mut Animator, window_id: u32, duration_ms: u32) -> AnimationId {
    animator.animate(
        AnimationTarget::Window(window_id),
        AnimationProperty::Opacity,
        0.0,
        1.0,
        duration_ms,
        EasingFunction::EaseInOut,
    )
}

pub fn fade_out_window(animator: &mut Animator, window_id: u32, duration_ms: u32) -> AnimationId {
    animator.animate(
        AnimationTarget::Window(window_id),
        AnimationProperty::Opacity,
        1.0,
        0.0,
        duration_ms,
        EasingFunction::EaseInOut,
    )
}

pub fn slide_in_window(animator: &mut Animator, window_id: u32, from_x: f32, to_x: f32, duration_ms: u32) -> AnimationId {
    animator.animate(
        AnimationTarget::Window(window_id),
        AnimationProperty::X,
        from_x,
        to_x,
        duration_ms,
        EasingFunction::EaseOut,
    )
}

pub fn scale_window(animator: &mut Animator, window_id: u32, from_scale: f32, to_scale: f32, duration_ms: u32) -> AnimationId {
    animator.animate(
        AnimationTarget::Window(window_id),
        AnimationProperty::Scale,
        from_scale,
        to_scale,
        duration_ms,
        EasingFunction::EaseOut,
    )
}
