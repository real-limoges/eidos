// src/dataviz/spline_fit.rs

use crate::{Color, EidosError, Easing, Tween};
use crate::primitives::Bezier;
use crate::dataviz::spline::catmull_rom_segment_to_bezier;

/// Internal animation config stored by animate_fit().
struct FitAnimation {
    start_time: f64,
    duration:   f64,
    easing:     Easing,
}

/// A spline curve that animates from invisible to its final fitted shape.
///
/// Animation: left-to-right reveal + y-value morphing from mean_y to fitted_y,
/// both driven by a scalar Tween<f64> progress in [0.0, 1.0].
///
/// Points are in DATA SPACE. The CALLER maps them to visual (pixel) space before
/// calling to_bezier(). See Pattern 3 in RESEARCH.md.
///
/// Without animate_fit(), renders fully revealed at any t_secs.
#[derive(Debug, Clone)]
pub struct SplineFit {
    /// Data-space points, sorted by x ascending at construction.
    pub points: Vec<(f64, f64)>,
    pub stroke_color: Color,
    pub stroke_width: f64,
    animation: Option<FitAnimation>,
}

impl std::fmt::Debug for FitAnimation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FitAnimation")
            .field("start_time", &self.start_time)
            .field("duration", &self.duration)
            .finish()
    }
}

impl Clone for FitAnimation {
    fn clone(&self) -> Self {
        FitAnimation {
            start_time: self.start_time,
            duration:   self.duration,
            easing:     self.easing,
        }
    }
}

impl SplineFit {
    /// Create a SplineFit from a Vec of fitted data points.
    /// Points are sorted by x ascending at construction.
    /// Returns Err(InvalidConfig) if fewer than 2 points are provided.
    pub fn new(mut points: Vec<(f64, f64)>) -> Result<Self, EidosError> {
        if points.len() < 2 {
            return Err(EidosError::InvalidConfig(
                "SplineFit requires at least 2 data points".into(),
            ));
        }
        points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        Ok(SplineFit {
            points,
            stroke_color: Color::WHITE,
            stroke_width: 2.0,
            animation: None,
        })
    }

    /// Set stroke color (infallible builder).
    pub fn color(mut self, color: Color) -> Self {
        self.stroke_color = color;
        self
    }

    /// Set stroke width. Negative values are clamped to 0.0.
    pub fn stroke_width(mut self, width: f64) -> Self {
        self.stroke_width = width.max(0.0);
        self
    }

    /// Configure the left-to-right reveal animation.
    ///
    /// - `start_time`: scene time (seconds) when animation begins.
    /// - `duration`: animation duration in seconds.
    /// - `easing`: easing function for the reveal.
    pub fn animate_fit(mut self, start_time: f64, duration: f64, easing: Easing) -> Self {
        self.animation = Some(FitAnimation { start_time, duration, easing });
        self
    }

