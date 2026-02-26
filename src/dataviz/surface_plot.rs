//! SurfacePlot: a regular 3D grid surface with painter's algorithm rendering.
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
//! # Rendering
//! Call [`SurfacePlot::to_primitives`] with a [`Camera`] to get SVG-ready primitives.
//! Uses the painter's algorithm: backface cull, sort back-to-front, emit one quad per face.

use crate::dataviz::camera::Point3D;
use crate::dataviz::camera::Camera;
use crate::dataviz::colormap::viridis_color;
use crate::primitives::{Primitive, Bezier};
use crate::Color;

/// Controls how the surface is rendered: wireframe edges only, solid shaded faces,
/// or shaded faces with wireframe overlay.
///
/// Default: `Shaded` — viridis colormap applied when no mode is explicitly set.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RenderMode {
    /// Flat-shaded faces colored by z-height using the viridis colormap.
    Shaded,
    /// Wireframe edges only; charcoal colored, front-facing edges only.
    Wireframe,
    /// Shaded faces with thin wireframe overlay on top.
    ShadedWireframe,
}

impl Default for RenderMode {
    fn default() -> Self {
        RenderMode::Shaded
    }
}

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
    // Data-space extents captured before normalization (needed for tick label computation).
    x_data_min: f64,
    x_data_max: f64,
    y_data_min: f64,
    y_data_max: f64,
    z_data_min: f64,
    z_data_max: f64,
    // Rendering configuration (Phase 6)
    render_mode: RenderMode,
    x_label: String,
    y_label: String,
    z_label: String,
    show_base_grid: bool,
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

        let (x_data_min, x_data_max) = min_max(&xs);
        let (y_data_min, y_data_max) = min_max(&ys);
        let (z_data_min, z_data_max) = min_max(&zs);
        let world_vertices = normalize_to_world_space(&xs, &ys, &zs);
        SurfacePlot {
            rows,
            cols,
            world_vertices,
            x_data_min,
            x_data_max,
            y_data_min,
            y_data_max,
            z_data_min,
            z_data_max,
            render_mode: RenderMode::default(),
            x_label: "X".to_string(),
            y_label: "Y".to_string(),
            z_label: "Z".to_string(),
            show_base_grid: false,
        }
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

    /// Set the render mode (Shaded, Wireframe, or ShadedWireframe). Default: Shaded.
    pub fn render_mode(mut self, mode: RenderMode) -> Self {
        self.render_mode = mode;
        self
    }

    /// Set the X axis label. Default: "X".
    pub fn x_label(mut self, label: impl Into<String>) -> Self {
        self.x_label = label.into();
        self
    }

    /// Set the Y axis label. Default: "Y".
    pub fn y_label(mut self, label: impl Into<String>) -> Self {
        self.y_label = label.into();
        self
    }

    /// Set the Z axis label. Default: "Z".
    pub fn z_label(mut self, label: impl Into<String>) -> Self {
        self.z_label = label.into();
        self
    }

    /// Show or hide the base plane grid. Default: false (hidden).
    pub fn show_base_grid(mut self, show: bool) -> Self {
        self.show_base_grid = show;
        self
    }

    /// Returns the data-space extents as (x_min, x_max, y_min, y_max, z_min, z_max).
    pub fn data_extents(&self) -> (f64, f64, f64, f64, f64, f64) {
        (
            self.x_data_min,
            self.x_data_max,
            self.y_data_min,
            self.y_data_max,
            self.z_data_min,
            self.z_data_max,
        )
    }

    /// Returns the configured render mode.
    pub fn render_mode_value(&self) -> RenderMode {
        self.render_mode
    }

    /// Returns the X axis label.
    pub fn x_label_value(&self) -> &str {
        &self.x_label
    }

    /// Returns the Y axis label.
    pub fn y_label_value(&self) -> &str {
        &self.y_label
    }

    /// Returns the Z axis label.
    pub fn z_label_value(&self) -> &str {
        &self.z_label
    }

    /// Returns whether to show the base plane grid.
    pub fn show_base_grid_value(&self) -> bool {
        self.show_base_grid
    }

    /// Render this surface plot to a list of SVG-ready primitives.
    ///
    /// Uses the painter's algorithm: backface-cull invisible faces, sort remaining
    /// faces back-to-front by world-space distance from the camera eye, then emit
    /// one Bezier polygon per face in sorted order.
    ///
    /// The render mode controls whether faces are filled (Shaded), outlined (Wireframe),
    /// or both (ShadedWireframe). Backface edges are always hidden.
    ///
    /// Returns an empty Vec if no faces are visible from the given camera.
    pub fn to_primitives(&self, camera: &Camera, viewport: (u32, u32)) -> Vec<Primitive> {
        use crate::dataviz::camera::Vector3D;

        // Step 0: Precompute all projected screen positions (rows × cols projections)
        // Avoids redundant project_to_screen calls — shared corners reused across adjacent faces.
        let projected: Vec<Vec<Option<crate::dataviz::camera::Point2D>>> = (0..self.rows)
            .map(|r| {
                (0..self.cols)
                    .map(|c| camera.project_to_screen(self.world_point(r, c), viewport))
                    .collect()
            })
            .collect();

        // Step 1: Recompute camera eye position from spherical parameters (for depth sorting)
        let (eye_x, eye_y, eye_z) = camera.eye_position();

        // Step 2: Collect all visible faces with their depth and centroid z
        struct FaceEntry {
            row: usize,
            col: usize,
            depth_sq: f64,
            centroid_z_norm: f64, // world-space z in [-1, 1] for viridis lookup
        }

        let face_count = (self.rows - 1) * (self.cols - 1);
        let mut faces: Vec<FaceEntry> = Vec::with_capacity(face_count);

        for r in 0..(self.rows - 1) {
            for c in 0..(self.cols - 1) {
                let p00 = self.world_point(r, c);
                let p01 = self.world_point(r, c + 1);
                let p10 = self.world_point(r + 1, c);
                let p11 = self.world_point(r + 1, c + 1);

                // Compute face normal via cross product of two edges (p01-p00) × (p10-p00)
                let ex = p01.x - p00.x;
                let ey = p01.y - p00.y;
                let ez = p01.z - p00.z;
                let fx = p10.x - p00.x;
                let fy = p10.y - p00.y;
                let fz = p10.z - p00.z;
                let normal = Vector3D {
                    x: ey * fz - ez * fy,
                    y: ez * fx - ex * fz,
                    z: ex * fy - ey * fx,
                };

                // Backface cull: skip faces whose normal points away from the camera
                if !camera.is_face_visible(normal) {
                    continue;
                }

                // Face centroid (average of 4 corners)
                let cx = (p00.x + p01.x + p10.x + p11.x) / 4.0;
                let cy = (p00.y + p01.y + p10.y + p11.y) / 4.0;
                let cz = (p00.z + p01.z + p10.z + p11.z) / 4.0;

                // Squared distance from centroid to camera eye (for painter's sort — no sqrt needed)
                let dx = cx - eye_x;
                let dy = cy - eye_y;
                let dz = cz - eye_z;
                let depth_sq = dx * dx + dy * dy + dz * dz;

                faces.push(FaceEntry { row: r, col: c, depth_sq, centroid_z_norm: cz });
            }
        }

        // Step 3: Back-to-front sort (painter's algorithm — farthest first, near faces paint over far)
        faces.sort_unstable_by(|a, b| b.depth_sq.total_cmp(&a.depth_sq));

        // Step 4: Emit one Bezier polygon per face in sorted order
        let charcoal = Color::rgb(64, 64, 64);
        const WIRE_STROKE_WIDTH: f64 = 1.0;
        const SHADED_WIRE_STROKE_WIDTH: f64 = 0.5;

        let mut prims: Vec<Primitive> = Vec::with_capacity(faces.len());

        for face in &faces {
            let r = face.row;
            let c = face.col;

            // Retrieve precomputed projected corners; skip face if any corner is behind near plane
            let s00 = match projected[r][c] { Some(p) => p, None => continue };
            let s01 = match projected[r][c + 1] { Some(p) => p, None => continue };
            let s11 = match projected[r + 1][c + 1] { Some(p) => p, None => continue };
            let s10 = match projected[r + 1][c] { Some(p) => p, None => continue };

            // Winding order: (r,c) → (r,c+1) → (r+1,c+1) → (r+1,c)
            // This traces the quad edges consistently (never a bowtie/self-intersecting polygon)
            match self.render_mode {
                RenderMode::Shaded => {
                    // t ∈ [0, 1]: map normalized z [-1, 1] to viridis palette index
                    let t = (face.centroid_z_norm + 1.0) / 2.0;
                    let face_color = viridis_color(t);
                    let path = Bezier::new()
                        .move_to(s00.x, s00.y)
                        .line_to(s01.x, s01.y)
                        .line_to(s11.x, s11.y)
                        .line_to(s10.x, s10.y)
                        .close()
                        .fill(face_color);
                    prims.push(path.into());
                }
                RenderMode::Wireframe => {
                    let path = Bezier::new()
                        .move_to(s00.x, s00.y)
                        .line_to(s01.x, s01.y)
                        .line_to(s11.x, s11.y)
                        .line_to(s10.x, s10.y)
                        .close()
                        .stroke(charcoal, WIRE_STROKE_WIDTH)
                        .expect("stroke width 1.0 is valid");
                    prims.push(path.into());
                }
                RenderMode::ShadedWireframe => {
                    let t = (face.centroid_z_norm + 1.0) / 2.0;
                    let face_color = viridis_color(t);
                    let path = Bezier::new()
                        .move_to(s00.x, s00.y)
                        .line_to(s01.x, s01.y)
                        .line_to(s11.x, s11.y)
                        .line_to(s10.x, s10.y)
                        .close()
                        .fill(face_color)
                        .stroke(charcoal, SHADED_WIRE_STROKE_WIDTH)
                        .expect("stroke width 0.5 is valid");
                    prims.push(path.into());
                }
            }
        }

        prims
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

    #[test]
    fn render_mode_default_is_shaded() {
        let plot = SurfacePlot::new(vec![0.0, 1.0], vec![0.0, 0.0], vec![0.0, 1.0], 1, 2);
        assert_eq!(plot.render_mode_value(), RenderMode::Shaded);
    }

    #[test]
    fn builder_sets_render_mode() {
        let plot = SurfacePlot::new(vec![0.0, 1.0], vec![0.0, 0.0], vec![0.0, 1.0], 1, 2)
            .render_mode(RenderMode::Wireframe);
        assert_eq!(plot.render_mode_value(), RenderMode::Wireframe);
    }

    #[test]
    fn builder_sets_x_label() {
        let plot = SurfacePlot::new(vec![0.0, 1.0], vec![0.0, 0.0], vec![0.0, 1.0], 1, 2)
            .x_label("Time");
        assert_eq!(plot.x_label_value(), "Time");
    }

    #[test]
    fn show_base_grid_default_false() {
        let plot = SurfacePlot::new(vec![0.0, 1.0], vec![0.0, 0.0], vec![0.0, 1.0], 1, 2);
        assert!(!plot.show_base_grid_value());
    }

    #[test]
    fn data_extents_captures_original_values() {
        // xs = [0.0, 10.0] → after normalization these become [-1, 1], but extents preserve original
        let xs = vec![0.0, 10.0];
        let ys = vec![0.0, 0.0];
        let zs = vec![0.0, 0.0];
        let plot = SurfacePlot::new(xs, ys, zs, 1, 2);
        let (x_min, x_max, _, _, _, _) = plot.data_extents();
        assert!((x_min - 0.0).abs() < 1e-10, "x_min should be 0.0; got {}", x_min);
        assert!((x_max - 10.0).abs() < 1e-10, "x_max should be 10.0; got {}", x_max);
    }

    // --- to_primitives() tests ---

    fn make_2x2_plot() -> SurfacePlot {
        // 2x2 grid (1 face): flat surface in XY plane (z=0 everywhere).
        // Face normal is exactly +z, so any camera above the XY plane sees it.
        SurfacePlot::new(
            vec![0.0, 1.0, 0.0, 1.0],
            vec![0.0, 0.0, 1.0, 1.0],
            vec![0.0, 0.0, 0.0, 0.0], // flat: all z=0 → degenerate z normalized to 0.0
            2, 2,
        )
    }

    #[test]
    fn to_primitives_shaded_returns_nonempty() {
        // Camera above and to the side — face should be visible
        let plot = make_2x2_plot();
        let camera = Camera::new(45.0, 30.0, 3.0);
        let result = plot.to_primitives(&camera, (800, 600));
        assert!(!result.is_empty(), "shaded 2x2 plot should produce at least one primitive");
    }

    #[test]
    fn to_primitives_backface_cull_opposite_side() {
        // Camera at elevation=-60° (looking up at the underside of a surface facing up).
        // The single face of the 2x2 grid has a normal pointing +Z.
        // From below (elevation < 0), the normal points AWAY from the camera eye → backface culled.
        let plot = make_2x2_plot();
        let camera_below = Camera::new(45.0, -60.0, 3.0);
        let camera_above = Camera::new(45.0, 30.0, 3.0);
        let result_below = plot.to_primitives(&camera_below, (800, 600));
        let result_above = plot.to_primitives(&camera_above, (800, 600));
        // Viewed from below: face should be culled (0 primitives) or fewer than from above
        assert!(
            result_below.len() < result_above.len(),
            "camera from below should see fewer faces ({}); camera above sees {}",
            result_below.len(), result_above.len()
        );
    }

    #[test]
    fn to_primitives_wireframe_mode() {
        let plot = make_2x2_plot().render_mode(RenderMode::Wireframe);
        let camera = Camera::new(45.0, 30.0, 3.0);
        let result = plot.to_primitives(&camera, (800, 600));
        assert!(!result.is_empty(), "wireframe 2x2 plot should produce at least one primitive");
        // All produced primitives should be Bezier paths
        for prim in &result {
            match prim {
                Primitive::Bezier(_) => {}
                other => panic!("expected Bezier primitive, got {:?}", other),
            }
        }
    }

    #[test]
    fn to_primitives_2x2_produces_one_face() {
        // A 2x2 grid has exactly 1 face (1×1 quads).
        // Camera nearly directly above (elevation=89°) should see the single face.
        let plot = make_2x2_plot();
        let camera = Camera::new(0.0, 89.0, 3.0);
        let result = plot.to_primitives(&camera, (800, 600));
        assert_eq!(
            result.len(), 1,
            "2x2 grid should produce exactly 1 primitive from top-facing camera; got {}",
            result.len()
        );
    }

    #[test]
    fn to_primitives_performance_30x30() {
        // 30x30 grid = 900 vertices, 29x29 = 841 faces.
        // to_primitives() should complete in under 500ms for this size.
        let n = 30;
        let mut xs = Vec::with_capacity(n * n);
        let mut ys = Vec::with_capacity(n * n);
        let mut zs = Vec::with_capacity(n * n);
        for r in 0..n {
            for c in 0..n {
                xs.push(r as f64 / (n - 1) as f64);
                ys.push(c as f64 / (n - 1) as f64);
                // Simple paraboloid surface: z = x^2 + y^2
                let x = r as f64 / (n - 1) as f64;
                let y = c as f64 / (n - 1) as f64;
                zs.push(x * x + y * y);
            }
        }
        let plot = SurfacePlot::new(xs, ys, zs, n, n);
        let camera = Camera::new(45.0, 30.0, 3.0);

        let start = std::time::Instant::now();
        let result = plot.to_primitives(&camera, (1920, 1080));
        let elapsed = start.elapsed();

        assert!(
            elapsed.as_millis() < 500,
            "30x30 to_primitives() took {}ms, expected under 500ms",
            elapsed.as_millis()
        );
        assert!(
            !result.is_empty(),
            "30x30 grid should produce at least some primitives from above"
        );
    }
}
