// examples/surface_fitting.rs
//
// 3D surface fitting walkthrough — 30-second animated example.
// Run with: cargo run --example surface_fitting
// Output: /tmp/surface_fitting.mp4
//
// Shows:
//   1. Title card fades in
//   2. Surface morphs from flat plane to a fitted response surface
//   3. Camera orbits 360° around the surface
//   4. Noisy scatter observations fade in over the surface
//   5. Full composition holds while camera continues rotating

use eidos::{
    Camera, Color, Easing, RenderMode, Scene, ScatterPlot, SurfacePlot, Text, TextState, Tween,
};

/// Simple LCG pseudo-random number generator (deterministic, no external crate).
struct Lcg {
    state: u64,
}

impl Lcg {
    fn new(seed: u64) -> Self {
        Lcg { state: seed }
    }

    fn next_f64(&mut self) -> f64 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (self.state >> 33) as f64 / (1u64 << 31) as f64
    }

    fn next_gaussian(&mut self) -> f64 {
        let u1 = self.next_f64().max(1e-10);
        let u2 = self.next_f64();
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
    }
}

/// Response surface: z = sin(x) * cos(y) + 0.3 * sin(x * y / 3)
fn surface_fn(x: f64, y: f64) -> f64 {
    x.sin() * y.cos() + 0.3 * (x * y / 3.0).sin()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- Surface grid: 30x30 over [-3, 3] x [-3, 3] ---
    let rows = 30;
    let cols = 30;
    let n = rows * cols;

    let mut xs = Vec::with_capacity(n);
    let mut ys = Vec::with_capacity(n);
    let mut zs = Vec::with_capacity(n);

    for r in 0..rows {
        for c in 0..cols {
            let x = -3.0 + 6.0 * c as f64 / (cols - 1) as f64;
            let y = -3.0 + 6.0 * r as f64 / (rows - 1) as f64;
            xs.push(x);
            ys.push(y);
            zs.push(surface_fn(x, y));
        }
    }

    // --- SurfacePlot with morph animation and camera orbit ---
    let surface = SurfacePlot::new(xs, ys, zs, rows, cols)
        .render_mode(RenderMode::ShadedWireframe)
        .x_label("X")
        .y_label("Y")
        .z_label("Z")
        .show_base_grid(true)
        .animate_fit(3.0, 5.0, Easing::EaseInOut)
        .animate_camera_azimuth(0.0, 30.0, 45.0, 405.0, Easing::Linear);

    // --- Scatter: 60 noisy observations on the surface ---
    let mut rng = Lcg::new(7);
    let scatter_points: Vec<(f64, f64, f64)> = (0..60)
        .map(|_| {
            let x = -3.0 + 6.0 * rng.next_f64();
            let y = -3.0 + 6.0 * rng.next_f64();
            let z = surface_fn(x, y) + 0.3 * rng.next_gaussian();
            (x, y, z)
        })
        .collect();

    let scatter = ScatterPlot::new(scatter_points, surface.data_extents())
        .with_color(Color::rgb(255, 120, 50))
        .with_radius(5.0)
        .animate_fade(12.0, 16.0);

    // --- Title tweens ---
    let title_in = Tween::build(
        TextState::new(640.0, 340.0, 42.0, Color::WHITE, 0.0),
        TextState::new(640.0, 340.0, 42.0, Color::WHITE, 1.0),
    )
    .start_at(0.0)
    .over(1.0)
    .easing(Easing::EaseOut)
    .build();

    let title_out = Tween::build(
        TextState::new(640.0, 340.0, 42.0, Color::WHITE, 1.0),
        TextState::new(640.0, 340.0, 42.0, Color::WHITE, 0.0),
    )
    .start_at(2.0)
    .over(1.0)
    .easing(Easing::EaseIn)
    .build();

    // --- Scene: 1280x720, 30fps, 30 seconds ---
    let scene = Scene::new(1280, 720, 30)?.duration(30.0);
    let viewport = (1280u32, 720u32);

    scene.render(
        move |s, t| {
            // --- Title (0.0–3.0s) ---
            if t < 3.0 {
                let state = if t < 2.0 {
                    title_in.value_at(t)
                } else {
                    title_out.value_at(t)
                };
                let title = state
                    .to_text("3D Surface Fitting")
                    .alignment(eidos::primitives::text::Alignment::Center);
                s.add(title);
            }

            // --- Surface + scatter (from t=2.5s onward) ---
            if t >= 2.5 {
                let azimuth = surface.camera_at(t).unwrap_or(45.0);
                let camera = Camera::new(azimuth, 30.0, 3.0);

                s.add_surface_at(&surface, &camera, viewport, t);

                // Scatter appears from t=12s via animate_fade
                if t >= 12.0 {
                    s.add_scatter_at(&scatter, &camera, viewport, t);
                }

                // Phase labels
                if t >= 3.0 && t < 9.0 {
                    let label_opacity = if t < 4.0 {
                        (t - 3.0).clamp(0.0, 1.0)
                    } else if t > 8.0 {
                        (9.0 - t).clamp(0.0, 1.0)
                    } else {
                        1.0
                    };
                    s.add(
                        Text::new(640.0, 30.0, "Surface morphing from flat")
                            .fill(Color::rgb(180, 220, 255))
                            .font_size(18.0)
                            .alignment(eidos::primitives::text::Alignment::Center)
                            .opacity(label_opacity),
                    );
                }

                if t >= 12.0 && t < 18.0 {
                    let label_opacity = if t < 13.0 {
                        (t - 12.0).clamp(0.0, 1.0)
                    } else if t > 17.0 {
                        (18.0 - t).clamp(0.0, 1.0)
                    } else {
                        1.0
                    };
                    s.add(
                        Text::new(640.0, 30.0, "Noisy observations")
                            .fill(Color::rgb(255, 160, 80))
                            .font_size(18.0)
                            .alignment(eidos::primitives::text::Alignment::Center)
                            .opacity(label_opacity),
                    );
                }
            }
        },
        "/tmp/surface_fitting.mp4",
    )?;

    println!("Rendered: /tmp/surface_fitting.mp4");
    Ok(())
}