    /// Evaluate the spline at scene time `t_secs` and produce a Bezier, or None if
    /// fewer than 2 points are currently revealed.
    ///
    /// `visual_pts`: caller-provided visual-space mapping of `self.points`
    /// (same order, same count). These MUST be computed by the caller using
    /// Axes coordinate mapping before the render loop.
    ///
    /// Algorithm:
    /// 1. Compute progress = Tween<f64>(0.0→1.0) value_at(t_secs), or 1.0 if no animation.
    /// 2. Reveal frontier: x <= visual_pts[0].x + progress * (visual_pts[-1].x - visual_pts[0].x)
    /// 3. For each revealed point: morphed_y = mean_y + progress * (fitted_y - mean_y)
    ///    where mean_y = average of ALL visual_pts y-values (computed in visual space)
    /// 4. Build Catmull-Rom Bezier from morphed points (same phantom endpoint duplication as DataCurve)
    /// 5. Return None if morphed.len() < 2
    pub fn to_bezier(&self, visual_pts: &[(f64, f64)], t_secs: f64) -> Option<Bezier> {
        if visual_pts.len() < 2 {
            return None;
        }

        let progress: f64 = match &self.animation {
            None => 1.0,
            Some(anim) => {
                let tween = Tween {
                    start: 0.0_f64,
                    end:   1.0_f64,
                    start_time: anim.start_time,
                    duration:   anim.duration,
                    easing:     anim.easing,
                };
                tween.value_at(t_secs)
            }
        };

        let x_min = visual_pts.first().unwrap().0;
        let x_max = visual_pts.last().unwrap().0;
        let frontier_x = x_min + progress * (x_max - x_min);

        // mean_y computed in visual space (after coordinate mapping)
        let mean_y: f64 = visual_pts.iter().map(|p| p.1).sum::<f64>() / visual_pts.len() as f64;

        let morphed: Vec<(f64, f64)> = visual_pts
            .iter()
            .filter(|(x, _)| *x <= frontier_x + 1e-9)
            .map(|&(x, fitted_y)| {
                let morphed_y = mean_y + progress * (fitted_y - mean_y);
                (x, morphed_y)
            })
            .collect();

        if morphed.len() < 2 {
            return None;
        }

        // Build Catmull-Rom path with phantom endpoint duplication
        let n = morphed.len();
        let mut bez = Bezier::new().move_to(morphed[0].0, morphed[0].1);
        for i in 0..(n - 1) {
            let p0 = if i == 0 { morphed[0] } else { morphed[i - 1] };
            let p1 = morphed[i];
            let p2 = morphed[i + 1];
            let p3 = if i + 2 >= n { morphed[n - 1] } else { morphed[i + 2] };
            let (cp1, cp2, end) = catmull_rom_segment_to_bezier(p0, p1, p2, p3);
            bez = bez.cubic_to(cp1.0, cp1.1, cp2.0, cp2.1, end.0, end.1);
        }

        Some(
            bez.stroke(self.stroke_color, self.stroke_width),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation::Easing;

    #[test]
    fn spline_fit_requires_at_least_two_points() {
        assert!(SplineFit::new(vec![]).is_err());
        assert!(SplineFit::new(vec![(0.0, 1.0)]).is_err());
    }

    #[test]
    fn spline_fit_points_sorted_on_construction() {
        let sf = SplineFit::new(vec![(3.0, 1.0), (1.0, 2.0), (2.0, 3.0)]).unwrap();
        assert_eq!(sf.points[0].0, 1.0);
        assert_eq!(sf.points[1].0, 2.0);
        assert_eq!(sf.points[2].0, 3.0);
    }

    #[test]
    fn spline_fit_no_animation_fully_revealed_at_any_t() {
        let sf = SplineFit::new(vec![(0.0, 0.0), (1.0, 1.0), (2.0, 2.0)]).unwrap();
        let visual_pts = &[(0.0, 500.0), (400.0, 300.0), (800.0, 100.0)];
        // Without animate_fit(), always fully revealed regardless of t
        let result = sf.to_bezier(visual_pts, 0.0);
        assert!(result.is_some(), "Expected Some at t=0 without animation");
        let result_late = sf.to_bezier(visual_pts, 1000.0);
        assert!(result_late.is_some(), "Expected Some at large t without animation");
    }

    #[test]
    fn spline_fit_animation_returns_none_at_t0_with_sparse_points() {
        // With only 2 visual points and progress near 0, the frontier is at x_min,
        // so only the first point passes the filter → morphed.len() == 1 → None.
        let sf = SplineFit::new(vec![(0.0, 0.0), (100.0, 50.0)])
            .unwrap()
            .animate_fit(0.0, 10.0, Easing::Linear);
        // visual_pts: x spans 0.0 to 800.0
        let visual_pts = &[(0.0, 500.0), (800.0, 200.0)];
        // At t very slightly before start, progress clamps to 0.0,
        // frontier_x = 0.0 + 0.0 * 800.0 = 0.0
        // Only point with x <= 0.0 + 1e-9 is (0.0, 500.0) → 1 point → None
        let result = sf.to_bezier(visual_pts, -1.0);
        assert!(result.is_none(), "Expected None when only 1 point revealed");
    }

    #[test]
    fn spline_fit_animation_returns_some_at_full_progress() {
        // At t = start_time + duration, progress = 1.0, all points revealed → Some
        let start = 0.0_f64;
        let duration = 5.0_f64;
        let sf = SplineFit::new(vec![(0.0, 10.0), (50.0, 30.0), (100.0, 20.0)])
            .unwrap()
            .animate_fit(start, duration, Easing::Linear);
        let visual_pts = &[(0.0, 400.0), (400.0, 200.0), (800.0, 300.0)];
        let result = sf.to_bezier(visual_pts, start + duration);
        assert!(result.is_some(), "Expected Some at t=duration (fully revealed)");
    }
}
