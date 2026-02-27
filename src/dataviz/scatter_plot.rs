//! Scatter plot rendering layer for eidos.
//!
//! Produces depth-tagged [`Circle`] primitives suitable for merging into
//! the painter's algorithm alongside [`SurfacePlot`] face primitives.

use crate::dataviz::camera::{Camera, Point3D};
use crate::dataviz::surface_plot::normalize;
use crate::primitives::{Circle, Primitive};
use crate::Color;

// ── Animation ────────────────────────────────────────────────────────────────

/// Fade-in animation parameters for a scatter plot.
struct ScatterAnimation {
    start_time: f64,
    duration: f64,
}

// ── Constants ────────────────────────────────────────────────────────────────

/// Opacity factor applied to points classified as behind a visible surface face.
/// Within the locked 15–20% range from CONTEXT.md.
const BEHIND_SURFACE_DIM: f64 = 0.17;

/// Hard floor applied to final_alpha to prevent completely invisible primitives.
const ALPHA_FLOOR: f64 = 0.03;

// ── ScatterPlot ───────────────────────────────────────────────────────────────

/// A scatter plot that renders 3D data points as depth-sorted [`Circle`] primitives.
///
/// Points are normalized into world space using the same extents as the accompanying
/// [`SurfacePlot`], then projected through the camera and depth-opacity blended.
///
/// # Example
/// ```rust,ignore
/// let scatter = ScatterPlot::new(points, surface.data_extents())
///     .with_color(Color::rgb(255, 120, 50))
///     .animate_fade(1.0, 3.0);
/// let circles = scatter.to_depth_sorted_circles_at(&camera, (800, 600), &face_depths, t);
/// ```
pub struct ScatterPlot {
    points: Vec<(f64, f64, f64)>,
    x_data_min: f64,
    x_data_max: f64,
    y_data_min: f64,
    y_data_max: f64,
    z_data_min: f64,
    z_data_max: f64,
    color: Color,
    radius: f64,
    fade_anim: Option<ScatterAnimation>,
}

impl ScatterPlot {
    /// Create a new scatter plot.
    ///
    /// - `points`: raw `(x, y, z)` data coordinates (not pre-normalized)
    /// - `surface_extents`: the 6-tuple returned by `SurfacePlot::data_extents()`
    ///   `(x_min, x_max, y_min, y_max, z_min, z_max)` — defines the normalization range
    pub fn new(
        points: Vec<(f64, f64, f64)>,
        surface_extents: (f64, f64, f64, f64, f64, f64),
    ) -> Self {
        let (x_data_min, x_data_max, y_data_min, y_data_max, z_data_min, z_data_max) =
            surface_extents;
        ScatterPlot {
            points,
            x_data_min,
            x_data_max,
            y_data_min,
            y_data_max,
            z_data_min,
            z_data_max,
            color: Color::rgb(255, 120, 50), // warm orange — visible against viridis colormap
            radius: 4.5,                     // within 4–5px locked range
            fade_anim: None,
        }
    }

    /// Override the point color. Default: `Color::rgb(255, 120, 50)` (warm orange).
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Override the point radius in pixels. Default: 4.5.
    pub fn with_radius(mut self, r: f64) -> Self {
        self.radius = r;
        self
    }

    /// Attach a linear fade-in animation.
    ///
    /// - Before `t_start`: all points are invisible (opacity 0, skipped entirely)
    /// - Between `t_start` and `t_end`: opacity scales linearly from 0 → 1
    /// - After `t_end`: full opacity
    pub fn animate_fade(mut self, t_start: f64, t_end: f64) -> Self {
        self.fade_anim = Some(ScatterAnimation {
            start_time: t_start,
            duration: (t_end - t_start).max(0.0),
        });
        self
    }

    // ── Private helpers ──────────────────────────────────────────────────────

    /// Normalize raw data coordinates into world space [-1, 1] per axis.
    fn world_point(&self, x: f64, y: f64, z: f64) -> Point3D {
        Point3D {
            x: normalize(x, self.x_data_min, self.x_data_max),
            y: normalize(y, self.y_data_min, self.y_data_max),
            z: normalize(z, self.z_data_min, self.z_data_max),
        }
    }

    /// Return the fade multiplier [0.0, 1.0] for the given playback time.
    ///
    /// - No animation attached → always 1.0
    /// - Before `start_time` → 0.0
    /// - After `start_time + duration` → 1.0
    /// - In between → linear interpolation
    fn fade_at(&self, t_secs: f64) -> f64 {
        let Some(ref anim) = self.fade_anim else {
            return 1.0;
        };
        if t_secs < anim.start_time {
            return 0.0;
        }
        if anim.duration < 1e-12 || t_secs >= anim.start_time + anim.duration {
            return 1.0;
        }
        (t_secs - anim.start_time) / anim.duration
    }

    /// Compute depth-based opacity using exponential falloff.
    ///
    /// - Nearest point → opacity near 1.0
    /// - Farthest point → opacity floored at 0.25
    fn depth_opacity(depth_sq: f64, min_depth_sq: f64, max_depth_sq: f64) -> f64 {
        const FLOOR: f64 = 0.25;
        let range = max_depth_sq - min_depth_sq;
        if range < 1e-12 {
            return 1.0;
        }
        let t = (depth_sq - min_depth_sq) / range;
        let raw = (-3.0 * t).exp(); // maps [0,1] → [1.0, ~0.05]
        FLOOR + (1.0 - FLOOR) * raw
    }

    // ── Public rendering methods ─────────────────────────────────────────────

