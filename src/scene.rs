// src/scene.rs
use crate::EidosError;
use std::sync::Arc;

pub struct Scene {
    width: u32,
    height: u32,
    fps: u32,
    duration_secs: f64,
    fontdb: Arc<resvg::usvg::fontdb::Database>,
}

/// Accumulates primitives during the render closure call.
pub struct SceneBuilder {
    primitives: Vec<crate::primitives::Primitive>,
    /// Depth (depth_sq) parallel to `primitives` — one entry per primitive.
    /// Non-surface primitives (axes, labels) use f64::NEG_INFINITY so they always
    /// render on top (painter's algorithm — NEG_INFINITY is "nearest").
    prim_depths: Vec<f64>,
    /// Squared centroid distances for all visible surface faces, populated by
    /// add_surface / add_surface_at. Read by add_scatter / add_scatter_at to
    /// determine behind-surface dimming. Empty if no surface has been added yet.
    face_depths: Vec<f64>,
}

impl SceneBuilder {
    /// Add a primitive to the scene. Returns &mut Self for chaining.
    ///
    /// Non-surface primitives (axes, labels, overlays) get depth = f64::NEG_INFINITY,
    /// ensuring they are always painted on top by the painter's algorithm.
    pub fn add(&mut self, p: impl Into<crate::primitives::Primitive>) -> &mut Self {
        self.primitives.push(p.into());
        self.prim_depths.push(f64::NEG_INFINITY);
        self
    }

    /// Decompose an Axes into its constituent primitives and add them all to the scene.
    /// Equivalent to calling add() for each primitive in axes.to_primitives().
    pub fn add_axes(&mut self, axes: &crate::dataviz::Axes) -> &mut Self {
        for prim in axes.to_primitives() {
            self.primitives.push(prim);
            self.prim_depths.push(f64::NEG_INFINITY);
        }
        self
    }

    /// Decompose a SurfacePlot into its constituent primitives and add them all to the scene.
    ///
    /// Uses the painter's algorithm internally (see SurfacePlot::to_primitives).
    /// Also records visible face depths for use by subsequent add_scatter() calls.
    pub fn add_surface(
        &mut self,
        plot: &crate::dataviz::SurfacePlot,
        camera: &crate::dataviz::Camera,
        viewport: (u32, u32),
    ) -> &mut Self {
        // Collect face depths (fast — no SVG work) for later scatter occlusion
        self.face_depths = plot.visible_face_depths(camera, viewport);

        // Collect face primitives with their depth tags
        let face_depths_snapshot = self.face_depths.clone();
        let prims = plot.to_primitives(camera, viewport);

        // Face primitives come back-to-front sorted; axis/label primitives are appended last.
        // We tag face primitives with their depth from face_depths_snapshot (same order),
        // and axis/label primitives with NEG_INFINITY (always on top).
        let face_count = face_depths_snapshot.len();
        // Sort face depths back-to-front (largest first) to match to_primitives order
        let mut sorted_face_depths: Vec<f64> = face_depths_snapshot;
        sorted_face_depths.sort_unstable_by(|a, b| b.total_cmp(a));
        let mut sorted_iter = sorted_face_depths.into_iter();

        for (i, prim) in prims.into_iter().enumerate() {
            let depth = if i < face_count {
                sorted_iter.next().unwrap_or(f64::NEG_INFINITY)
            } else {
                f64::NEG_INFINITY // axis/label primitives always on top
            };
            self.primitives.push(prim);
            self.prim_depths.push(depth);
        }
        self
    }

    /// Decompose an animated SurfacePlot into primitives at scene time t_secs.
    ///
    /// Uses to_primitives_at() internally — surface vertices are animated (via animate_fit)
    /// and the camera azimuth should be resolved via plot.camera_at(t_secs) before calling this.
    ///
    /// For static surfaces (no animate_fit called), this produces the same output as add_surface.
    /// For animated surfaces, call this instead of add_surface inside Scene::render closures.
    pub fn add_surface_at(
        &mut self,
        plot: &crate::dataviz::SurfacePlot,
        camera: &crate::dataviz::Camera,
        viewport: (u32, u32),
        t_secs: f64,
    ) -> &mut Self {
        self.face_depths = plot.visible_face_depths_at(camera, viewport, t_secs);

        let face_count = self.face_depths.len();
        let mut sorted_face_depths = self.face_depths.clone();
        sorted_face_depths.sort_unstable_by(|a, b| b.total_cmp(a));
        let mut sorted_iter = sorted_face_depths.into_iter();

        let prims = plot.to_primitives_at(camera, viewport, t_secs);
        for (i, prim) in prims.into_iter().enumerate() {
            let depth = if i < face_count {
                sorted_iter.next().unwrap_or(f64::NEG_INFINITY)
            } else {
                f64::NEG_INFINITY
            };
            self.primitives.push(prim);
            self.prim_depths.push(depth);
        }
        self
    }

