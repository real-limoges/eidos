//! SurfacePlot: a pure data container for a regular 3D grid surface.
//!
//! # Input format
//! - `xs`, `ys`, `zs`: flat row-major vecs of length `rows * cols`
//! - Element at grid position (row, col) is at index `row * cols + col`
//!
//! # Normalization
//! Each axis is independently normalized to [-1, 1] world space at construction.
//! This ensures data scale differences never affect the visual proportions of the surface.
//! A flat surface (all z equal) maps all z values to 0.0 (center of [-1, 1]).
//!
//! # Axis labels and rendering config
//! These are Phase 6 rendering concerns. SurfacePlot is pure data — no rendering logic here.

use crate::dataviz::camera::Point3D;

/// A regular-grid 3D surface, stored as normalized world-space vertices.
///
/// Construct from flat row-major coordinate arrays. Retrieve normalized vertices
/// via [`SurfacePlot::world_point`] for use with [`crate::Camera::project_to_screen`].
///
/// ```
/// use eidos::SurfacePlot;
///
/// let xs = vec![0.0, 1.0, 2.0];
/// let ys = vec![0.0, 0.0, 0.0];
/// let zs = vec![0.0, 0.5, 1.0];
/// let plot = SurfacePlot::new(xs, ys, zs, 1, 3);
/// let p = plot.world_point(0, 0);
/// assert!((p.x - (-1.0)).abs() < 1e-10); // x_min normalizes to -1
/// ```
#[derive(Debug, Clone)]
pub struct SurfacePlot {
    rows: usize,
    cols: usize,
    /// Normalized world-space vertices in row-major order.
    /// Index: row * cols + col
    world_vertices: Vec<Point3D>,
}

impl SurfacePlot {
    /// Construct a SurfacePlot from flat row-major coordinate arrays.
    ///
    /// # Panics
    /// Panics with a clear message if `xs.len()`, `ys.len()`, or `zs.len()` != `rows * cols`.
    ///
    /// # Normalization
    /// Each axis is independently normalized to [-1, 1]. A degenerate axis
    /// (all values equal) maps to 0.0.
    pub fn new(xs: Vec<f64>, ys: Vec<f64>, zs: Vec<f64>, rows: usize, cols: usize) -> Self {
        let n = rows * cols;
        assert_eq!(
            xs.len(), n,
            "xs.len() ({}) != rows * cols ({}*{}={})",
            xs.len(), rows, cols, n
        );
        assert_eq!(
            ys.len(), n,
            "ys.len() ({}) != rows * cols ({}*{}={})",
            ys.len(), rows, cols, n
        );
        assert_eq!(
            zs.len(), n,
            "zs.len() ({}) != rows * cols ({}*{}={})",
            zs.len(), rows, cols, n
        );

        let world_vertices = normalize_to_world_space(&xs, &ys, &zs);
        SurfacePlot { rows, cols, world_vertices }
    }

    /// Returns the normalized world-space point at grid position (row, col).
    ///
    /// # Panics
    /// Panics if row >= self.rows() or col >= self.cols().
    pub fn world_point(&self, row: usize, col: usize) -> Point3D {
        self.world_vertices[row * self.cols + col]
    }

    /// Number of rows in the grid.
    pub fn rows(&self) -> usize {
        self.rows
    }

    /// Number of columns in the grid.
    pub fn cols(&self) -> usize {
        self.cols
    }
}

/// Normalize all three coordinate arrays independently to [-1, 1] world space.
fn normalize_to_world_space(xs: &[f64], ys: &[f64], zs: &[f64]) -> Vec<Point3D> {
    let (x_min, x_max) = min_max(xs);
    let (y_min, y_max) = min_max(ys);
    let (z_min, z_max) = min_max(zs);

    xs.iter()
        .zip(ys.iter())
        .zip(zs.iter())
        .map(|((x, y), z)| Point3D {
            x: normalize(*x, x_min, x_max),
            y: normalize(*y, y_min, y_max),
            z: normalize(*z, z_min, z_max),
        })
        .collect()
}

/// Compute the min and max of a slice.
fn min_max(vals: &[f64]) -> (f64, f64) {
    let lo = vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let hi = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    (lo, hi)
}

