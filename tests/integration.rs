// tests/integration.rs
use eidos::{Color, Scene};
use eidos::primitives::{Circle, Rect, Text};
use std::path::Path;

/// Returns true if ffmpeg is available on PATH.
fn ffmpeg_available() -> bool {
    std::process::Command::new("ffmpeg")
        .arg("-version")
        .output()
        .is_ok()
}

#[test]
fn render_scene_with_all_primitives_produces_mp4() {
    if !ffmpeg_available() {
        eprintln!("SKIP: ffmpeg not found on PATH");
        return;
    }

    let output_path = "/tmp/eidos_integration_test.mp4";

    // Clean up any prior run
    let _ = std::fs::remove_file(output_path);

    let scene = Scene::new(640, 480, 24).unwrap().duration(1.0);

    scene.render(|s| {
        s.add(Circle::new(320.0, 240.0, 100.0).fill(Color::RED));
        s.add(Rect::new(50.0, 50.0, 200.0, 100.0).fill(Color::BLUE));
        s.add(Text::new(320.0, 400.0, "Integration Test").fill(Color::WHITE));
    }, output_path).expect("render should succeed");

    assert!(
        Path::new(output_path).exists(),
        "MP4 file should exist at {}", output_path
    );

    // Verify file size is non-trivial (at least 1KB — proves ffmpeg actually ran)
    let metadata = std::fs::metadata(output_path).unwrap();
    assert!(
        metadata.len() > 1024,
        "MP4 file should be at least 1KB, got {} bytes", metadata.len()
    );

    // Clean up
    let _ = std::fs::remove_file(output_path);
}

#[test]
fn scene_new_rejects_odd_dimensions() {
    // H.264 requires even dimensions — Scene::new must catch this before ffmpeg
    let result = Scene::new(641, 480, 30);
    assert!(result.is_err(), "odd width should return Err");

    let result = Scene::new(640, 481, 30);
    assert!(result.is_err(), "odd height should return Err");
}

#[test]
fn scene_new_rejects_zero_fps() {
    let result = Scene::new(640, 480, 0);
    assert!(result.is_err(), "fps=0 should return Err");
}
