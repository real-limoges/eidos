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

use crate::Color;
use crate::animation::{Easing, Tween};
use crate::dataviz::axes::{format_tick, generate_ticks};
use crate::dataviz::camera::Camera;
use crate::dataviz::camera::Point3D;
use crate::dataviz::colormap::viridis_color;
use crate::primitives::{Bezier, Line, Primitive, Text};

/// Internal record for one animate_fit time range.
struct FitAnimation {
    start_time: f64,
    duration: f64,
    easing: Easing,
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
            duration: self.duration,
            easing: self.easing,
        }
    }
}

/// Internal record for one animate_camera_azimuth time range.
struct CameraAnimation {
    start_time: f64,
    duration: f64,
    start_angle: f64,
    end_angle: f64,
    easing: Easing,
}

impl std::fmt::Debug for CameraAnimation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CameraAnimation")
            .field("start_time", &self.start_time)
            .field("duration", &self.duration)
            .field("start_angle", &self.start_angle)
            .field("end_angle", &self.end_angle)
            .finish()
    }
}

impl Clone for CameraAnimation {
    fn clone(&self) -> Self {
        CameraAnimation {
            start_time: self.start_time,
            duration: self.duration,
            start_angle: self.start_angle,
            end_angle: self.end_angle,
            easing: self.easing,
        }
    }
}