/// Normalize a value from [lo, hi] to [-1, 1].
///
/// Degenerate range (hi ≈ lo): returns 0.0 (center of [-1, 1]).
/// This handles flat surfaces where all z values are equal.
fn normalize(v: f64, lo: f64, hi: f64) -> f64 {
    let span = hi - lo;
    if span.abs() < 1e-12 {
        0.0
    } else {
        2.0 * (v - lo) / span - 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_panics_on_xs_length_mismatch() {
        let result = std::panic::catch_unwind(|| {
            SurfacePlot::new(vec![1.0, 2.0], vec![1.0, 2.0], vec![1.0], 1, 2);
        });
        assert!(result.is_err(), "should panic when zs.len() != rows * cols");
    }

    #[test]
    fn new_panics_on_ys_length_mismatch() {
        let result = std::panic::catch_unwind(|| {
            SurfacePlot::new(vec![1.0, 2.0], vec![1.0], vec![1.0, 2.0], 1, 2);
        });
        assert!(result.is_err(), "should panic when ys.len() != rows * cols");
    }

    #[test]
    fn normalizes_x_min_to_neg1_and_max_to_pos1() {
        // 1x3 grid: x in [0, 10], y constant, z constant
        let xs = vec![0.0, 5.0, 10.0];
        let ys = vec![0.0, 0.0, 0.0];
        let zs = vec![0.0, 0.0, 0.0];
        let plot = SurfacePlot::new(xs, ys, zs, 1, 3);
        let p_min = plot.world_point(0, 0);
        let p_max = plot.world_point(0, 2);
        assert!((p_min.x - (-1.0)).abs() < 1e-10, "x_min should normalize to -1; got {}", p_min.x);
        assert!((p_max.x - 1.0).abs() < 1e-10, "x_max should normalize to +1; got {}", p_max.x);
    }

    #[test]
    fn normalizes_z_axis_independently() {
        // z in [2, 8] should normalize to [-1, 1]
        let xs = vec![0.0, 1.0];
        let ys = vec![0.0, 0.0];
        let zs = vec![2.0, 8.0];
        let plot = SurfacePlot::new(xs, ys, zs, 1, 2);
        let p0 = plot.world_point(0, 0);
        let p1 = plot.world_point(0, 1);
        assert!((p0.z - (-1.0)).abs() < 1e-10, "z_min should normalize to -1; got {}", p0.z);
        assert!((p1.z - 1.0).abs() < 1e-10, "z_max should normalize to +1; got {}", p1.z);
    }

    #[test]
    fn flat_surface_no_panic_z_maps_to_zero() {
        // All z values equal → degenerate range → should NOT divide by zero; all z map to 0.0
        let xs = vec![0.0, 1.0, 0.0, 1.0];
        let ys = vec![0.0, 0.0, 1.0, 1.0];
        let zs = vec![5.0, 5.0, 5.0, 5.0];
        let plot = SurfacePlot::new(xs, ys, zs, 2, 2);
        for r in 0..2 {
            for c in 0..2 {
                let p = plot.world_point(r, c);
                assert!((p.z - 0.0).abs() < 1e-10,
                    "flat surface z should normalize to 0.0; got {} at ({},{})", p.z, r, c);
            }
        }
    }

    #[test]
    fn world_point_row_major_ordering() {
        // 2x2 grid: xs[r*cols + c] maps to point (r, c)
        // xs = [0, 10, 0, 10], ys = [0, 0, 10, 10], zs all 0
        // After normalization: x in {-1, +1}, y in {-1, +1}
        let xs = vec![0.0, 10.0, 0.0, 10.0];
        let ys = vec![0.0, 0.0, 10.0, 10.0];
        let zs = vec![0.0; 4];
        let plot = SurfacePlot::new(xs, ys, zs, 2, 2);
        let p00 = plot.world_point(0, 0); // xs[0]=0 → x=-1, ys[0]=0 → y=-1
        let p01 = plot.world_point(0, 1); // xs[1]=10 → x=+1
        let p10 = plot.world_point(1, 0); // ys[2]=10 → y=+1
        assert!((p00.x - (-1.0)).abs() < 1e-10);
        assert!((p01.x - 1.0).abs() < 1e-10);
        assert!((p00.y - (-1.0)).abs() < 1e-10);
        assert!((p10.y - 1.0).abs() < 1e-10);
    }

    #[test]
    fn rows_and_cols_accessors() {
        let plot = SurfacePlot::new(vec![0.0; 6], vec![0.0; 6], vec![0.0; 6], 2, 3);
        assert_eq!(plot.rows(), 2);
        assert_eq!(plot.cols(), 3);
    }
}
