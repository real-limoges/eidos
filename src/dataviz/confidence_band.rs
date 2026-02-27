// src/dataviz/confidence_band.rs

use crate::dataviz::spline::catmull_rom_segment_to_bezier;
use crate::primitives::Bezier;
use crate::{Color, EidosError};

/// A shaded region between upper and lower bound curves.
///
/// Renders as a single closed SVG path: upper curve forward (Catmull-Rom) +
/// lower curve reversed + Z close. Fill-only — no stroke on the bound lines.
///
/// Points are in DATA SPACE. Axes::to_primitives() maps them to pixel space
/// before calling to_bezier_path().
#[derive(Debug, Clone)]
pub struct ConfidenceBand {
    pub upper_points: Vec<(f64, f64)>,
    pub lower_points: Vec<(f64, f64)>,
    pub fill_color: Color,
    pub opacity: f64,
}

impl ConfidenceBand {
    /// Create a ConfidenceBand. Requires >= 2 points in each bound array.
    pub fn new(upper: Vec<(f64, f64)>, lower: Vec<(f64, f64)>) -> Result<Self, EidosError> {
        if upper.len() < 2 {
            return Err(EidosError::InvalidConfig(
                "ConfidenceBand upper_points requires at least 2 points".into(),
            ));
        }
        if lower.len() < 2 {
            return Err(EidosError::InvalidConfig(
                "ConfidenceBand lower_points requires at least 2 points".into(),
            ));
        }
        Ok(ConfidenceBand {
            upper_points: upper,
            lower_points: lower,
            fill_color: Color::rgb(100, 149, 237), // cornflower blue default
            opacity: 0.25,                         // 25% — semi-transparent per CONTEXT.md
        })
    }

    /// Set fill color. Returns Self (builder pattern, infallible).
    pub fn fill_color(mut self, color: Color) -> Self {
        self.fill_color = color;
        self
    }

    /// Set opacity in [0.0, 1.0]. Values outside the range are clamped.
    pub fn opacity(mut self, value: f64) -> Self {
        self.opacity = value.clamp(0.0, 1.0);
        self
    }

