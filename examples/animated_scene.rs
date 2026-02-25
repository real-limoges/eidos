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

use eidos::{Easing, Scene, Tween};
use eidos::primitives::circle::CircleState;
use eidos::primitives::rect::RectState;
use eidos::primitives::line::LineState;
use eidos::primitives::text::TextState;

fn main() -> Result<(), eidos::EidosError> {
    let scene = Scene::new(1280, 720, 30)?.duration(3.0);

    scene.render(|s, t_secs| {
        // ANIM-01: circle moves right and changes color (EaseInOut)
        let circle_tween = Tween {
            start: CircleState {
                cx: 150.0, cy: 360.0, r: 100.0,
                fill_r: 255.0, fill_g: 0.0, fill_b: 0.0,
                opacity: 1.0,
            },
            end: CircleState {
                cx: 1130.0, cy: 360.0, r: 100.0,
                fill_r: 0.0, fill_g: 0.0, fill_b: 255.0,
                opacity: 1.0,
            },
            start_time: 0.0,
            duration: 3.0,
            easing: Easing::EaseInOut,
        };
        s.add(circle_tween.value_at(t_secs).to_circle());

        // ANIM-02: rect fades simultaneously (Linear) — parallel composition
        let rect_tween = Tween {
            start: RectState {
                x: 540.0, y: 260.0, width: 200.0, height: 200.0,
                fill_r: 255.0, fill_g: 255.0, fill_b: 255.0,
                opacity: 1.0,
            },
            end: RectState {
                x: 540.0, y: 260.0, width: 200.0, height: 200.0,
                fill_r: 255.0, fill_g: 255.0, fill_b: 255.0,
                opacity: 0.2,
            },
            start_time: 0.0,
            duration: 3.0,
            easing: Easing::Linear,
        };
        s.add(rect_tween.value_at(t_secs).to_rect());

        // LineState: a line that slides right and changes color from green to orange over 3 seconds
        let line_tween = Tween {
            start: LineState {
                x1: 100.0, y1: 600.0, x2: 400.0, y2: 600.0,
                stroke_r: 0.0, stroke_g: 255.0, stroke_b: 0.0,
                stroke_width: 4.0,
                opacity: 1.0,
            },
            end: LineState {
                x1: 700.0, y1: 600.0, x2: 1100.0, y2: 600.0,
                stroke_r: 255.0, stroke_g: 165.0, stroke_b: 0.0,
                stroke_width: 4.0,
                opacity: 1.0,
            },
            start_time: 0.0,
            duration: 3.0,
            easing: Easing::EaseIn,
        };
        s.add(line_tween.value_at(t_secs).to_line());

        // TextState: a label that grows in size and fades color from white to yellow over 3 seconds
        let text_tween = Tween {
            start: TextState {
                x: 640.0, y: 100.0, font_size: 24.0,
                fill_r: 255.0, fill_g: 255.0, fill_b: 255.0,
                opacity: 1.0,
            },
            end: TextState {
                x: 640.0, y: 100.0, font_size: 48.0,
                fill_r: 255.0, fill_g: 200.0, fill_b: 0.0,
                opacity: 0.5,
            },
            start_time: 0.0,
            duration: 3.0,
            easing: Easing::EaseOut,
        };
        s.add(text_tween.value_at(t_secs).to_text("Phase 2 - Animation"));
    }, "/tmp/animated_scene.mp4")?;

    println!("Rendered: /tmp/animated_scene.mp4");
    Ok(())
}
