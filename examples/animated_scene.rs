//! Animated scene example — demonstrates Phase 2 Tween-based animation.
//! Run with: cargo run --example animated_scene
//! Output: /tmp/animated_scene.mp4
//!
//! Shows:
//!   - Circle moving right and transitioning red -> blue (EaseInOut, 3s)
//!   - Rect fading opacity 1.0 -> 0.2 simultaneously (Linear, 3s) — ANIM-02 parallel
//!   - Line sliding right and changing color green -> orange (EaseIn, 3s)
//!   - Text label growing in size and fading color white -> yellow (EaseOut, 3s)
//!   - All four Easing variants are imported and available

use eidos::{CircleState, LineState, RectState, TextState};
use eidos::{Color, Easing, Scene, Tween};

fn main() -> Result<(), eidos::EidosError> {
    let scene = Scene::new(1280, 720, 30)?.duration(3.0);

    scene.render(
        |s, t_secs| {
            // ANIM-01: circle moves right and changes color (EaseInOut)
            let circle_tween = Tween::build(
                CircleState::new(150.0, 360.0, 100.0, Color::RED, 1.0),
                CircleState::new(1130.0, 360.0, 100.0, Color::BLUE, 1.0),
            )
            .over(3.0)
            .easing(Easing::EaseInOut)
            .build();
            s.add(circle_tween.value_at(t_secs).to_circle());

            // ANIM-02: rect fades simultaneously (Linear) — parallel composition
            let rect_tween = Tween::build(
                RectState::new(540.0, 260.0, 200.0, 200.0, Color::WHITE, 1.0),
                RectState::new(540.0, 260.0, 200.0, 200.0, Color::WHITE, 0.2),
            )
            .over(3.0)
            .build();
            s.add(rect_tween.value_at(t_secs).to_rect());

            // LineState: a line that slides right and changes color from green to orange over 3 seconds
            let line_tween = Tween::build(
                LineState::new(100.0, 600.0, 400.0, 600.0, Color::GREEN, 4.0, 1.0),
                LineState::new(
                    700.0,
                    600.0,
                    1100.0,
                    600.0,
                    Color::rgb(255, 165, 0),
                    4.0,
                    1.0,
                ),
            )
            .over(3.0)
            .easing(Easing::EaseIn)
            .build();
            s.add(line_tween.value_at(t_secs).to_line());

            // TextState: a label that grows in size and fades color from white to yellow over 3 seconds
            let text_tween = Tween::build(
                TextState::new(640.0, 100.0, 24.0, Color::WHITE, 1.0),
                TextState::new(640.0, 100.0, 48.0, Color::rgb(255, 200, 0), 0.5),
            )
            .over(3.0)
            .easing(Easing::EaseOut)
            .build();
            s.add(text_tween.value_at(t_secs).to_text("Phase 2 - Animation"));
        },
        "/tmp/animated_scene.mp4",
    )?;

    println!("Rendered: /tmp/animated_scene.mp4");
    Ok(())
}
