// src/dataviz/spline.rs
//
// Shared Catmull-Rom -> cubic Bezier conversion helper.
// Used by DataCurve, ConfidenceBand, and SplineFit.

/// Convert a Catmull-Rom segment (4 points) to cubic Bezier control points.
///
/// Source: formula from "Conversion Between Cubic Bezier Curves and Catmull-Rom Splines",
/// Tamaghna et al. 2020 (arXiv:2011.08232), also reproduced at:
/// https://gist.github.com/njvack/6925609
///
/// Given 4 Catmull-Rom points p0, p1, p2, p3:
///   Bezier control points for segment p1 -> p2:
///     cp1 = (-p0 + 6*p1 + p2) / 6   [component-wise]
///     cp2 = (p1 + 6*p2 - p3) / 6    [component-wise]
///
/// Returns (cp1, cp2, p2) -- the two control points and the segment endpoint.
pub(crate) fn catmull_rom_segment_to_bezier(
    p0: (f64, f64),
    p1: (f64, f64),
    p2: (f64, f64),
    p3: (f64, f64),
) -> ((f64, f64), (f64, f64), (f64, f64)) {
    let cp1x = (-p0.0 + 6.0 * p1.0 + p2.0) / 6.0;
    let cp1y = (-p0.1 + 6.0 * p1.1 + p2.1) / 6.0;
    let cp2x = (p1.0 + 6.0 * p2.0 - p3.0) / 6.0;
    let cp2y = (p1.1 + 6.0 * p2.1 - p3.1) / 6.0;
    ((cp1x, cp1y), (cp2x, cp2y), p2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catmull_rom_all_same_points_returns_same_point() {
        let (cp1, cp2, end) =
            catmull_rom_segment_to_bezier((0.0, 0.0), (0.0, 0.0), (0.0, 0.0), (0.0, 0.0));
        assert!((cp1.0).abs() < 1e-10);
        assert!((cp1.1).abs() < 1e-10);
        assert!((cp2.0).abs() < 1e-10);
        assert!((cp2.1).abs() < 1e-10);
        assert_eq!(end, (0.0, 0.0));
    }

    #[test]
    fn catmull_rom_collinear_points_returns_endpoint() {
        // For perfectly linear points, endpoint is always p2
        let (_, _, end) =
            catmull_rom_segment_to_bezier((0.0, 0.0), (1.0, 1.0), (2.0, 2.0), (3.0, 3.0));
        assert!((end.0 - 2.0).abs() < 1e-10);
        assert!((end.1 - 2.0).abs() < 1e-10);
    }
}
