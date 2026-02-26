// src/animation/tween.rs
use crate::animation::Easing;
use keyframe::functions::{EaseIn, EaseInOut, EaseOut, Linear};
use keyframe::{ease, CanTween};

/// A tween that interpolates between two states of type P over a duration.
pub struct Tween<P: CanTween + Clone> {
    pub start: P,
    pub end: P,
    /// Absolute scene time (seconds) when the tween begins.
    pub start_time: f64,
    /// Total tween duration in seconds.
    pub duration: f64,
    pub easing: Easing,
}

impl<P: CanTween + Clone> Tween<P> {
    /// Create a fluent builder for a Tween interpolating from `start` to `end`.
    ///
    /// Chain `.start_at()`, `.over()`, `.easing()` then `.build()` to produce a
    /// `Tween`. Defaults: start_time = 0.0, duration = 1.0, easing = Linear.
    pub fn build(start: P, end: P) -> TweenBuilder<P> {
        TweenBuilder {
            start,
            end,
            start_time: 0.0,
            duration: 1.0,
            easing: Easing::Linear,
        }
    }

    /// Evaluate the tween at absolute scene time t_secs.
    /// Returns start before the tween begins, end after it completes.
    /// local_t is always clamped to [0.0, 1.0] before calling keyframe::ease().
    pub fn value_at(&self, t_secs: f64) -> P {
        let local_t = ((t_secs - self.start_time) / self.duration).clamp(0.0, 1.0);
        match self.easing {
            Easing::Linear => ease(Linear, self.start.clone(), self.end.clone(), local_t),
            Easing::EaseIn => ease(EaseIn, self.start.clone(), self.end.clone(), local_t),
            Easing::EaseOut => ease(EaseOut, self.start.clone(), self.end.clone(), local_t),
            Easing::EaseInOut => ease(EaseInOut, self.start.clone(), self.end.clone(), local_t),
        }
    }
}

/// Fluent builder for `Tween`. Construct via `Tween::build(start, end)`.
pub struct TweenBuilder<P: CanTween + Clone> {
    start: P,
    end: P,
    start_time: f64,
    duration: f64,
    easing: Easing,
}

impl<P: CanTween + Clone> TweenBuilder<P> {
    /// Set the absolute start time in seconds. Default: 0.0.
    pub fn start_at(mut self, t: f64) -> Self {
        self.start_time = t;
        self
    }

    /// Set the tween duration in seconds. Default: 1.0.
    pub fn over(mut self, duration: f64) -> Self {
        self.duration = duration;
        self
    }

    /// Set the easing function. Default: `Easing::Linear`.
    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    /// Consume the builder and produce a `Tween`.
    pub fn build(self) -> Tween<P> {
        Tween {
            start: self.start,
            end: self.end,
            start_time: self.start_time,
            duration: self.duration,
            easing: self.easing,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation::Easing;
    use crate::primitives::circle::CircleState;

    fn make_tween(easing: Easing) -> Tween<CircleState> {
        Tween {
            start: CircleState {
                cx: 100.0,
                cy: 300.0,
                r: 80.0,
                fill_r: 255.0,
                fill_g: 0.0,
                fill_b: 0.0,
                opacity: 1.0,
            },
            end: CircleState {
                cx: 600.0,
                cy: 300.0,
                r: 80.0,
                fill_r: 255.0,
                fill_g: 0.0,
                fill_b: 255.0,
                opacity: 1.0,
            },
            start_time: 0.0,
            duration: 3.0,
            easing,
        }
    }

    #[test]
    fn tween_linear_t0_returns_start() {
        let tw = make_tween(Easing::Linear);
        let v = tw.value_at(0.0);
        assert_eq!(v.cx, 100.0);
        assert_eq!(v.fill_b, 0.0);
    }

    #[test]
    fn tween_linear_t_duration_returns_end() {
        let tw = make_tween(Easing::Linear);
        let v = tw.value_at(3.0);
        assert_eq!(v.cx, 600.0);
        assert_eq!(v.fill_b, 255.0);
    }

    #[test]
    fn tween_linear_midpoint_is_arithmetic_mean() {
        let tw = make_tween(Easing::Linear);
        let v = tw.value_at(1.5); // t=0.5
        assert!(
            (v.cx - 350.0).abs() < 1e-9,
            "Expected cx=350.0 at midpoint, got {}",
            v.cx
        );
    }

    #[test]
    fn tween_easeinout_differs_from_linear_at_quarter() {
        let tw_linear = make_tween(Easing::Linear);
        let tw_ease = make_tween(Easing::EaseInOut);
        // At t=0.25 (25% through the tween), EaseInOut lags behind Linear —
        // the function is accelerating (ease-in phase), so cx should be less than
        // the linear quarter-point of 225.0.
        let linear_quarter = tw_linear.value_at(0.75); // 0.75s = local_t=0.25
        let ease_quarter = tw_ease.value_at(0.75);
        assert!(
            (linear_quarter.cx - ease_quarter.cx).abs() > 1e-6,
            "EaseInOut at t=0.25 should differ from Linear, both were {}",
            linear_quarter.cx
        );
    }

    #[test]
    fn tween_clamps_before_start() {
        let tw = make_tween(Easing::Linear);
        let v = tw.value_at(-5.0);
        assert_eq!(v.cx, 100.0, "t before start_time should clamp to start");
    }

    #[test]
    fn tween_clamps_after_end() {
        let tw = make_tween(Easing::Linear);
        let v = tw.value_at(100.0);
        assert_eq!(v.cx, 600.0, "t after duration should clamp to end");
    }

    #[test]
    fn tween_circle_to_circle_fill_red() {
        let tw = make_tween(Easing::Linear);
        let v = tw.value_at(0.0);
        let c = v.to_circle();
        assert_eq!(c.fill, Some(crate::Color::RED));
        assert_eq!(c.opacity, 1.0);
    }

    #[test]
    fn tween_builder_defaults() {
        use crate::Color;
        let s1 = CircleState::new(0.0, 0.0, 10.0, Color::RED, 1.0);
        let s2 = CircleState::new(100.0, 0.0, 10.0, Color::BLUE, 1.0);
        let tw = Tween::build(s1, s2).build();
        assert_eq!(tw.start_time, 0.0);
        assert_eq!(tw.duration, 1.0);
        assert_eq!(tw.easing, Easing::Linear);
    }

    #[test]
    fn tween_builder_chained() {
        use crate::Color;
        let s1 = CircleState::new(0.0, 0.0, 10.0, Color::RED, 1.0);
        let s2 = CircleState::new(100.0, 0.0, 10.0, Color::BLUE, 1.0);
        let tw = Tween::build(s1, s2)
            .start_at(2.0)
            .over(5.0)
            .easing(Easing::EaseInOut)
            .build();
        assert_eq!(tw.start_time, 2.0);
        assert_eq!(tw.duration, 5.0);
        assert_eq!(tw.easing, Easing::EaseInOut);
    }
}
