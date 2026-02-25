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

    // Pass 1: collect all arrow defs and add them before any shape elements (Pitfall 3).
    // SVG requires <defs> to appear before elements that reference them via url(#id).
    for primitive in primitives {
        if let crate::primitives::Primitive::Arrow(arrow) = primitive {
            let (defs, _line) = arrow.to_svg_parts();
            doc = doc.add(defs);
        }
    }

    // Pass 2: add all shape elements in order
    for primitive in primitives {
        doc = match primitive {
            crate::primitives::Primitive::Circle(c) => doc.add(c.to_svg_element()),
            crate::primitives::Primitive::Rect(r) => doc.add(r.to_svg_element()),
            crate::primitives::Primitive::Line(l) => doc.add(l.to_svg_element()),
            crate::primitives::Primitive::Arrow(a) => {
                let (_defs, line) = a.to_svg_parts(); // defs already added in Pass 1
                doc.add(line)
            }
            crate::primitives::Primitive::Text(t) => doc.add(t.to_svg_element()),
            crate::primitives::Primitive::Bezier(b) => doc.add(b.to_svg_element()),
        };
    }

    doc.to_string()
}

/// Rasterize an SVG string to RGBA8 pixel bytes using resvg + tiny-skia.
///
/// The `fontdb` parameter must already have Noto Sans loaded (done in Scene::new()).
/// A new fontdb is NOT created here — passing it as an Arc avoids per-frame font loading.
/// The fontdb is set on `Options.fontdb` which is how resvg 0.47 expects it to be supplied.
///
/// Note: tiny-skia Pixmap::data() returns RGBA byte order (per tiny-skia source docs).
/// The ffmpeg input format must be set to "rgba" to match — NOT "bgra".
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

    Ok(pixmap.data().to_vec()) // RGBA8 bytes (tiny-skia Pixmap::data() is RGBA, not BGRA)
}

/// Encode raw RGBA frames to an H.264 MP4 file via ffmpeg subprocess.
///
/// Calls `frame_fn(frame_index)` once per frame to generate unique RGBA bytes for each frame.
/// `frame_fn(frame_index)` once per frame to generate unique RGBA bytes for each frame.
/// Frames are streamed directly to ffmpeg stdin — no memory accumulation.
///
/// frame_fn receives the 0-based frame index (u64). Scene time in seconds is
/// computed by the caller as `frame_idx as f64 / fps as f64` before calling frame_fn.
///
/// Frames are written to ffmpeg stdin immediately after generation (streaming),
/// not buffered — avoids OOM on long animations (research Pitfall 5).
pub fn encode_to_mp4_animated<F>(
    frame_fn: F,
    total_frames: u64,
    width: u32,
    height: u32,
    fps: u32,
    output_path: &std::path::Path,
) -> Result<(), EidosError>
where
    F: Fn(u64) -> Result<Vec<u8>, EidosError>,
{
    use std::io::Write;
    use std::process::{Command, Stdio};

    let output_str = output_path
        .to_str()
        .ok_or_else(|| EidosError::InvalidConfig("output path is not valid UTF-8".into()))?;

    let mut child = Command::new("ffmpeg")
        .args([
            "-f", "rawvideo",
            "-pix_fmt", "rgba",  // matches tiny-skia RGBA byte order (NOT bgra)
            "-s", &format!("{}x{}", width, height),
            "-r", &fps.to_string(),
            "-i", "pipe:0",
            "-pix_fmt", "yuv420p",
            "-c:v", "libx264",
            "-y",
            output_str,
        ])
        .stdin(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| EidosError::RenderFailed(format!("failed to spawn ffmpeg: {}", e)))?;

    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| EidosError::RenderFailed("failed to open ffmpeg stdin".into()))?;

        for frame_idx in 0..total_frames {
            let rgba = frame_fn(frame_idx)?;
            stdin.write_all(&rgba).map_err(|e| {
                EidosError::RenderFailed(format!("failed to write frame to ffmpeg: {}", e))
            })?;
        }
    }

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
