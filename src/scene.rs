// src/scene.rs
use std::sync::Arc;
use crate::EidosError;

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
}

impl SceneBuilder {
    /// Add a primitive to the scene. Returns &mut Self for chaining.
    pub fn add(&mut self, p: impl Into<crate::primitives::Primitive>) -> &mut Self {
        self.primitives.push(p.into());
        self
    }

    /// Decompose an Axes into its constituent primitives and add them all to the scene.
    /// Equivalent to calling add() for each primitive in axes.to_primitives().
    pub fn add_axes(&mut self, axes: &crate::dataviz::Axes) -> &mut Self {
        for prim in axes.to_primitives() {
            self.primitives.push(prim);
        }
        self
    }

    /// Decompose a SurfacePlot into its constituent primitives and add them all to the scene.
    ///
    /// Uses the painter's algorithm internally (see SurfacePlot::to_primitives).
    /// Equivalent to calling add() for each primitive in plot.to_primitives(camera, viewport).
    pub fn add_surface(
        &mut self,
        plot: &crate::dataviz::SurfacePlot,
        camera: &crate::dataviz::Camera,
        viewport: (u32, u32),
    ) -> &mut Self {
        for prim in plot.to_primitives(camera, viewport) {
            self.primitives.push(prim);
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
        for prim in plot.to_primitives_at(camera, viewport, t_secs) {
            self.primitives.push(prim);
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataviz::{SurfacePlot, Camera};
    use crate::animation::Easing;

    #[test]
    fn add_surface_adds_primitives_to_builder() {
        // Flat 2x2 grid with normal pointing +z — visible from any above-horizon camera
        let plot = SurfacePlot::new(
            vec![0.0, 1.0, 0.0, 1.0],
            vec![0.0, 0.0, 1.0, 1.0],
            vec![0.0, 0.0, 0.0, 0.0],
            2, 2,
        );
        let camera = Camera::new(45.0, 30.0, 3.0);
        let mut sb = SceneBuilder { primitives: vec![] };
        sb.add_surface(&plot, &camera, (800, 600));
        assert!(!sb.primitives.is_empty(), "add_surface should produce at least one primitive");
    }

    #[test]
    fn add_surface_at_produces_primitives() {
        // Flat 2x2 grid with morph animation registered
        let plot = SurfacePlot::new(
            vec![0.0, 1.0, 0.0, 1.0],
            vec![0.0, 0.0, 1.0, 1.0],
            vec![0.0, 0.0, 0.0, 0.0],
            2, 2,
        ).animate_fit(0.0, 3.0, Easing::Linear);
        let camera = Camera::new(45.0, 30.0, 3.0);
        let mut sb = SceneBuilder { primitives: vec![] };
        sb.add_surface_at(&plot, &camera, (800, 600), 1.5);
        assert!(!sb.primitives.is_empty(), "add_surface_at should produce primitives");
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
                let mut builder = SceneBuilder { primitives: Vec::new() };
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