    /// Produce depth-tagged [`Circle`] primitives for all visible scatter points.
    ///
    /// Returns `Vec<(depth_sq, Primitive::Circle)>` — unsorted. The caller (SceneBuilder)
    /// is responsible for merge-sorting with surface face primitives before rendering.
    ///
    /// # Parameters
    /// - `camera`: the current camera (provides projection and eye position)
    /// - `viewport`: `(width_px, height_px)` SVG canvas size
    /// - `face_depths`: squared distances from camera to each visible surface face centroid.
    ///   Any point with a `depth_sq` greater than at least one `face_depth` entry is
    ///   considered partially occluded and dimmed to ~17% opacity.
    pub fn to_depth_sorted_circles(
        &self,
        camera: &Camera,
        viewport: (u32, u32),
        face_depths: &[f64],
    ) -> Vec<(f64, Primitive)> {
        self.render_circles(camera, viewport, face_depths, 1.0)
    }

    /// Same as [`to_depth_sorted_circles`] but multiplies the fade animation progress
    /// at time `t_secs` into every point's opacity.
    ///
    /// Points are skipped entirely when `fade_at(t_secs) == 0.0` (pre-fade period)
    /// — this saves projection and SVG work for invisible points.
    pub fn to_depth_sorted_circles_at(
        &self,
        camera: &Camera,
        viewport: (u32, u32),
        face_depths: &[f64],
        t_secs: f64,
    ) -> Vec<(f64, Primitive)> {
        let fade = self.fade_at(t_secs);
        if fade == 0.0 {
            return Vec::new();
        }
        self.render_circles(camera, viewport, face_depths, fade)
    }

    /// Internal: project all points, compute depth, apply opacity, return primitives.
    fn render_circles(
        &self,
        camera: &Camera,
        viewport: (u32, u32),
        face_depths: &[f64],
        fade: f64,
    ) -> Vec<(f64, Primitive)> {
        let (eye_x, eye_y, eye_z) = camera.eye_position();

        // Pass 1: project all points and compute depth_sq for each visible point.
        // Collect (depth_sq, screen_pt) — skip points that project to None.
        let projected: Vec<(f64, crate::dataviz::camera::Point2D)> = self
            .points
            .iter()
            .filter_map(|&(x, y, z)| {
                let wp = self.world_point(x, y, z);
                let screen = camera.project_to_screen(wp, viewport)?;
                let dx = wp.x - eye_x;
                let dy = wp.y - eye_y;
                let dz = wp.z - eye_z;
                let depth_sq = dx * dx + dy * dy + dz * dz;
                Some((depth_sq, screen))
            })
            .collect();

        if projected.is_empty() {
            return Vec::new();
        }

        // Pass 2: find min/max depth_sq across the visible set for normalization.
        let min_depth_sq = projected
            .iter()
            .map(|(d, _)| *d)
            .fold(f64::INFINITY, f64::min);
        let max_depth_sq = projected
            .iter()
            .map(|(d, _)| *d)
            .fold(f64::NEG_INFINITY, f64::max);

        // Pass 3: compute opacity for each point and build Circle primitives.
        projected
            .into_iter()
            .map(|(depth_sq, screen)| {
                let d_opacity = Self::depth_opacity(depth_sq, min_depth_sq, max_depth_sq);
                // Point is "behind surface" if any face centroid is closer to the camera.
                let is_behind = face_depths.iter().any(|&fd| fd < depth_sq);
                let occlusion = if is_behind { BEHIND_SURFACE_DIM } else { 1.0 };
                let final_alpha = (d_opacity * occlusion * fade).clamp(ALPHA_FLOOR, 1.0);

                let circle = Circle::new(screen.x, screen.y, self.radius)
                    .fill(self.color)
                    .opacity(final_alpha);

                (depth_sq, Primitive::Circle(circle))
            })
            .collect()
    }
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataviz::camera::Camera;
    use crate::primitives::Primitive;

    #[test]
    fn scatter_fade_at_interpolates() {
        let scatter = ScatterPlot::new(vec![(0.0, 0.0, 0.0)], (0.0, 1.0, 0.0, 1.0, 0.0, 1.0))
            .animate_fade(2.0, 4.0);
        assert_eq!(scatter.fade_at(1.0), 0.0);
        assert!((scatter.fade_at(3.0) - 0.5).abs() < 1e-9);
        assert_eq!(scatter.fade_at(5.0), 1.0);
    }

    #[test]
    fn scatter_static_fade_is_one() {
        let scatter = ScatterPlot::new(vec![(0.5, 0.5, 0.5)], (0.0, 1.0, 0.0, 1.0, 0.0, 1.0));
        assert_eq!(scatter.fade_at(0.0), 1.0);
        assert_eq!(scatter.fade_at(999.0), 1.0);
    }

    #[test]
    fn scatter_circles_at_empty_before_fade_start() {
        let scatter = ScatterPlot::new(vec![(0.5, 0.5, 0.5)], (0.0, 1.0, 0.0, 1.0, 0.0, 1.0))
            .animate_fade(3.0, 5.0);
        let camera = Camera::new(45.0, 30.0, 3.0);
        let result = scatter.to_depth_sorted_circles_at(&camera, (800, 600), &[], 0.0);
        assert!(result.is_empty(), "no circles before fade start");
    }

    #[test]
    fn scatter_single_point_produces_one_circle() {
        let scatter = ScatterPlot::new(vec![(0.5, 0.5, 0.5)], (0.0, 1.0, 0.0, 1.0, 0.0, 1.0));
        let camera = Camera::new(45.0, 30.0, 3.0);
        let result = scatter.to_depth_sorted_circles(&camera, (800, 600), &[]);
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0].1, Primitive::Circle(_)));
    }
}