    /// Add scatter points to the scene, depth-sorted alongside any previously added surface.
    ///
    /// Call after add_surface() or add_surface_at() — SceneBuilder uses internally accumulated
    /// face_depths for behind-surface dimming. If called before add_surface, no dimming occurs
    /// (graceful degradation — scatter renders at full depth-based opacity without occlusion).
    pub fn add_scatter(
        &mut self,
        scatter: &crate::dataviz::ScatterPlot,
        camera: &crate::dataviz::Camera,
        viewport: (u32, u32),
        t_secs: f64,
    ) -> &mut Self {
        let depth_circles =
            scatter.to_depth_sorted_circles_at(camera, viewport, &self.face_depths, t_secs);
        self.merge_scatter(depth_circles);
        self
    }

    /// Same as add_scatter — explicit alias for clarity in animated scenes.
    pub fn add_scatter_at(
        &mut self,
        scatter: &crate::dataviz::ScatterPlot,
        camera: &crate::dataviz::Camera,
        viewport: (u32, u32),
        t_secs: f64,
    ) -> &mut Self {
        self.add_scatter(scatter, camera, viewport, t_secs)
    }

    /// Merge depth-tagged scatter circles into self.primitives in back-to-front order.
    ///
    /// O(n+m) merge of two back-to-front sorted lists (larger depth_sq = farther from camera).
    fn merge_scatter(&mut self, mut circles: Vec<(f64, crate::primitives::Primitive)>) {
        if circles.is_empty() {
            return;
        }

        // Sort circles back-to-front (largest depth_sq first, same convention as prim_depths)
        circles.sort_unstable_by(|a, b| b.0.total_cmp(&a.0));

        // Drain existing primitives into a temp vec for O(n+m) merge
        let old_prims: Vec<crate::primitives::Primitive> = self.primitives.drain(..).collect();
        let old_depths: Vec<f64> = self.prim_depths.drain(..).collect();

        let total = old_prims.len() + circles.len();
        let mut merged_prims: Vec<crate::primitives::Primitive> = Vec::with_capacity(total);
        let mut merged_depths: Vec<f64> = Vec::with_capacity(total);

        let mut ci = 0usize;
        for (prim, pd) in old_prims.into_iter().zip(old_depths.into_iter()) {
            // Insert any circles that are farther than this primitive (larger depth_sq)
            while ci < circles.len() && circles[ci].0 >= pd {
                merged_depths.push(circles[ci].0);
                merged_prims.push(circles[ci].1.clone());
                ci += 1;
            }
            merged_depths.push(pd);
            merged_prims.push(prim);
        }
        // Append remaining circles (closer to camera than all existing primitives)
        for idx in ci..circles.len() {
            merged_depths.push(circles[idx].0);
            merged_prims.push(circles[idx].1.clone());
        }

        self.primitives = merged_prims;
        self.prim_depths = merged_depths;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation::Easing;
    use crate::dataviz::{Camera, SurfacePlot};

    #[test]
    fn add_surface_adds_primitives_to_builder() {
        // Flat 2x2 grid with normal pointing +z — visible from any above-horizon camera
        let plot = SurfacePlot::new(
            vec![0.0, 1.0, 0.0, 1.0],
            vec![0.0, 0.0, 1.0, 1.0],
            vec![0.0, 0.0, 0.0, 0.0],
            2,
            2,
        );
        let camera = Camera::new(45.0, 30.0, 3.0);
        let mut sb = SceneBuilder {
            primitives: vec![],
            prim_depths: vec![],
            face_depths: vec![],
        };
        sb.add_surface(&plot, &camera, (800, 600));
        assert!(
            !sb.primitives.is_empty(),
            "add_surface should produce at least one primitive"
        );
    }

    #[test]
    fn add_surface_at_produces_primitives() {
        // Flat 2x2 grid with morph animation registered
        let plot = SurfacePlot::new(
            vec![0.0, 1.0, 0.0, 1.0],
            vec![0.0, 0.0, 1.0, 1.0],
            vec![0.0, 0.0, 0.0, 0.0],
            2,
            2,
        )
        .animate_fit(0.0, 3.0, Easing::Linear);
        let camera = Camera::new(45.0, 30.0, 3.0);
        let mut sb = SceneBuilder {
            primitives: vec![],
            prim_depths: vec![],
            face_depths: vec![],
        };
        sb.add_surface_at(&plot, &camera, (800, 600), 1.5);
        assert!(
            !sb.primitives.is_empty(),
            "add_surface_at should produce primitives"
        );
    }
}

impl Scene {
    /// Create a new Scene with the given video configuration.
    ///
    /// Validates:
    /// - width and height must both be even (H.264 requirement)
    /// - fps must be > 0
    /// - ffmpeg must be present on PATH
    ///
    /// Initializes fontdb with Noto Sans loaded once at construction time.
    pub fn new(width: u32, height: u32, fps: u32) -> Result<Self, EidosError> {
        // H.264 requires even dimensions
        if width % 2 != 0 {
            return Err(EidosError::InvalidConfig(
                "width must be an even number".into(),
            ));
        }
        if height % 2 != 0 {
            return Err(EidosError::InvalidConfig(
                "height must be an even number".into(),
            ));
        }

        // fps must be non-zero
        if fps == 0 {
            return Err(EidosError::InvalidConfig(
                "fps must be greater than zero".into(),
            ));
        }

        // Probe for ffmpeg on PATH at construction time so failures are eager and clear
        match std::process::Command::new("ffmpeg")
            .arg("-version")
            .output()
        {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(EidosError::RenderFailed(
                    "ffmpeg not found on PATH — install ffmpeg to use eidos".into(),
                ));
            }
            Err(e) => {
                return Err(EidosError::RenderFailed(format!(
                    "failed to probe ffmpeg: {}",
                    e
                )));
            }
        }

