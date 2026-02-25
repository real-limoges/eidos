//! Animated scene example — demonstrates Phase 2 Tween-based animation.
//! Run with: cargo run --example animated_scene
//! Output: /tmp/animated_scene.mp4
//!
//! Shows:
//!   - Circle moving right and transitioning red -> blue (EaseInOut, 3s)
//!   - Rect fading opacity 1.0 -> 0.2 simultaneously (Linear, 3s) — ANIM-02 parallel
//!   - All four Easing variants are imported and available

use eidos::{Easing, Scene, Tween};
use eidos::primitives::circle::CircleState;
use eidos::primitives::rect::RectState;

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
    }, "/tmp/animated_scene.mp4")?;

    println!("Rendered: /tmp/animated_scene.mp4");
    Ok(())
}
