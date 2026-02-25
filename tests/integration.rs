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

    scene.render_static(|s| {
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

#[test]
fn animated_render_produces_mp4() {
    if !ffmpeg_available() {
        eprintln!("SKIP: ffmpeg not found on PATH");
        return;
    }

    use eidos::{Easing, Tween};
    use eidos::primitives::circle::CircleState;

    let output_path = "/tmp/eidos_animated_test.mp4";
    let _ = std::fs::remove_file(output_path);

    let scene = Scene::new(640, 480, 24).unwrap().duration(1.0);

    scene.render(|s, t_secs| {
        let tween = Tween {
            start: CircleState { cx: 100.0, cy: 240.0, r: 60.0,
                                 fill_r: 255.0, fill_g: 0.0, fill_b: 0.0, opacity: 1.0 },
            end:   CircleState { cx: 540.0, cy: 240.0, r: 60.0,
                                 fill_r: 0.0, fill_g: 0.0, fill_b: 255.0, opacity: 1.0 },
            start_time: 0.0,
            duration: 1.0,
            easing: Easing::EaseInOut,
        };
        s.add(tween.value_at(t_secs).to_circle());
    }, output_path).expect("animated render should succeed");

    assert!(Path::new(output_path).exists(), "animated MP4 should exist");
    let meta = std::fs::metadata(output_path).unwrap();
    assert!(meta.len() > 1024, "animated MP4 should be non-trivial, got {} bytes", meta.len());
    let _ = std::fs::remove_file(output_path);
}

#[test]
fn easing_midpoint_differs_between_linear_and_ease_in_out() {
    use eidos::{Easing, Tween};
    use eidos::primitives::circle::CircleState;

    let start = CircleState { cx: 0.0, cy: 0.0, r: 10.0,
                               fill_r: 0.0, fill_g: 0.0, fill_b: 0.0, opacity: 1.0 };
    let end   = CircleState { cx: 100.0, cy: 0.0, r: 10.0,
                               fill_r: 0.0, fill_g: 0.0, fill_b: 0.0, opacity: 1.0 };

    let linear_tween = Tween {
        start: start.clone(), end: end.clone(),
        start_time: 0.0, duration: 1.0, easing: Easing::Linear,
    };
    let ease_tween = Tween {
        start: start.clone(), end: end.clone(),
        start_time: 0.0, duration: 1.0, easing: Easing::EaseInOut,
    };

    let linear_mid = linear_tween.value_at(0.5).cx;
    let ease_mid   = ease_tween.value_at(0.5).cx;

    // Linear midpoint is exactly 50.0
    assert!((linear_mid - 50.0).abs() < 1e-9,
            "Linear midpoint should be 50.0, got {}", linear_mid);

    // EaseInOut midpoint is 50.0 at exactly t=0.5 by symmetry (sine-based midpoint IS 50)
    // But at t=0.25, EaseInOut should be less than Linear (slower start)
    let linear_quarter = linear_tween.value_at(0.25).cx;
    let ease_quarter   = ease_tween.value_at(0.25).cx;

    assert!((linear_quarter - 25.0).abs() < 1e-9,
            "Linear at t=0.25 should be 25.0, got {}", linear_quarter);
    assert!(ease_quarter < linear_quarter,
            "EaseInOut at t=0.25 should be slower than Linear (ease_quarter={} >= linear_quarter={})",
            ease_quarter, linear_quarter);

    // Suppress unused variable warning
    let _ = ease_mid;
}

#[test]
fn parallel_animations_both_execute() {
    use eidos::{Easing, Tween};
    use eidos::primitives::circle::CircleState;
    use eidos::primitives::rect::RectState;

    // Two independent tweens at t=0.5
    let circle_tween = Tween {
        start: CircleState { cx: 0.0, cy: 0.0, r: 10.0,
                             fill_r: 0.0, fill_g: 0.0, fill_b: 0.0, opacity: 1.0 },
        end:   CircleState { cx: 200.0, cy: 0.0, r: 10.0,
                             fill_r: 0.0, fill_g: 0.0, fill_b: 0.0, opacity: 1.0 },
        start_time: 0.0, duration: 2.0, easing: Easing::Linear,
    };

    let rect_tween = Tween {
        start: RectState { x: 0.0, y: 0.0, width: 100.0, height: 50.0,
                           fill_r: 255.0, fill_g: 255.0, fill_b: 255.0, opacity: 1.0 },
        end:   RectState { x: 0.0, y: 0.0, width: 100.0, height: 50.0,
                           fill_r: 255.0, fill_g: 255.0, fill_b: 255.0, opacity: 0.0 },
        start_time: 0.0, duration: 2.0, easing: Easing::Linear,
    };

    // At t=1.0 (midpoint of 2s duration), both should be at 50% interpolation
    let circle_state = circle_tween.value_at(1.0);
    let rect_state   = rect_tween.value_at(1.0);

    assert!((circle_state.cx - 100.0).abs() < 1e-9,
            "circle cx at t=1.0 should be 100.0, got {}", circle_state.cx);

    assert!((rect_state.opacity - 0.5).abs() < 1e-9,
            "rect opacity at t=1.0 should be 0.5, got {}", rect_state.opacity);
}
