/// Easing Functions - Phase G.6
///
/// Easing functions for smooth animations

/// Easing function type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EasingFunction {
    /// Linear interpolation (no easing)
    Linear,
    /// Ease in (slow start)
    EaseIn,
    /// Ease out (slow end)
    EaseOut,
    /// Ease in and out (slow start and end)
    EaseInOut,
    /// Quadratic ease in
    EaseInQuad,
    /// Quadratic ease out
    EaseOutQuad,
    /// Quadratic ease in-out
    EaseInOutQuad,
    /// Cubic ease in
    EaseInCubic,
    /// Cubic ease out
    EaseOutCubic,
    /// Cubic ease in-out
    EaseInOutCubic,
    /// Bounce effect
    Bounce,
    /// Elastic effect
    Elastic,
    /// Back easing (overshoot)
    Back,
}

/// Apply an easing function to a normalized time value (0.0 - 1.0)
pub fn apply_easing(t: f32, easing: &EasingFunction) -> f32 {
    let t = t.clamp(0.0, 1.0);

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

        EasingFunction::EaseInQuad => t * t,

        EasingFunction::EaseOutQuad => t * (2.0 - t),

        EasingFunction::EaseInOutQuad => {
            if t < 0.5 {
                2.0 * t * t
            } else {
                -1.0 + (4.0 - 2.0 * t) * t
            }
        }

        EasingFunction::EaseInCubic => t * t * t,

        EasingFunction::EaseOutCubic => {
            let t = t - 1.0;
            t * t * t + 1.0
        }

        EasingFunction::EaseInOutCubic => {
            if t < 0.5 {
                4.0 * t * t * t
            } else {
                let t = 2.0 * t - 2.0;
                (t * t * t + 2.0) / 2.0
            }
        }

        EasingFunction::Bounce => {
            if t < 0.5 {
                8.0 * (1.0 - 2.0 * t).powi(2) * (1.0 - t.abs())
            } else {
                1.0 - 8.0 * (1.0 - t).powi(2) * (1.0 - (1.0 - t).abs())
            }
        }

        EasingFunction::Elastic => {
            if t == 0.0 || t == 1.0 {
                t
            } else {
                let p = 0.3;
                let s = p / 4.0;
                let t = t - 1.0;
                -(2.0_f32.powf(10.0 * t) * ((t - s) * (2.0 * core::f32::consts::PI / p)).sin())
            }
        }

        EasingFunction::Back => {
            let c1 = 1.70158;
            let c3 = c1 + 1.0;
            c3 * t * t * t - c1 * t * t
        }
    }
}

/// Linear interpolation between two values
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Smooth step interpolation (smoother than linear)
pub fn smooth_step(a: f32, b: f32, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let t = t * t * (3.0 - 2.0 * t);
    lerp(a, b, t)
}

/// Smoother step interpolation (even smoother)
pub fn smoother_step(a: f32, b: f32, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let t = t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
    lerp(a, b, t)
}

/// Inverse lerp (find t given a, b, and value)
pub fn inverse_lerp(a: f32, b: f32, value: f32) -> f32 {
    if (b - a).abs() < f32::EPSILON {
        0.0
    } else {
        (value - a) / (b - a)
    }
}

/// Remap a value from one range to another
pub fn remap(value: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    let t = inverse_lerp(in_min, in_max, value);
    lerp(out_min, out_max, t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lerp() {
        assert_eq!(lerp(0.0, 10.0, 0.0), 0.0);
        assert_eq!(lerp(0.0, 10.0, 1.0), 10.0);
        assert_eq!(lerp(0.0, 10.0, 0.5), 5.0);
    }

    #[test]
    fn test_easing_bounds() {
        let easings = [
            EasingFunction::Linear,
            EasingFunction::EaseIn,
            EasingFunction::EaseOut,
            EasingFunction::EaseInOut,
        ];

        for easing in &easings {
            assert_eq!(apply_easing(0.0, easing), 0.0);
            // EaseOut and EaseInOut should be close to 1.0 at t=1.0
            let result = apply_easing(1.0, easing);
            assert!(result >= 0.9 && result <= 1.1, "Easing {:?} at t=1.0 = {}", easing, result);
        }
    }

    #[test]
    fn test_remap() {
        // Remap 5 from [0, 10] to [0, 100]
        assert_eq!(remap(5.0, 0.0, 10.0, 0.0, 100.0), 50.0);

        // Remap 0 from [-1, 1] to [0, 1]
        assert_eq!(remap(0.0, -1.0, 1.0, 0.0, 1.0), 0.5);
    }
}
