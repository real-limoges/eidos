// src/dataviz/data_curve.rs

use crate::{Color, EidosError};
use crate::primitives::Bezier;
use crate::dataviz::spline::catmull_rom_segment_to_bezier;

/// A smooth data curve rendered as a cubic spline through the provided data points.
///
/// `to_bezier_path()` converts the data points to a `Bezier` path using
/// Catmull-Rom -> cubic Bezier conversion. The caller (Axes::to_primitives) is
/// responsible for coordinate-mapping data points to pixel space BEFORE calling
/// to_bezier_path -- spline control points must be computed in visual space.
#[derive(Debug, Clone)]
pub struct DataCurve {
    pub points: Vec<(f64, f64)>,
    pub stroke_color: Color,
    pub stroke_width: f64,
    pub opacity: f64,
}

impl DataCurve {
    /// Create a new DataCurve from a Vec of (x, y) data points.
    /// Returns Err(InvalidConfig) if fewer than 2 points are provided.
    pub fn new(points: Vec<(f64, f64)>) -> Result<Self, EidosError> {
        if points.len() < 2 {
            return Err(EidosError::InvalidConfig(
                "DataCurve requires at least 2 data points".into(),
            ));
        }
        Ok(DataCurve {
            points,
            stroke_color: Color::WHITE,
            stroke_width: 2.0,
            opacity: 1.0,
        })
    }

    /// Set stroke color and width. Returns Err if width is negative.
    pub fn stroke(mut self, color: Color, width: f64) -> Result<Self, EidosError> {
        if width < 0.0 {
            return Err(EidosError::InvalidConfig(
                "stroke width must be non-negative".into(),
            ));
        }
        self.stroke_color = color;
        self.stroke_width = width;
        Ok(self)
    }

    /// Set opacity in [0.0, 1.0]. Returns Err if outside range.
    pub fn opacity(mut self, value: f64) -> Result<Self, EidosError> {
        if !(0.0..=1.0).contains(&value) {
            return Err(EidosError::InvalidConfig(
                "opacity must be in range [0.0, 1.0]".into(),
            ));
        }
        self.opacity = value;
        Ok(self)
    }

    /// Convert pre-mapped visual-space points to a Bezier path using Catmull-Rom spline.
    ///
    /// IMPORTANT: `visual_points` must already be in pixel coordinates (not data space).
    /// Catmull-Rom tangents depend on point distances -- computing in data space with
    /// asymmetric X/Y scales produces distorted curves.
    ///
    /// Uses phantom endpoint duplication at both boundaries to prevent kinks:
    ///   p[-1] = p[0]  (first segment phantom)
    ///   p[n]  = p[n-1] (last segment phantom)
    pub fn to_bezier_path(&self, visual_points: &[(f64, f64)]) -> Bezier {
        assert!(visual_points.len() >= 2, "to_bezier_path requires at least 2 points");

        let n = visual_points.len();
        let mut bez = Bezier::new().move_to(visual_points[0].0, visual_points[0].1);

        for i in 0..(n - 1) {
            // Phantom point duplication at boundaries:
            // p0 is the point before p1; phantom at start = p[0]
            // p3 is the point after p2; phantom at end = p[n-1]
            let p0 = if i == 0 { visual_points[0] } else { visual_points[i - 1] };
            let p1 = visual_points[i];
            let p2 = visual_points[i + 1];
            let p3 = if i + 2 >= n { visual_points[n - 1] } else { visual_points[i + 2] };

            let (cp1, cp2, end) = catmull_rom_segment_to_bezier(p0, p1, p2, p3);
            bez = bez.cubic_to(cp1.0, cp1.1, cp2.0, cp2.1, end.0, end.1);
        }

        // Apply stroke and opacity -- these are infallible because we already validated them
        // in DataCurve::stroke() and DataCurve::opacity(); use defaults here.
        bez.stroke(self.stroke_color, self.stroke_width)
           .expect("stroke validated at construction")
           .opacity(self.opacity)
           .expect("opacity validated at construction")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Color;

    #[test]
    fn data_curve_requires_at_least_two_points() {
        assert!(DataCurve::new(vec![]).is_err());
        assert!(DataCurve::new(vec![(0.0, 0.0)]).is_err());
    }

    #[test]
    fn data_curve_two_points_ok() {
        let result = DataCurve::new(vec![(0.0, 0.0), (1.0, 1.0)]);
        assert!(result.is_ok());
    }

    #[test]
    fn data_curve_negative_stroke_width_returns_err() {
        let c = DataCurve::new(vec![(0.0, 0.0), (1.0, 1.0)]).unwrap();
        assert!(c.stroke(Color::WHITE, -1.0).is_err());
    }

    #[test]
    fn data_curve_opacity_out_of_range_returns_err() {
        let c = DataCurve::new(vec![(0.0, 0.0), (1.0, 1.0)]).unwrap();
        assert!(c.opacity(1.5).is_err());
    }

    #[test]
    fn catmull_rom_interior_segment_produces_smooth_control_points() {
        // With p0=p1=p2=p3=(0,0), control points should all be (0,0)
        let (cp1, cp2, end) = catmull_rom_segment_to_bezier(
            (0.0, 0.0), (0.0, 0.0), (0.0, 0.0), (0.0, 0.0),
        );
        assert!((cp1.0).abs() < 1e-10);
        assert!((cp2.0).abs() < 1e-10);
        assert_eq!(end, (0.0, 0.0));
    }

    #[test]
    fn to_bezier_path_two_points_produces_bezier_with_move_and_cubic() {
        let curve = DataCurve::new(vec![(0.0, 0.0), (100.0, 100.0)]).unwrap();
        let visual_pts = &[(0.0, 500.0), (800.0, 100.0)];
        let bez = curve.to_bezier_path(visual_pts);
        // Should have: MoveTo + CubicTo = 2 commands
        assert_eq!(bez.commands.len(), 2);
    }

    #[test]
    fn to_bezier_path_multiple_points_has_correct_command_count() {
        let pts: Vec<(f64, f64)> = (0..5).map(|i| (i as f64, (i as f64).sin())).collect();
        let curve = DataCurve::new(pts.clone()).unwrap();
        // Map data to visual space (identity for this test)
        let visual: Vec<(f64, f64)> = pts.iter().map(|&(x, y)| (x * 100.0, 500.0 - y * 100.0)).collect();
        let bez = curve.to_bezier_path(&visual);
        // 1 MoveTo + (n-1) CubicTo = n commands
        assert_eq!(bez.commands.len(), pts.len());
    }
}