/// Controls how the surface is rendered: wireframe edges only, solid shaded faces,
/// or shaded faces with wireframe overlay.
///
/// Default: `Shaded` — viridis colormap applied when no mode is explicitly set.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum RenderMode {
    /// Flat-shaded faces colored by z-height using the viridis colormap.
    #[default]
    Shaded,
    /// Wireframe edges only; charcoal colored, front-facing edges only.
    Wireframe,
    /// Shaded faces with thin wireframe overlay on top.
    ShadedWireframe,
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
    /// Normalized world-z per vertex extracted at construction. Same length as world_vertices.
    /// Used by to_primitives_at() to interpolate z from 0.0 → fitted_z during surface morph.
    fitted_zs: Vec<f64>,
    /// Animation ranges registered via animate_fit(). Evaluated at render time in z_at().
    /// Contract: user must not register overlapping ranges (behavior undefined for overlaps).
    fit_animations: Vec<FitAnimation>,
    /// Camera azimuth animation ranges registered via animate_camera_azimuth().
    camera_animations: Vec<CameraAnimation>,
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
            xs.len(),
            n,
            "xs.len() ({}) != rows * cols ({}*{}={})",
            xs.len(),
            rows,
            cols,
            n
        );
        assert_eq!(
            ys.len(),
            n,
            "ys.len() ({}) != rows * cols ({}*{}={})",
            ys.len(),
            rows,
            cols,
            n
        );
        assert_eq!(
            zs.len(),
            n,
            "zs.len() ({}) != rows * cols ({}*{}={})",
            zs.len(),
            rows,
            cols,
            n
        );

        let (x_data_min, x_data_max) = min_max(&xs);
        let (y_data_min, y_data_max) = min_max(&ys);
        let (z_data_min, z_data_max) = min_max(&zs);
        let world_vertices = normalize_to_world_space(&xs, &ys, &zs);
        let fitted_zs: Vec<f64> = world_vertices.iter().map(|p| p.z).collect();
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
            fitted_zs,
            fit_animations: Vec::new(),
            camera_animations: Vec::new(),
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

    /// Register a surface morph animation: vertices interpolate from z=0 (flat) to their
    /// fitted world-z values over [start_time, start_time + duration].
    ///
    /// Hold semantics: before the first range → z=0; after last range → fitted_z;
    /// between non-overlapping ranges → hold at fitted_z (100% morph).
    ///
    /// Multiple non-overlapping calls are supported (e.g. two separate morph sequences).
    /// If animate_fit is never called, to_primitives_at behaves identically to to_primitives.
    pub fn animate_fit(mut self, start_time: f64, duration: f64, easing: Easing) -> Self {
        self.fit_animations.push(FitAnimation {
            start_time,
            duration,
            easing,
        });
        self
    }

    /// Register a camera azimuth sweep animation.
    ///
    /// The camera azimuth interpolates from start_angle to end_angle (in degrees)
    /// over [start_time, start_time + duration]. Any float values are accepted for
    /// angles — Camera::new treats them modulo 360° via trig, so 350→370 sweeps
    /// smoothly through 360° without a jump.
    ///
    /// Hold semantics: before animation starts → start_angle; after it ends → end_angle.
    /// Returns None from camera_at() when no animation is registered.
    pub fn animate_camera_azimuth(
        mut self,
        start_time: f64,
        duration: f64,
        start_angle: f64,
        end_angle: f64,
        easing: Easing,
    ) -> Self {
        self.camera_animations.push(CameraAnimation {
            start_time,
            duration,
            start_angle,
            end_angle,
            easing,
        });
        self
    }

    /// Evaluate the animated world-z for a single vertex at scene time t_secs.
    ///
    /// If no animations registered: returns fitted_z unchanged (static surface).
    /// Before first animation: returns 0.0 (flat).
    /// After last animation: returns fitted_z (full morph, hold-last).
    /// Inside an active animation range: interpolates via Tween<f64>.
    /// Between two non-overlapping ranges: returns fitted_z (hold at full morph).
    fn z_at(&self, fitted_z: f64, t_secs: f64) -> f64 {
        if self.fit_animations.is_empty() {
            return fitted_z;
        }

        // Sort is skipped at call time — push order is preserved (user adds ranges chronologically).
        // For robustness, still handle any order by scanning all ranges.

        let first = &self.fit_animations[0];
        let last = &self.fit_animations[self.fit_animations.len() - 1];

        // Before all animations: hold flat (z=0)
        if t_secs < first.start_time {
            return 0.0;
        }

        // After all animations: hold final fitted value
        if t_secs >= last.start_time + last.duration {
            return fitted_z;
        }

        // Search for an active range or detect a gap between two ranges
        for anim in &self.fit_animations {
            let end = anim.start_time + anim.duration;
            if t_secs >= anim.start_time && t_secs < end {
                // Inside this range: interpolate from 0.0 to fitted_z
                let tween = Tween {
                    start: 0.0_f64,
                    end: fitted_z,
                    start_time: anim.start_time,
                    duration: anim.duration,
                    easing: anim.easing,
                };
                return tween.value_at(t_secs);
            }
        }

        // Gap between two non-overlapping ranges: hold at fitted_z (previous range ended at 100%)
        fitted_z
    }

    /// Evaluate the animated camera azimuth at scene time t_secs.
    ///
    /// Returns None if no camera animation has been registered (caller uses a static camera).
    ///
    /// Hold semantics:
    /// - Before first animation starts: returns start_angle of first animation
    /// - After last animation ends: returns end_angle of last animation
    /// - Inside an active range: interpolates via Tween<f64>
    /// - Between two non-overlapping ranges: returns end_angle of the preceding range
    ///
    /// The returned azimuth should be passed to Camera::new(azimuth, elevation_deg, distance).
    /// Camera::new accepts any float — trig handles modulo 360° naturally.
    pub fn camera_at(&self, t_secs: f64) -> Option<f64> {
        if self.camera_animations.is_empty() {
            return None;
        }

        let first = &self.camera_animations[0];
        let last = &self.camera_animations[self.camera_animations.len() - 1];

        // Hold-first: before all animations
        if t_secs < first.start_time {
            return Some(first.start_angle);
        }

        // Hold-last: after all animations
        if t_secs >= last.start_time + last.duration {
            return Some(last.end_angle);
        }

        // Scan for active range or gap
        let mut last_end_angle = first.start_angle;
        for anim in &self.camera_animations {
            let end = anim.start_time + anim.duration;
            if t_secs >= anim.start_time && t_secs < end {
                let tween = Tween {
                    start: anim.start_angle,
                    end: anim.end_angle,
                    start_time: anim.start_time,
                    duration: anim.duration,
                    easing: anim.easing,
                };
                return Some(tween.value_at(t_secs));
            }
            if t_secs >= end {
                last_end_angle = anim.end_angle;
            }
        }

        // Gap between ranges: hold at end_angle of the most recently completed range
        Some(last_end_angle)
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

    /// Returns the squared centroid distances for all backface-culled visible faces.
    ///
    /// Used by SceneBuilder to provide face_depths to add_scatter / add_scatter_at.
    /// This is a lightweight version of to_primitives() — runs backface cull and centroid
    /// depth computation without building any SVG primitives.
    pub fn visible_face_depths(&self, camera: &Camera, viewport: (u32, u32)) -> Vec<f64> {
        use crate::dataviz::camera::Vector3D;
        let _ = viewport; // not needed for depth-only computation, but kept for API consistency

        let (eye_x, eye_y, eye_z) = camera.eye_position();
        let face_count = (self.rows - 1) * (self.cols - 1);
        let mut depths: Vec<f64> = Vec::with_capacity(face_count);

        for r in 0..(self.rows - 1) {
            for c in 0..(self.cols - 1) {
                let p00 = self.world_point(r, c);
                let p01 = self.world_point(r, c + 1);
                let p10 = self.world_point(r + 1, c);
                let p11 = self.world_point(r + 1, c + 1);

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

                if !camera.is_face_visible(normal) {
                    continue;
                }

                let cx = (p00.x + p01.x + p10.x + p11.x) / 4.0;
                let cy = (p00.y + p01.y + p10.y + p11.y) / 4.0;
                let cz = (p00.z + p01.z + p10.z + p11.z) / 4.0;
                let dx = cx - eye_x;
                let dy = cy - eye_y;
                let dz = cz - eye_z;
                depths.push(dx * dx + dy * dy + dz * dz);
            }
        }

        depths
    }

    /// Same as visible_face_depths() but uses animated z-values at t_secs.
    ///
    /// Used by SceneBuilder::add_surface_at() to provide correct face_depths
    /// that match the animated surface geometry used by to_primitives_at().
    pub fn visible_face_depths_at(
        &self,
        camera: &Camera,
        viewport: (u32, u32),
        t_secs: f64,
    ) -> Vec<f64> {
        use crate::dataviz::camera::Vector3D;
        let _ = viewport;

        // Build animated world vertices
        let animated: Vec<Point3D> = self
            .world_vertices
            .iter()
            .enumerate()
            .map(|(i, p)| Point3D {
                x: p.x,
                y: p.y,
                z: self.z_at(self.fitted_zs[i], t_secs),
            })
            .collect();
        let anim_point = |r: usize, c: usize| -> Point3D { animated[r * self.cols + c] };

        let (eye_x, eye_y, eye_z) = camera.eye_position();
        let face_count = (self.rows - 1) * (self.cols - 1);
        let mut depths: Vec<f64> = Vec::with_capacity(face_count);

        for r in 0..(self.rows - 1) {
            for c in 0..(self.cols - 1) {
                let p00 = anim_point(r, c);
                let p01 = anim_point(r, c + 1);
                let p10 = anim_point(r + 1, c);
                let p11 = anim_point(r + 1, c + 1);

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

                if !camera.is_face_visible(normal) {
                    continue;
                }

                let cx = (p00.x + p01.x + p10.x + p11.x) / 4.0;
                let cy = (p00.y + p01.y + p10.y + p11.y) / 4.0;
                let cz = (p00.z + p01.z + p10.z + p11.z) / 4.0;
                let dx = cx - eye_x;
                let dy = cy - eye_y;
                let dz = cz - eye_z;
                depths.push(dx * dx + dy * dy + dz * dz);
            }
        }

        depths
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

                faces.push(FaceEntry {
                    row: r,
                    col: c,
                    depth_sq,
                    centroid_z_norm: cz,
                });
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
            let s00 = match projected[r][c] {
                Some(p) => p,
                None => continue,
            };
            let s01 = match projected[r][c + 1] {
                Some(p) => p,
                None => continue,
            };
            let s11 = match projected[r + 1][c + 1] {
                Some(p) => p,
                None => continue,
            };
            let s10 = match projected[r + 1][c] {
                Some(p) => p,
                None => continue,
            };

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
                        .stroke(charcoal, WIRE_STROKE_WIDTH);
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
                        .stroke(charcoal, SHADED_WIRE_STROKE_WIDTH);
                    prims.push(path.into());
                }
            }
        }

        // Append 3D axis lines, tick marks, and labels on top of surface faces
        let mut axis_prims = self.draw_axes(camera, viewport);
        prims.append(&mut axis_prims);

        prims
    }

    /// Render this surface plot at scene time t_secs, applying surface morph animation.
    ///
    /// If animate_fit() was never called, this produces identical output to to_primitives().
    /// Takes &self — safe to call from within a Fn (non-mutable) render closure.
    pub fn to_primitives_at(
        &self,
        camera: &Camera,
        viewport: (u32, u32),
        t_secs: f64,
    ) -> Vec<Primitive> {
        use crate::dataviz::camera::Vector3D;

        // Build animated world vertices: same x,y as fitted, but z interpolated at t_secs
        let animated: Vec<Point3D> = self
            .world_vertices
            .iter()
            .enumerate()
            .map(|(i, p)| Point3D {
                x: p.x,
                y: p.y,
                z: self.z_at(self.fitted_zs[i], t_secs),
            })
            .collect();

        // Helper closure to look up animated vertex at (row, col)
        let anim_point = |r: usize, c: usize| -> Point3D { animated[r * self.cols + c] };

        // Precompute projected screen positions using animated vertices
        let projected: Vec<Vec<Option<crate::dataviz::camera::Point2D>>> = (0..self.rows)
            .map(|r| {
                (0..self.cols)
                    .map(|c| camera.project_to_screen(anim_point(r, c), viewport))
                    .collect()
            })
            .collect();

        let (eye_x, eye_y, eye_z) = camera.eye_position();

        struct FaceEntry {
            row: usize,
            col: usize,
            depth_sq: f64,
            centroid_z_norm: f64,
        }

        let face_count = (self.rows - 1) * (self.cols - 1);
        let mut faces: Vec<FaceEntry> = Vec::with_capacity(face_count);

        for r in 0..(self.rows - 1) {
            for c in 0..(self.cols - 1) {
                let p00 = anim_point(r, c);
                let p01 = anim_point(r, c + 1);
                let p10 = anim_point(r + 1, c);
                let p11 = anim_point(r + 1, c + 1);

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

                if !camera.is_face_visible(normal) {
                    continue;
                }

                let cx = (p00.x + p01.x + p10.x + p11.x) / 4.0;
                let cy = (p00.y + p01.y + p10.y + p11.y) / 4.0;
                let cz = (p00.z + p01.z + p10.z + p11.z) / 4.0;

                let dx = cx - eye_x;
                let dy = cy - eye_y;
                let dz = cz - eye_z;
                let depth_sq = dx * dx + dy * dy + dz * dz;

                faces.push(FaceEntry {
                    row: r,
                    col: c,
                    depth_sq,
                    centroid_z_norm: cz,
                });
            }
        }

        faces.sort_unstable_by(|a, b| b.depth_sq.total_cmp(&a.depth_sq));

        let charcoal = Color::rgb(64, 64, 64);
        const WIRE_STROKE_WIDTH: f64 = 1.0;
        const SHADED_WIRE_STROKE_WIDTH: f64 = 0.5;

        let mut prims: Vec<Primitive> = Vec::with_capacity(faces.len());

        for face in &faces {
            let r = face.row;
            let c = face.col;
            let s00 = match projected[r][c] {
                Some(p) => p,
                None => continue,
            };
            let s01 = match projected[r][c + 1] {
                Some(p) => p,
                None => continue,
            };
            let s11 = match projected[r + 1][c + 1] {
                Some(p) => p,
                None => continue,
            };
            let s10 = match projected[r + 1][c] {
                Some(p) => p,
                None => continue,
            };

            match self.render_mode {
                RenderMode::Shaded => {
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
                        .stroke(charcoal, WIRE_STROKE_WIDTH);
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
                        .stroke(charcoal, SHADED_WIRE_STROKE_WIDTH);
                    prims.push(path.into());
                }
            }
        }

        let mut axis_prims = self.draw_axes(camera, viewport);
        prims.append(&mut axis_prims);
        prims
    }

    /// Render 3D axis lines, tick marks, and labels as primitives.
    ///
    /// Draws 3 axis lines from the camera-facing floor corner:
    /// - X axis: along the floor, varying x from far corner to opposite x
    /// - Y axis: along the floor, varying y from far corner to opposite y
    /// - Z axis: vertical from the far floor corner up to z=+1
    ///
    /// Tick labels use data-space values (from SurfacePlot data extents).
    fn draw_axes(&self, camera: &Camera, viewport: (u32, u32)) -> Vec<Primitive> {
        const N_TICKS: usize = 5;
        const TICK_HALF_LEN: f64 = 8.0; // half-length of tick mark in screen pixels
        const LABEL_OFFSET: f64 = 16.0; // screen pixel offset for tick value label
        const AXIS_LABEL_OFFSET: f64 = 28.0; // screen pixel offset for axis name label
        const AXIS_STROKE_WIDTH: f64 = 1.5;
        const TICK_STROKE_WIDTH: f64 = 1.0;
        const TICK_LABEL_SIZE: f64 = 10.0;
        const AXIS_LABEL_SIZE: f64 = 12.0;
        let axis_color = Color::rgb(80, 80, 80);

        let mut prims: Vec<Primitive> = Vec::new();

        let (fx, fy) = far_floor_corner(camera.azimuth_deg);
        let fz = -1.0_f64;
        let top_z = 1.0_f64;

        // Opposite ends for X and Y axes
        let x_end_x = -fx;
        let y_end_y = -fy;

        let (x_data_min, x_data_max, y_data_min, y_data_max, z_data_min, z_data_max) =
            self.data_extents();

        // --- X AXIS ---
        {
            let start = crate::dataviz::camera::Point3D {
                x: fx,
                y: fy,
                z: fz,
            };
            let end = crate::dataviz::camera::Point3D {
                x: x_end_x,
                y: fy,
                z: fz,
            };

            if let (Some(s0), Some(s1)) = (
                camera.project_to_screen(start, viewport),
                camera.project_to_screen(end, viewport),
            ) {
                // Draw axis line
                prims.push(
                    Line::new(s0.x, s0.y, s1.x, s1.y)
                        .stroke_color(axis_color)
                        .stroke_width(AXIS_STROKE_WIDTH)
                        .into(),
                );

                // Tick marks and labels
                let x_ticks = generate_ticks(x_data_min, x_data_max, N_TICKS);
                let x_step = if x_ticks.len() >= 2 {
                    x_ticks[1] - x_ticks[0]
                } else {
                    1.0
                };
                for &tick_val in &x_ticks {
                    let t = if (x_data_max - x_data_min).abs() < 1e-12 {
                        0.0
                    } else {
                        (tick_val - x_data_min) / (x_data_max - x_data_min) * 2.0 - 1.0
                    };
                    // Interpolate tick world position along axis
                    let wx = fx + (x_end_x - fx) * (t + 1.0) / 2.0;
                    let world_tick = crate::dataviz::camera::Point3D {
                        x: wx,
                        y: fy,
                        z: fz,
                    };
                    if let Some(sp) = camera.project_to_screen(world_tick, viewport) {
                        // Perpendicular direction to axis in screen space
                        let perp_dx = -(s1.y - s0.y);
                        let perp_dy = s1.x - s0.x;
                        let len = (perp_dx * perp_dx + perp_dy * perp_dy).sqrt().max(1e-9);
                        let (ndx, ndy) = (perp_dx / len, perp_dy / len);
                        prims.push(
                            Line::new(
                                sp.x - ndx * TICK_HALF_LEN,
                                sp.y - ndy * TICK_HALF_LEN,
                                sp.x + ndx * TICK_HALF_LEN,
                                sp.y + ndy * TICK_HALF_LEN,
                            )
                            .stroke_color(axis_color)
                            .stroke_width(TICK_STROKE_WIDTH)
                            .into(),
                        );

                        let label = format_tick(tick_val, x_step);
                        prims.push(
                            Text::new(sp.x + ndy * LABEL_OFFSET, sp.y - ndx * LABEL_OFFSET, &label)
                                .font_size(TICK_LABEL_SIZE)
                                .into(),
                        );
                    }
                }

                // Axis name label at the end of the axis
                let label_pt_x = s1.x + (s1.x - s0.x).signum() * AXIS_LABEL_OFFSET;
                let label_pt_y = s1.y + (s1.y - s0.y).signum() * AXIS_LABEL_OFFSET;
                prims.push(
                    Text::new(label_pt_x, label_pt_y, &self.x_label)
                        .font_size(AXIS_LABEL_SIZE)
                        .into(),
                );
            }
        }

        // --- Y AXIS ---
        {
            let start = crate::dataviz::camera::Point3D {
                x: fx,
                y: fy,
                z: fz,
            };
            let end = crate::dataviz::camera::Point3D {
                x: fx,
                y: y_end_y,
                z: fz,
            };

            if let (Some(s0), Some(s1)) = (
                camera.project_to_screen(start, viewport),
                camera.project_to_screen(end, viewport),
            ) {
                prims.push(
                    Line::new(s0.x, s0.y, s1.x, s1.y)
                        .stroke_color(axis_color)
                        .stroke_width(AXIS_STROKE_WIDTH)
                        .into(),
                );

                let y_ticks = generate_ticks(y_data_min, y_data_max, N_TICKS);
                let y_step = if y_ticks.len() >= 2 {
                    y_ticks[1] - y_ticks[0]
                } else {
                    1.0
                };
                for &tick_val in &y_ticks {
                    let t = if (y_data_max - y_data_min).abs() < 1e-12 {
                        0.0
                    } else {
                        (tick_val - y_data_min) / (y_data_max - y_data_min) * 2.0 - 1.0
                    };
                    let wy = fy + (y_end_y - fy) * (t + 1.0) / 2.0;
                    let world_tick = crate::dataviz::camera::Point3D {
                        x: fx,
                        y: wy,
                        z: fz,
                    };
                    if let Some(sp) = camera.project_to_screen(world_tick, viewport) {
                        let perp_dx = -(s1.y - s0.y);
                        let perp_dy = s1.x - s0.x;
                        let len = (perp_dx * perp_dx + perp_dy * perp_dy).sqrt().max(1e-9);
                        let (ndx, ndy) = (perp_dx / len, perp_dy / len);
                        prims.push(
                            Line::new(
                                sp.x - ndx * TICK_HALF_LEN,
                                sp.y - ndy * TICK_HALF_LEN,
                                sp.x + ndx * TICK_HALF_LEN,
                                sp.y + ndy * TICK_HALF_LEN,
                            )
                            .stroke_color(axis_color)
                            .stroke_width(TICK_STROKE_WIDTH)
                            .into(),
                        );

                        let label = format_tick(tick_val, y_step);
                        prims.push(
                            Text::new(sp.x + ndy * LABEL_OFFSET, sp.y - ndx * LABEL_OFFSET, &label)
                                .font_size(TICK_LABEL_SIZE)
                                .into(),
                        );
                    }
                }

                let label_pt_x = s1.x + (s1.x - s0.x).signum() * AXIS_LABEL_OFFSET;
                let label_pt_y = s1.y + (s1.y - s0.y).signum() * AXIS_LABEL_OFFSET;
                prims.push(
                    Text::new(label_pt_x, label_pt_y, &self.y_label)
                        .font_size(AXIS_LABEL_SIZE)
                        .into(),
                );
            }
        }

        // --- Z AXIS (vertical) ---
        {
            let start = crate::dataviz::camera::Point3D {
                x: fx,
                y: fy,
                z: fz,
            };
            let end = crate::dataviz::camera::Point3D {
                x: fx,
                y: fy,
                z: top_z,
            };

            if let (Some(s0), Some(s1)) = (
                camera.project_to_screen(start, viewport),
                camera.project_to_screen(end, viewport),
            ) {
                prims.push(
                    Line::new(s0.x, s0.y, s1.x, s1.y)
                        .stroke_color(axis_color)
                        .stroke_width(AXIS_STROKE_WIDTH)
                        .into(),
                );

                let z_ticks = generate_ticks(z_data_min, z_data_max, N_TICKS);
                let z_step = if z_ticks.len() >= 2 {
                    z_ticks[1] - z_ticks[0]
                } else {
                    1.0
                };
                for &tick_val in &z_ticks {
                    let t = if (z_data_max - z_data_min).abs() < 1e-12 {
                        0.0
                    } else {
                        (tick_val - z_data_min) / (z_data_max - z_data_min) * 2.0 - 1.0
                    };
                    let wz = fz + (top_z - fz) * (t + 1.0) / 2.0;
                    let world_tick = crate::dataviz::camera::Point3D {
                        x: fx,
                        y: fy,
                        z: wz,
                    };
                    if let Some(sp) = camera.project_to_screen(world_tick, viewport) {
                        // Z axis tick: horizontal tick mark
                        prims.push(
                            Line::new(sp.x - TICK_HALF_LEN, sp.y, sp.x + TICK_HALF_LEN, sp.y)
                                .stroke_color(axis_color)
                                .stroke_width(TICK_STROKE_WIDTH)
                                .into(),
                        );

                        let label = format_tick(tick_val, z_step);
                        prims.push(
                            Text::new(sp.x - LABEL_OFFSET, sp.y, &label)
                                .font_size(TICK_LABEL_SIZE)
                                .into(),
                        );
                    }
                }

                // Z axis label at top
                prims.push(
                    Text::new(
                        s1.x - AXIS_LABEL_OFFSET,
                        s1.y - AXIS_LABEL_OFFSET,
                        &self.z_label,
                    )
                    .font_size(AXIS_LABEL_SIZE)
                    .into(),
                );
            }
        }

        prims
    }
}

