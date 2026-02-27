//! Basic scene example — demonstrates all Phase 1 primitive types.
//! Run with: cargo run --example basic_scene
//! Output: /tmp/basic_scene.mp4

use eidos::primitives::{Arrow, Bezier, Circle, Line, Rect, Text, text::Alignment};
use eidos::{Color, Scene};

fn main() -> Result<(), eidos::EidosError> {
    let scene = Scene::new(1920, 1080, 30)?.duration(2.0);

    scene.render_static(
        |s| {
            // Circle: center at (300, 300), radius 150, red fill, white stroke
            s.add(
                Circle::new(300.0, 300.0, 150.0)
                    .fill(Color::RED)
                    .stroke(Color::WHITE, 3.0),
            );

            // Rect: top-left (600, 150), 300x300, blue fill, 80% opacity
            s.add(
                Rect::new(600.0, 150.0, 300.0, 300.0)
                    .fill(Color::BLUE)
                    .opacity(0.8),
            );

            // Line: diagonal from (1000, 100) to (1200, 400), green, width 4
            s.add(
                Line::new(1000.0, 100.0, 1200.0, 400.0)
                    .stroke_color(Color::GREEN)
                    .stroke_width(4.0),
            );

            // Arrow: horizontal arrow from (1300, 250) to (1600, 250), yellow
            s.add(
                Arrow::new(1300.0, 250.0, 1600.0, 250.0)
                    .stroke_color(Color::YELLOW)
                    .stroke_width(3.0),
            );

            // Text: multi-line, white, 36px, centered
            s.add(
                Text::new(960.0, 600.0, "Eidos\nRendering Pipeline\nPhase 1")
                    .font_size(36.0)
                    .alignment(Alignment::Center),
            );

            // BezierPath: S-curve from (200, 700) to (800, 900)
            s.add(
                Bezier::new()
                    .move_to(200.0, 700.0)
                    .cubic_to(400.0, 600.0, 600.0, 1000.0, 800.0, 900.0)
                    .stroke(Color::CYAN, 4.0),
            );
        },
        "/tmp/basic_scene.mp4",
    )?;

    println!("Rendered: /tmp/basic_scene.mp4");
    Ok(())
}
