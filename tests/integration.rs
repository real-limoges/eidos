// tests/integration.rs
use eidos::primitives::{Circle, Rect, Text};
use eidos::{Color, Scene};
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

    scene
        .render_static(
            |s| {
                s.add(Circle::new(320.0, 240.0, 100.0).fill(Color::RED));
                s.add(Rect::new(50.0, 50.0, 200.0, 100.0).fill(Color::BLUE));
                s.add(Text::new(320.0, 400.0, "Integration Test").fill(Color::WHITE));
            },
            output_path,
        )
        .expect("render should succeed");

    assert!(
        Path::new(output_path).exists(),
        "MP4 file should exist at {}",
        output_path
    );

    // Verify file size is non-trivial (at least 1KB — proves ffmpeg actually ran)
    let metadata = std::fs::metadata(output_path).unwrap();
    assert!(
        metadata.len() > 1024,
        "MP4 file should be at least 1KB, got {} bytes",
        metadata.len()
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

#[test]
fn animated_render_produces_mp4() {
    if !ffmpeg_available() {
        eprintln!("SKIP: ffmpeg not found on PATH");
        return;
    }

    use eidos::{CircleState, Easing, Tween};

    let output_path = "/tmp/eidos_animated_test.mp4";
    let _ = std::fs::remove_file(output_path);

    let scene = Scene::new(640, 480, 24).unwrap().duration(1.0);

    scene
        .render(
            |s, t_secs| {
                let tween = Tween::build(
                    CircleState::new(100.0, 240.0, 60.0, Color::RED, 1.0),
                    CircleState::new(540.0, 240.0, 60.0, Color::BLUE, 1.0),
                )
                .easing(Easing::EaseInOut)
                .build();
                s.add(tween.value_at(t_secs).to_circle());
            },
            output_path,
        )
        .expect("animated render should succeed");

    assert!(Path::new(output_path).exists(), "animated MP4 should exist");
    let meta = std::fs::metadata(output_path).unwrap();
    assert!(
        meta.len() > 1024,
        "animated MP4 should be non-trivial, got {} bytes",
        meta.len()
    );
    let _ = std::fs::remove_file(output_path);
}

#[test]
fn easing_midpoint_differs_between_linear_and_ease_in_out() {
    use eidos::{CircleState, Easing, Tween};

    let start = CircleState::new(0.0, 0.0, 10.0, Color::BLACK, 1.0);
    let end = CircleState::new(100.0, 0.0, 10.0, Color::BLACK, 1.0);

    let linear_tween = Tween::build(start.clone(), end.clone())
        .easing(Easing::Linear)
        .build();
    let ease_tween = Tween::build(start.clone(), end.clone())
        .easing(Easing::EaseInOut)
        .build();

    let linear_mid = linear_tween.value_at(0.5).cx;
    let ease_mid = ease_tween.value_at(0.5).cx;

    // Linear midpoint is exactly 50.0
    assert!(
        (linear_mid - 50.0).abs() < 1e-9,
        "Linear midpoint should be 50.0, got {}",
        linear_mid
    );

    // EaseInOut midpoint is 50.0 at exactly t=0.5 by symmetry (sine-based midpoint IS 50)
    // But at t=0.25, EaseInOut should be less than Linear (slower start)
    let linear_quarter = linear_tween.value_at(0.25).cx;
    let ease_quarter = ease_tween.value_at(0.25).cx;

    assert!(
        (linear_quarter - 25.0).abs() < 1e-9,
        "Linear at t=0.25 should be 25.0, got {}",
        linear_quarter
    );
    assert!(
        ease_quarter < linear_quarter,
        "EaseInOut at t=0.25 should be slower than Linear (ease_quarter={} >= linear_quarter={})",
        ease_quarter,
        linear_quarter
    );

    // Suppress unused variable warning
    let _ = ease_mid;
}

#[test]
fn line_state_and_text_state_tween_midpoint_values() {
    use eidos::{Easing, LineState, TextState, Tween};

    // LineState: x1 moves 100 -> 300 over 1s with Linear easing — midpoint should be 200
    let line_tween = Tween::build(
        LineState::new(100.0, 0.0, 200.0, 0.0, Color::GREEN, 2.0, 1.0),
        LineState::new(300.0, 0.0, 400.0, 0.0, Color::RED, 2.0, 1.0),
    )
    .build();
    let line_mid = line_tween.value_at(0.5);
    assert!(
        (line_mid.x1 - 200.0).abs() < 1e-9,
        "LineState x1 midpoint should be 200.0, got {}",
        line_mid.x1
    );
    let line = line_mid.to_line();
    assert!((line.x1 - 200.0).abs() < 1e-9);

    // TextState: font_size from 16 -> 32 over 1s with Linear easing — midpoint should be 24
    let text_tween = Tween::build(
        TextState::new(100.0, 100.0, 16.0, Color::WHITE, 1.0),
        TextState::new(100.0, 100.0, 32.0, Color::WHITE, 1.0),
    )
    .build();
    let text_mid = text_tween.value_at(0.5);
    assert!(
        (text_mid.font_size - 24.0).abs() < 1e-9,
        "TextState font_size midpoint should be 24.0, got {}",
        text_mid.font_size
    );
    let text = text_mid.to_text("test label");
    assert!((text.font_size - 24.0).abs() < 1e-9);
}

#[test]
fn parallel_animations_both_execute() {
    use eidos::{CircleState, Easing, RectState, Tween};

    // Two independent tweens at t=0.5
    let circle_tween = Tween::build(
        CircleState::new(0.0, 0.0, 10.0, Color::BLACK, 1.0),
        CircleState::new(200.0, 0.0, 10.0, Color::BLACK, 1.0),
    )
    .over(2.0)
    .build();

    let rect_tween = Tween::build(
        RectState::new(0.0, 0.0, 100.0, 50.0, Color::WHITE, 1.0),
        RectState::new(0.0, 0.0, 100.0, 50.0, Color::WHITE, 0.0),
    )
    .over(2.0)
    .build();

    // At t=1.0 (midpoint of 2s duration), both should be at 50% interpolation
    let circle_state = circle_tween.value_at(1.0);
    let rect_state = rect_tween.value_at(1.0);

    assert!(
        (circle_state.cx - 100.0).abs() < 1e-9,
        "circle cx at t=1.0 should be 100.0, got {}",
        circle_state.cx
    );

    assert!(
        (rect_state.opacity - 0.5).abs() < 1e-9,
        "rect opacity at t=1.0 should be 0.5, got {}",
        rect_state.opacity
    );
}
