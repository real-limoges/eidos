// src/svg_gen.rs
// SVG document assembly, rasterization, and ffmpeg encoding pipeline.
// build_svg_document() and rasterize_frame() are implemented here (plan 01-02).
// SVG primitive conversion dispatch is completed in plan 01-05.
use crate::EidosError;

/// Build an SVG document string from a list of primitives.
///
/// Sets width, height, and viewBox to the same values so primitive coordinates
/// map 1:1 to output pixels (avoids scaling surprises — research Pitfall 6).
///
/// Adds a black background rectangle as the first element.
///
/// Primitive-to-SVG conversion stubs are here; full dispatch is completed in plan 01-05.
/// Arrow defs (markers) are emitted first, before shape nodes, as required by the SVG spec
/// (defs must precede references — research Pitfall 3).
pub fn build_svg_document(
    width: u32,
    height: u32,
    primitives: &[crate::primitives::Primitive],
) -> String {
    use svg::Document;
    use svg::node::element::Rectangle;

    let mut doc = Document::new()
        .set("width", width)
        .set("height", height)
        .set("viewBox", format!("0 0 {} {}", width, height));

    // Black background rectangle
    let bg = Rectangle::new()
        .set("x", 0)
        .set("y", 0)
        .set("width", width)
        .set("height", height)
        .set("fill", "black");
    doc = doc.add(bg);

    // Pass 1: collect SVG Definitions (Arrow markers must precede their references).
    // TODO(01-05): implement Arrow defs extraction when Arrow struct is complete.
    for primitive in primitives {
        match primitive {
            crate::primitives::Primitive::Arrow(_) => {
                // TODO(01-05): add arrow marker Definitions node here
            }
            _ => {}
        }
    }

    // Pass 2: add shape nodes
    for primitive in primitives {
        match primitive {
            crate::primitives::Primitive::Circle(_circle) => {
                // TODO(01-05): convert Circle to SVG circle element
            }
            crate::primitives::Primitive::Rect(_rect) => {
                // TODO(01-05): convert Rect to SVG rect element
            }
            crate::primitives::Primitive::Line(_line) => {
                // TODO(01-05): convert Line to SVG line element
            }
            crate::primitives::Primitive::Arrow(_arrow) => {
                // TODO(01-05): convert Arrow to SVG line + marker-end element
            }
            crate::primitives::Primitive::Text(_text) => {
                // TODO(01-05): convert Text to SVG text element
            }
            crate::primitives::Primitive::Bezier(_bezier) => {
                // TODO(01-05): convert Bezier to SVG path element
            }
        }
    }

    doc.to_string()
}

/// Rasterize an SVG string to BGRA8 pixel bytes using resvg + tiny-skia.
///
/// The `fontdb` parameter must already have Noto Sans loaded (done in Scene::new()).
/// A new fontdb is NOT created here — passing it as an Arc avoids per-frame font loading.
/// The fontdb is set on `Options.fontdb` which is how resvg 0.47 expects it to be supplied.
pub fn rasterize_frame(
    svg_str: &str,
    width: u32,
    height: u32,
    fontdb: &std::sync::Arc<resvg::usvg::fontdb::Database>,
) -> Result<Vec<u8>, EidosError> {
    use resvg::usvg::{self, Options};
    use resvg::tiny_skia;

    let mut options = Options::default();
    options.fontdb = fontdb.clone();

    let tree = usvg::Tree::from_str(svg_str, &options)
        .map_err(|e| EidosError::RenderFailed(format!("SVG parse error: {}", e)))?;

    let mut pixmap = tiny_skia::Pixmap::new(width, height)
        .ok_or_else(|| EidosError::RenderFailed("failed to allocate pixmap".into()))?;

    resvg::render(&tree, tiny_skia::Transform::identity(), &mut pixmap.as_mut());

    Ok(pixmap.data().to_vec()) // BGRA8 bytes
}

/// Encode raw BGRA frames to an H.264 MP4 file via ffmpeg subprocess.
///
/// The same `bgra_frame` is written `total_frames` times for Phase 1 static scenes.
/// Phase 2 will instead supply a different frame per animation step.
///
/// Uses "-pix_fmt bgra" (not rgba) because tiny-skia Pixmap::data() returns BGRA byte order
/// (research Pitfall 2 — confusing rgba/bgra causes color channel corruption).
///
/// stdin is explicitly dropped before wait() so ffmpeg receives EOF and finalizes the file.
/// Forgetting to close stdin causes ffmpeg to block indefinitely waiting for more input.
pub fn encode_to_mp4(
    bgra_frame: &[u8],
    total_frames: u64,
    width: u32,
    height: u32,
    fps: u32,
    output_path: &std::path::Path,
) -> Result<(), EidosError> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let output_str = output_path
        .to_str()
        .ok_or_else(|| EidosError::InvalidConfig("output path is not valid UTF-8".into()))?;

    let mut child = Command::new("ffmpeg")
        .args([
            "-f",
            "rawvideo",
            "-pix_fmt",
            "bgra", // matches tiny-skia Pixmap::data() byte order — NOT rgba (Pitfall 2)
            "-s",
            &format!("{}x{}", width, height),
            "-r",
            &fps.to_string(),
            "-i",
            "pipe:0",
            "-pix_fmt",
            "yuv420p", // H.264 requires YUV color space
            "-c:v",
            "libx264",
            "-y", // overwrite output file without prompting
            output_str,
        ])
        .stdin(Stdio::piped())
        .stderr(Stdio::null()) // suppress ffmpeg log noise; use Stdio::inherit() for debugging
        .spawn()
        .map_err(|e| EidosError::RenderFailed(format!("failed to spawn ffmpeg: {}", e)))?;

    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| EidosError::RenderFailed("failed to open ffmpeg stdin".into()))?;

        // Write the same static frame total_frames times.
        // Phase 2 will write a different frame for each animation step.
        for _ in 0..total_frames {
            stdin.write_all(bgra_frame).map_err(|e| {
                EidosError::RenderFailed(format!("failed to write frame to ffmpeg: {}", e))
            })?;
        }
    } // stdin borrow is dropped here (via block scope)

    // Explicitly drop stdin so ffmpeg receives EOF and knows input is complete.
    // Without this, ffmpeg blocks indefinitely waiting for more input before finalizing.
    drop(child.stdin.take());

    let status = child
        .wait()
        .map_err(|e| EidosError::RenderFailed(format!("ffmpeg wait failed: {}", e)))?;

    if !status.success() {
        return Err(EidosError::RenderFailed(format!(
            "ffmpeg exited with non-zero status: {}",
            status
        )));
    }

    Ok(())
}