    /// Convert pre-mapped visual-space upper and lower points to a closed filled Bezier path.
    ///
    /// IMPORTANT: visual_upper and visual_lower MUST already be in pixel coordinates.
    /// Tracing order: upper forward (Catmull-Rom), line_to first reversed-lower point,
    /// lower reversed (Catmull-Rom), close() back to first upper point.
    /// This ensures the path encloses the band with correct SVG winding.
    pub fn to_bezier_path(
        &self,
        visual_upper: &[(f64, f64)],
        visual_lower: &[(f64, f64)],
    ) -> Bezier {
        assert!(
            visual_upper.len() >= 2,
            "to_bezier_path requires at least 2 upper points"
        );
        assert!(
            visual_lower.len() >= 2,
            "to_bezier_path requires at least 2 lower points"
        );

        let n_upper = visual_upper.len();
        let lower_rev: Vec<(f64, f64)> = visual_lower.iter().rev().cloned().collect();
        let n_lower = lower_rev.len();

        // Start at the first upper point
        let mut bez = Bezier::new().move_to(visual_upper[0].0, visual_upper[0].1);

        // Forward upper curve using Catmull-Rom with phantom endpoint duplication
        for i in 0..(n_upper - 1) {
            let p0 = if i == 0 {
                visual_upper[0]
            } else {
                visual_upper[i - 1]
            };
            let p1 = visual_upper[i];
            let p2 = visual_upper[i + 1];
            let p3 = if i + 2 >= n_upper {
                visual_upper[n_upper - 1]
            } else {
                visual_upper[i + 2]
            };

            let (cp1, cp2, end) = catmull_rom_segment_to_bezier(p0, p1, p2, p3);
            bez = bez.cubic_to(cp1.0, cp1.1, cp2.0, cp2.1, end.0, end.1);
        }

        // Line to first reversed-lower point (connects upper end to lower end)
        bez = bez.line_to(lower_rev[0].0, lower_rev[0].1);

        // Reversed lower curve using Catmull-Rom with phantom endpoint duplication
        for i in 0..(n_lower - 1) {
            let p0 = if i == 0 {
                lower_rev[0]
            } else {
                lower_rev[i - 1]
            };
            let p1 = lower_rev[i];
            let p2 = lower_rev[i + 1];
            let p3 = if i + 2 >= n_lower {
                lower_rev[n_lower - 1]
            } else {
                lower_rev[i + 2]
            };

            let (cp1, cp2, end) = catmull_rom_segment_to_bezier(p0, p1, p2, p3);
            bez = bez.cubic_to(cp1.0, cp1.1, cp2.0, cp2.1, end.0, end.1);
        }

        // Close the path back to the first upper point
        bez = bez.close();

        // Fill-only: no stroke on the band boundary lines
        bez.fill(self.fill_color).opacity(self.opacity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Color;

    #[test]
    fn confidence_band_requires_at_least_two_points_per_bound() {
        // Both empty
        assert!(ConfidenceBand::new(vec![], vec![]).is_err());
        // Upper has only 1 point
        assert!(ConfidenceBand::new(vec![(0.0, 0.0)], vec![(0.0, 0.0), (1.0, 0.0)]).is_err());
        // Lower has only 1 point
        assert!(ConfidenceBand::new(vec![(0.0, 0.0), (1.0, 0.0)], vec![(0.0, 0.0)]).is_err());
    }

    #[test]
    fn confidence_band_two_points_ok() {
        let upper = vec![(0.0, 1.0), (1.0, 1.0)];
        let lower = vec![(0.0, 0.0), (1.0, 0.0)];
        let band = ConfidenceBand::new(upper, lower);
        assert!(band.is_ok());
        let b = band.unwrap();
        assert_eq!(b.opacity, 0.25);
        // Default fill color is cornflower blue
        assert_eq!(b.fill_color, Color::rgb(100, 149, 237));
    }

    #[test]
    fn confidence_band_fill_color_builder_sets_color() {
        let upper = vec![(0.0, 1.0), (1.0, 1.0)];
        let lower = vec![(0.0, 0.0), (1.0, 0.0)];
        let band = ConfidenceBand::new(upper, lower)
            .unwrap()
            .fill_color(Color::RED);
        assert_eq!(band.fill_color, Color::RED);
    }

    #[test]
    fn confidence_band_opacity_is_clamped() {
        let upper = vec![(0.0, 1.0), (1.0, 1.0)];
        let lower = vec![(0.0, 0.0), (1.0, 0.0)];
        let band = ConfidenceBand::new(upper, lower).unwrap();
        assert_eq!(band.opacity(1.5).opacity, 1.0);
        let upper = vec![(0.0, 1.0), (1.0, 1.0)];
        let lower = vec![(0.0, 0.0), (1.0, 0.0)];
        let band2 = ConfidenceBand::new(upper, lower).unwrap();
        assert_eq!(band2.opacity(-0.1).opacity, 0.0);
    }

    #[test]
    fn confidence_band_closed_path_has_move_line_cubic_commands() {
        let upper = vec![(0.0, 100.0), (100.0, 100.0)];
        let lower = vec![(0.0, 0.0), (100.0, 0.0)];
        let band = ConfidenceBand::new(upper.clone(), lower.clone()).unwrap();
        let bez = band.to_bezier_path(&upper, &lower);

        // Verify the path has a Close command
        let has_close = bez
            .commands
            .iter()
            .any(|cmd| matches!(cmd, crate::primitives::bezier::PathCommand::Close));
        assert!(has_close, "path must be closed");

        // Verify fill is set (no stroke on the boundary)
        assert!(bez.fill.is_some(), "band must have fill");
        assert!(bez.stroke.is_none(), "band must be fill-only, no stroke");

        // Verify opacity is 0.25 (default)
        assert!(
            (bez.opacity - 0.25).abs() < 1e-10,
            "default opacity should be 0.25"
        );

        // Verify command count: MoveTo + 1 CubicTo (upper) + LineTo + 1 CubicTo (lower) + Close = 5
        assert_eq!(
            bez.commands.len(),
            5,
            "expected 5 commands, got {}",
            bez.commands.len()
        );
    }
}