        // Initialize fontdb once — reused for every frame rasterization (not per-frame)
        let mut fontdb = resvg::usvg::fontdb::Database::new();
        fontdb.load_font_data(ttf_noto_sans::REGULAR.to_vec());

        Ok(Scene {
            width,
            height,
            fps,
            duration_secs: 1.0,
            fontdb: Arc::new(fontdb),
        })
    }

    /// Set the video duration in seconds (builder-style, consumes self).
    pub fn duration(mut self, secs: f64) -> Self {
        self.duration_secs = secs;
        self
    }

    /// Render the scene to an MP4 file at `output_path`.
    ///
    /// The `build_scene` closure receives a `&mut SceneBuilder` and scene time in seconds (f64).
    /// It is called once per frame — use the time parameter to interpolate animated properties.
    ///
    /// For static scenes, use `render_static()` which wraps this with `|s, _t| f(s)`.
    ///
    /// fontdb is cloned (Arc clone — cheap) before the loop; not re-initialized per frame.
    pub fn render<F, P>(&self, build_scene: F, output_path: P) -> Result<(), EidosError>
    where
        F: Fn(&mut SceneBuilder, f64),
        P: AsRef<std::path::Path>,
    {
        let total_frames = (self.fps as f64 * self.duration_secs).round() as u64;
        let width = self.width;
        let height = self.height;
        let fps = self.fps;
        let fontdb = self.fontdb.clone(); // Arc clone — cheap, done once before loop

        crate::svg_gen::encode_to_mp4_animated(
            |frame_idx| {
                let t_secs = frame_idx as f64 / fps as f64;
                let mut builder = SceneBuilder {
                    primitives: Vec::new(),
                    prim_depths: Vec::new(),
                    face_depths: Vec::new(),
                };
                build_scene(&mut builder, t_secs);
                let svg = crate::svg_gen::build_svg_document(width, height, &builder.primitives);
                crate::svg_gen::rasterize_frame(&svg, width, height, &fontdb)
            },
            total_frames,
            width,
            height,
            fps,
            output_path.as_ref(),
        )
    }

    /// Convenience wrapper for static (non-animated) scenes.
    /// Accepts `Fn(&mut SceneBuilder)` — the Phase 1 API.
    /// Delegates to `render()` with the time parameter ignored.
    /// Use `render()` directly for animated scenes.
    pub fn render_static<F, P>(&self, build_scene: F, output_path: P) -> Result<(), EidosError>
    where
        F: Fn(&mut SceneBuilder),
        P: AsRef<std::path::Path>,
    {
        self.render(|s, _t| build_scene(s), output_path)
    }
}
