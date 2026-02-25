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
    /// The `build_scene` closure receives a `&mut SceneBuilder` and should call
    /// `builder.add(primitive)` for each primitive to include in the scene.
    ///
    /// For Phase 1 (static scenes), all frames are identical — the same SVG is
    /// rasterized once and written to ffmpeg stdin `total_frames` times.
    ///
    /// Phase 2 will call build_scene() per-frame with animated state instead.
    pub fn render<F, P>(&self, build_scene: F, output_path: P) -> Result<(), EidosError>
    where
        F: Fn(&mut SceneBuilder),
        P: AsRef<std::path::Path>,
    {
        let mut builder = SceneBuilder {
            primitives: Vec::new(),
        };
        build_scene(&mut builder);

        // Generate SVG string from accumulated primitives
        let svg_string = crate::svg_gen::build_svg_document(
            self.width,
            self.height,
            &builder.primitives,
        );

        // Rasterize once — for static Phase 1 scenes all frames are identical
        // Phase 2 will rasterize a different frame per animation step
        let rgba_frame = crate::svg_gen::rasterize_frame(
            &svg_string,
            self.width,
            self.height,
            &self.fontdb,
        )?;

        // Total frames = fps * duration (rounded to nearest whole frame)
        let total_frames = (self.fps as f64 * self.duration_secs).round() as u64;

        // Encode to H.264 MP4 via ffmpeg subprocess
        crate::svg_gen::encode_to_mp4(
            &rgba_frame,
            total_frames,
            self.width,
            self.height,
            self.fps,
            output_path.as_ref(),
        )
    }
}