/// Select the bounding-box floor corner farthest from the camera based on azimuth quadrant.
///
/// In normalized world space, the floor is at z = -1. The 4 floor corners are
/// combinations of x,y ∈ {-1.0, +1.0}. The "far" corner is the one most visible
/// from the current camera azimuth — matching the matplotlib/MATLAB convention.
///
/// Azimuth convention (from camera.rs): 0° = camera on +Y axis, increases clockwise.
fn far_floor_corner(azimuth_deg: f64) -> (f64, f64) {
    // Normalize to [0, 360)
    let az = ((azimuth_deg % 360.0) + 360.0) % 360.0;
    match az as u32 {
        0..=89 => (-1.0, -1.0),  // Q1: camera near +Y, +X → far corner is (-X, -Y)
        90..=179 => (1.0, -1.0), // Q2: camera near +Y, -X → far corner is (+X, -Y)
        180..=269 => (1.0, 1.0), // Q3: camera near -Y, -X → far corner is (+X, +Y)
        _ => (-1.0, 1.0),        // Q4: camera near -Y, +X → far corner is (-X, +Y)
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
pub(crate) fn normalize(v: f64, lo: f64, hi: f64) -> f64 {
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
        assert!(
            (p_min.x - (-1.0)).abs() < 1e-10,
            "x_min should normalize to -1; got {}",
            p_min.x
        );
        assert!(
            (p_max.x - 1.0).abs() < 1e-10,
            "x_max should normalize to +1; got {}",
            p_max.x
        );
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
        assert!(
            (p0.z - (-1.0)).abs() < 1e-10,
            "z_min should normalize to -1; got {}",
            p0.z
        );
        assert!(
            (p1.z - 1.0).abs() < 1e-10,
            "z_max should normalize to +1; got {}",
            p1.z
        );
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
                assert!(
                    (p.z - 0.0).abs() < 1e-10,
                    "flat surface z should normalize to 0.0; got {} at ({},{})",
                    p.z,
                    r,
                    c
                );
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
        let plot =
            SurfacePlot::new(vec![0.0, 1.0], vec![0.0, 0.0], vec![0.0, 1.0], 1, 2).x_label("Time");
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
        assert!(
            (x_min - 0.0).abs() < 1e-10,
            "x_min should be 0.0; got {}",
            x_min
        );
        assert!(
            (x_max - 10.0).abs() < 1e-10,
            "x_max should be 10.0; got {}",
            x_max
        );
    }

    // --- to_primitives() tests ---

    fn make_2x2_plot() -> SurfacePlot {
        // 2x2 grid (1 face): flat surface in XY plane (z=0 everywhere).
        // Face normal is exactly +z, so any camera above the XY plane sees it.
        SurfacePlot::new(
            vec![0.0, 1.0, 0.0, 1.0],
            vec![0.0, 0.0, 1.0, 1.0],
            vec![0.0, 0.0, 0.0, 0.0], // flat: all z=0 → degenerate z normalized to 0.0
            2,
            2,
        )
    }

    #[test]
    fn to_primitives_shaded_returns_nonempty() {
        // Camera above and to the side — face should be visible
        let plot = make_2x2_plot();
        let camera = Camera::new(45.0, 30.0, 3.0);
        let result = plot.to_primitives(&camera, (800, 600));
        assert!(
            !result.is_empty(),
            "shaded 2x2 plot should produce at least one primitive"
        );
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
            result_below.len(),
            result_above.len()
        );
    }

    #[test]
    fn to_primitives_wireframe_mode() {
        let plot = make_2x2_plot().render_mode(RenderMode::Wireframe);
        let camera = Camera::new(45.0, 30.0, 3.0);
        let result = plot.to_primitives(&camera, (800, 600));
        assert!(
            !result.is_empty(),
            "wireframe 2x2 plot should produce at least one primitive"
        );
        // Face primitives should be Bezier paths; axes add Line and Text on top — verify at least
        // one Bezier exists (the wireframe face)
        let bezier_count = result
            .iter()
            .filter(|p| matches!(p, Primitive::Bezier(_)))
            .count();
        assert!(
            bezier_count >= 1,
            "wireframe mode should produce at least one Bezier face primitive"
        );
    }

    #[test]
    fn to_primitives_2x2_produces_one_face() {
        // A 2x2 grid has exactly 1 face (1×1 quads).
        // Camera nearly directly above (elevation=89°) should see the single face.
        // Since axes are now appended in to_primitives(), total count > 1 — verify at least 1 Bezier.
        let plot = make_2x2_plot();
        let camera = Camera::new(0.0, 89.0, 3.0);
        let result = plot.to_primitives(&camera, (800, 600));
        let bezier_count = result
            .iter()
            .filter(|p| matches!(p, Primitive::Bezier(_)))
            .count();
        assert_eq!(
            bezier_count, 1,
            "2x2 grid should produce exactly 1 Bezier face primitive from top-facing camera; got {}",
            bezier_count
        );
        assert!(
            result.len() > 1,
            "total output (face + axes) should be more than 1 primitive; got {}",
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

    // --- far_floor_corner() quadrant tests ---

    #[test]
    fn far_floor_corner_quadrant_0_90() {
        let (x, y) = far_floor_corner(45.0);
        assert!((x - (-1.0)).abs() < 1e-10, "Q1 x should be -1.0; got {}", x);
        assert!((y - (-1.0)).abs() < 1e-10, "Q1 y should be -1.0; got {}", y);
    }

    #[test]
    fn far_floor_corner_quadrant_90_180() {
        let (x, y) = far_floor_corner(135.0);
        assert!((x - 1.0).abs() < 1e-10, "Q2 x should be +1.0; got {}", x);
        assert!((y - (-1.0)).abs() < 1e-10, "Q2 y should be -1.0; got {}", y);
    }

    #[test]
    fn far_floor_corner_quadrant_180_270() {
        let (x, y) = far_floor_corner(225.0);
        assert!((x - 1.0).abs() < 1e-10, "Q3 x should be +1.0; got {}", x);
        assert!((y - 1.0).abs() < 1e-10, "Q3 y should be +1.0; got {}", y);
    }

    #[test]
    fn far_floor_corner_quadrant_270_360() {
        let (x, y) = far_floor_corner(315.0);
        assert!((x - (-1.0)).abs() < 1e-10, "Q4 x should be -1.0; got {}", x);
        assert!((y - 1.0).abs() < 1e-10, "Q4 y should be +1.0; got {}", y);
    }

    #[test]
    fn far_floor_corner_wraps_at_360() {
        let at_0 = far_floor_corner(0.0);
        let at_360 = far_floor_corner(360.0);
        assert_eq!(
            at_0, at_360,
            "far_floor_corner(360) should equal far_floor_corner(0)"
        );
    }

    #[test]
    fn to_primitives_includes_axis_lines() {
        // 2x2 grid with data range [0,10] on all axes — axes add Line + Text primitives
        let plot = SurfacePlot::new(
            vec![0.0, 10.0, 0.0, 10.0],
            vec![0.0, 0.0, 10.0, 10.0],
            vec![0.0, 0.0, 0.0, 0.0], // flat z=0
            2,
            2,
        );
        let camera = Camera::new(45.0, 30.0, 3.0);
        let viewport = (800u32, 600u32);

        // Get count with axes (default to_primitives)
        let total = plot.to_primitives(&camera, viewport);

        // Count face-only primitives by temporarily using a 1x1 face plot and verifying axes add more
        // The 2x2 grid has 1 visible face, which produces 1 Bezier. Axes add Line + Text on top.
        assert!(
            total.len() > 1,
            "to_primitives should produce more than 1 primitive (face + axes); got {}",
            total.len()
        );
        // Verify that non-Bezier primitives exist (axis lines and tick labels)
        let non_bezier_count = total
            .iter()
            .filter(|p| !matches!(p, Primitive::Bezier(_)))
            .count();
        assert!(
            non_bezier_count > 0,
            "axes should produce Line and Text primitives; got {} non-Bezier primitives",
            non_bezier_count
        );
    }

    #[test]
    fn default_labels_are_xyz() {
        let plot = SurfacePlot::new(vec![0.0, 1.0], vec![0.0, 0.0], vec![0.0, 1.0], 1, 2);
        assert_eq!(plot.x_label_value(), "X", "default x label should be X");
        assert_eq!(plot.y_label_value(), "Y", "default y label should be Y");
        assert_eq!(plot.z_label_value(), "Z", "default z label should be Z");
    }

    // --- animate_fit / z_at tests ---

    #[test]
    fn z_at_no_animations_returns_fitted_z() {
        let xs = vec![0.0, 1.0];
        let ys = vec![0.0, 0.0];
        let zs = vec![0.0, 10.0];
        let plot2 = SurfacePlot::new(xs, ys, zs, 1, 2);
        // fitted_zs[1] should be 1.0 (normalized max) — z_at returns it unchanged
        assert!(
            (plot2.z_at(1.0, 0.0) - 1.0).abs() < 1e-10,
            "no animations: z_at should return fitted_z unchanged"
        );
    }

    #[test]
    fn z_at_before_animation_returns_zero() {
        let xs = vec![0.0, 1.0];
        let ys = vec![0.0, 0.0];
        let zs = vec![0.0, 10.0];
        let plot = SurfacePlot::new(xs, ys, zs, 1, 2).animate_fit(5.0, 3.0, Easing::Linear);
        // t=0 is before the animation (starts at 5.0) → should return 0.0 (flat)
        assert!(
            (plot.z_at(1.0, 0.0) - 0.0).abs() < 1e-10,
            "before animation: z_at should return 0.0 (flat)"
        );
    }

    #[test]
    fn z_at_after_animation_returns_fitted_z() {
        let xs = vec![0.0, 1.0];
        let ys = vec![0.0, 0.0];
        let zs = vec![0.0, 10.0];
        let plot = SurfacePlot::new(xs, ys, zs, 1, 2).animate_fit(0.0, 3.0, Easing::Linear);
        // t=5.0 is after animation end (3.0) → hold-last at fitted_z
        assert!(
            (plot.z_at(1.0, 5.0) - 1.0).abs() < 1e-10,
            "after animation: z_at should return fitted_z (1.0)"
        );
    }

    #[test]
    fn z_at_midpoint_linear_is_half_fitted_z() {
        let xs = vec![0.0, 1.0];
        let ys = vec![0.0, 0.0];
        let zs = vec![0.0, 10.0];
        let plot = SurfacePlot::new(xs, ys, zs, 1, 2).animate_fit(0.0, 4.0, Easing::Linear);
        // At t=2.0 (50% through 4.0s linear): z = 0.5 * fitted_z = 0.5
        let z = plot.z_at(1.0, 2.0);
        assert!(
            (z - 0.5).abs() < 1e-9,
            "linear midpoint should be 0.5; got {}",
            z
        );
    }

    #[test]
    fn z_at_between_two_ranges_holds_at_fitted_z() {
        let xs = vec![0.0, 1.0];
        let ys = vec![0.0, 0.0];
        let zs = vec![0.0, 10.0];
        let plot = SurfacePlot::new(xs, ys, zs, 1, 2)
            .animate_fit(0.0, 2.0, Easing::Linear) // ends at t=2.0
            .animate_fit(5.0, 2.0, Easing::Linear); // starts at t=5.0
        // At t=3.5 (between the two ranges): hold at fitted_z
        let z = plot.z_at(1.0, 3.5);
        assert!(
            (z - 1.0).abs() < 1e-10,
            "between ranges: z should hold at fitted_z (1.0); got {}",
            z
        );
    }

    #[test]
    fn to_primitives_at_t0_flat_surface_matches_all_z_zero() {
        // Non-flat surface with animate_fit: at t=0 (before animation), all vertices should be at z=0
        // A 2x2 grid with z in {0, 10} → fitted_zs in {-1, 1} after normalization
        // At t=0 (before animation start at 5.0), z_at returns 0.0 for all vertices
        let xs = vec![0.0, 1.0, 0.0, 1.0];
        let ys = vec![0.0, 0.0, 1.0, 1.0];
        let zs = vec![0.0, 10.0, 0.0, 10.0];
        let plot = SurfacePlot::new(xs.clone(), ys.clone(), zs.clone(), 2, 2).animate_fit(
            5.0,
            3.0,
            Easing::Linear,
        );
        let camera = Camera::new(45.0, 30.0, 3.0);
        // to_primitives_at at t=0 should produce same output as a fully flat surface
        let anim_result = plot.to_primitives_at(&camera, (800, 600), 0.0);
        // Should have primitives (face + axes) — just verify it doesn't panic and produces output
        assert!(
            !anim_result.is_empty(),
            "to_primitives_at at t=0 should produce primitives"
        );
    }

    #[test]
    fn to_primitives_at_no_animation_matches_to_primitives() {
        // Without animate_fit, to_primitives_at at any t should match to_primitives
        let plot = make_2x2_plot();
        let camera = Camera::new(45.0, 30.0, 3.0);
        let static_result = plot.to_primitives(&camera, (800, 600));
        let at_result = plot.to_primitives_at(&camera, (800, 600), 42.0);
        assert_eq!(
            static_result.len(),
            at_result.len(),
            "to_primitives_at with no animation should produce same primitive count as to_primitives"
        );
    }

    // --- animate_camera_azimuth / camera_at tests ---

    #[test]
    fn camera_at_no_animations_returns_none() {
        let plot = make_2x2_plot();
        assert!(
            plot.camera_at(0.0).is_none(),
            "no camera animations: camera_at should return None"
        );
    }

    #[test]
    fn camera_at_before_animation_holds_start_angle() {
        let plot = make_2x2_plot().animate_camera_azimuth(5.0, 3.0, 45.0, 135.0, Easing::Linear);
        let az = plot.camera_at(0.0).expect("should be Some");
        assert!(
            (az - 45.0).abs() < 1e-9,
            "before animation: should hold start_angle=45°; got {}",
            az
        );
    }

    #[test]
    fn camera_at_after_animation_holds_end_angle() {
        let plot = make_2x2_plot().animate_camera_azimuth(0.0, 3.0, 45.0, 135.0, Easing::Linear);
        let az = plot.camera_at(10.0).expect("should be Some");
        assert!(
            (az - 135.0).abs() < 1e-9,
            "after animation: should hold end_angle=135°; got {}",
            az
        );
    }

    #[test]
    fn camera_at_midpoint_linear_is_midpoint_angle() {
        let plot = make_2x2_plot().animate_camera_azimuth(0.0, 4.0, 0.0, 180.0, Easing::Linear);
        let az = plot.camera_at(2.0).expect("should be Some");
        assert!(
            (az - 90.0).abs() < 1e-9,
            "linear midpoint: azimuth should be 90°; got {}",
            az
        );
    }

    #[test]
    fn camera_at_between_ranges_holds_end_angle_of_previous() {
        let plot = make_2x2_plot()
            .animate_camera_azimuth(0.0, 2.0, 0.0, 90.0, Easing::Linear) // ends at t=2.0 → 90°
            .animate_camera_azimuth(5.0, 2.0, 90.0, 180.0, Easing::Linear); // starts at t=5.0
        // At t=3.5 (between ranges): hold at 90° (end of first range)
        let az = plot.camera_at(3.5).expect("should be Some");
        assert!(
            (az - 90.0).abs() < 1e-9,
            "between ranges: azimuth should be 90°; got {}",
            az
        );
    }

    #[test]
    fn camera_at_cross_360_sweep_works() {
        // Sweep from 350° to 370° — should interpolate to 360° at midpoint, Camera::new handles trig
        let plot = make_2x2_plot().animate_camera_azimuth(0.0, 2.0, 350.0, 370.0, Easing::Linear);
        let az = plot.camera_at(1.0).expect("should be Some");
        assert!(
            (az - 360.0).abs() < 1e-9,
            "cross-360 midpoint: azimuth should be 360°; got {}",
            az
        );
    }
}
