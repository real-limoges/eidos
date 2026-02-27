// examples/gam_fitting.rs
//
// In-depth GAM fitting walkthrough — 30-second animated example.
// Run with: cargo run --example gam_fitting
// Output: /tmp/gam_fitting.mp4
//
// Shows the story of fitting a GAM model:
//   1. Title card
//   2. Axes appear
//   3. Raw scatter points appear one-by-one
//   4. Mean line sweeps across
//   5. SplineFit reveals the fitted curve
//   6. Confidence band fades in
//   7. True function overlays for comparison
//   8. Hold final composition

use eidos::{
    Axes, Circle, Color, ConfidenceBand, Easing, Line, Scene, SplineFit, Text, TextState, Tween,
};

/// Simple LCG pseudo-random number generator (deterministic, no external crate).
struct Lcg {
    state: u64,
}

impl Lcg {
    fn new(seed: u64) -> Self {
        Lcg { state: seed }
    }

    /// Returns a value in [0.0, 1.0).
    fn next_f64(&mut self) -> f64 {
        // Numerical Recipes LCG parameters
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (self.state >> 33) as f64 / (1u64 << 31) as f64
    }

    /// Approximate Gaussian via Box-Muller (pairs of uniforms → normal).
    fn next_gaussian(&mut self) -> f64 {
        let u1 = self.next_f64().max(1e-10); // avoid log(0)
        let u2 = self.next_f64();
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
    }
}

/// The true underlying function: f(x) = sin(x) + 0.3 * cos(2x)
fn true_fn(x: f64) -> f64 {
    x.sin() + 0.3 * (2.0 * x).cos()
}

/// Confidence band half-width: wider at edges, narrower in center.
fn band_width(x: f64) -> f64 {
    // Parabolic shape: wider at x=0 and x=10, narrow at x=5
    let center = 5.0;
    let t = ((x - center) / center).abs(); // 0 at center, 1 at edges
    0.15 + 0.20 * t * t
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- Data generation ---
    let num_points = 40;
    let mut rng = Lcg::new(42);

    // Noisy observations
    let scatter_data: Vec<(f64, f64)> = (0..num_points)
        .map(|i| {
            let x = i as f64 * 10.0 / (num_points - 1) as f64;
            let y = true_fn(x) + 0.4 * rng.next_gaussian();
            (x, y)
        })
        .collect();

    // Fitted curve points (dense, using the true function as "what GAM recovers")
    let n_curve = 50;
    let fitted: Vec<(f64, f64)> = (0..=n_curve)
        .map(|i| {
            let x = i as f64 * 10.0 / n_curve as f64;
            (x, true_fn(x))
        })
        .collect();

    // Confidence band upper/lower
    let upper: Vec<(f64, f64)> = fitted.iter().map(|&(x, y)| (x, y + band_width(x))).collect();
    let lower: Vec<(f64, f64)> = fitted.iter().map(|&(x, y)| (x, y - band_width(x))).collect();

    // Mean of observed y-values (for the mean line)
    let mean_y: f64 = scatter_data.iter().map(|(_, y)| y).sum::<f64>() / scatter_data.len() as f64;

    // --- Axes setup ---
    let axes = Axes::new(100.0, 60.0, 1100.0, 600.0)
        .x_range(0.0, 10.0)
        .y_range(-1.8, 2.0)
        .x_title("x")
        .y_title("y");

    // --- Pre-map all points to visual space ---
    let scatter_visual: Vec<(f64, f64)> = scatter_data
        .iter()
        .map(|&(x, y)| axes.map_point(x, y))
        .collect();

    let fitted_visual: Vec<(f64, f64)> =
        fitted.iter().map(|&(x, y)| axes.map_point(x, y)).collect();

    let upper_visual: Vec<(f64, f64)> =
        upper.iter().map(|&(x, y)| axes.map_point(x, y)).collect();

    let lower_visual: Vec<(f64, f64)> =
        lower.iter().map(|&(x, y)| axes.map_point(x, y)).collect();

    // Mean line endpoints in visual space
    let mean_left = axes.map_point(0.0, mean_y);
    let mean_right = axes.map_point(10.0, mean_y);

    // --- SplineFit: fitted curve animating from 10.0s to 16.0s ---
    let spline_fit = SplineFit::new(fitted.clone())?
        .color(Color::rgb(255, 200, 50)) // warm yellow
        .stroke_width(2.5)
        .animate_fit(10.0, 6.0, Easing::EaseInOut);

    // --- SplineFit: true function overlay animating from 22.0s to 26.0s ---
    let spline_true = SplineFit::new(fitted.clone())?
        .color(Color::rgb(100, 255, 100)) // green
        .stroke_width(2.0)
        .animate_fit(22.0, 4.0, Easing::EaseInOut);

    // --- ConfidenceBand ---
    let band = ConfidenceBand::new(upper, lower)?
        .fill_color(Color::rgb(100, 149, 237));

    // --- Title tween: fade in 0.0→1.0s, hold 1.0→2.0s, fade out 2.0→3.0s ---
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

    scene.render(
        move |s, t| {
            // --- Phase: Title (0.0–3.0s) ---
            if t < 3.0 {
                let state = if t < 2.0 {
                    title_in.value_at(t)
                } else {
                    title_out.value_at(t)
                };
                let title = state
                    .to_text("GAM Model Fitting")
                    .alignment(eidos::primitives::text::Alignment::Center);
                s.add(title);
            }

            // --- Phase: Axes appear at t >= 2.5s (fade overlap with title out) ---
            if t >= 2.5 {
                s.add_axes(&axes);
            }

            // --- Phase: Scatter points appear (3.0–7.0s) ---
            if t >= 3.0 {
                let scatter_color = Color::rgb(180, 200, 255);
                for (i, &(vx, vy)) in scatter_visual.iter().enumerate() {
                    let t_appear = 3.0 + (i as f64 / scatter_visual.len() as f64) * 4.0;
                    if t >= t_appear {
                        let opacity = ((t - t_appear) / 0.2).clamp(0.0, 0.7);
                        s.add(
                            Circle::new(vx, vy, 4.0)
                                .fill(scatter_color)
                                .opacity(opacity),
                        );
                    }
                }
            }

            // --- Phase: "Raw observations" label (7.0–9.0s) ---
            if t >= 7.0 && t < 10.0 {
                let label_opacity = ((t - 7.0) / 0.5).clamp(0.0, 1.0);
                s.add(
                    Text::new(640.0, 40.0, "Raw observations")
                        .fill(Color::rgb(180, 200, 255))
                        .font_size(20.0)
                        .alignment(eidos::primitives::text::Alignment::Center)
                        .opacity(label_opacity),
                );
            }

            // --- Phase: Mean line sweep (8.0–9.0s) ---
            if t >= 8.0 {
                let sweep_progress = ((t - 8.0) / 1.0).clamp(0.0, 1.0);
                let current_x2 = mean_left.0 + sweep_progress * (mean_right.0 - mean_left.0);
                s.add(
                    Line::new(mean_left.0, mean_left.1, current_x2, mean_left.1)
                        .stroke_color(Color::rgb(200, 200, 200))
                        .stroke_width(1.5)
                        .opacity(0.6),
                );

                // "Overall mean" label
                if t >= 8.5 {
                    let label_opacity = ((t - 8.5) / 0.5).clamp(0.0, 1.0);
                    s.add(
                        Text::new(mean_right.0 + 10.0, mean_left.1 + 5.0, "Overall mean")
                            .fill(Color::rgb(200, 200, 200))
                            .font_size(14.0)
                            .opacity(label_opacity),
                    );
                }
            }

            // --- Phase: SplineFit fitted curve (10.0–16.0s) ---
            if t >= 10.0 {
                if let Some(bez) = spline_fit.to_bezier(&fitted_visual, t) {
                    s.add(bez);
                }

                // "Fitted smooth" label
                if t >= 11.0 {
                    let label_opacity = ((t - 11.0) / 0.5).clamp(0.0, 1.0);
                    s.add(
                        Text::new(640.0, 40.0, "Fitted smooth")
                            .fill(Color::rgb(255, 200, 50))
                            .font_size(20.0)
                            .alignment(eidos::primitives::text::Alignment::Center)
                            .opacity(label_opacity),
                    );
                }
            }

            // --- Phase: Confidence band fade-in (17.0–21.0s) ---
            if t >= 17.0 {
                let band_opacity = ((t - 17.0) / 4.0).clamp(0.0, 0.25);
                let frame_band = band.to_bezier_path(&upper_visual, &lower_visual);
                // Override the default opacity with our animated value
                let frame_band = frame_band.opacity(band_opacity);
                s.add(frame_band);

                // "95% confidence band" label
                if t >= 18.0 {
                    let label_opacity = ((t - 18.0) / 0.5).clamp(0.0, 1.0);
                    s.add(
                        Text::new(640.0, 40.0, "95% confidence band")
                            .fill(Color::rgb(100, 149, 237))
                            .font_size(20.0)
                            .alignment(eidos::primitives::text::Alignment::Center)
                            .opacity(label_opacity),
                    );
                }
            }

            // --- Phase: True function overlay (22.0–26.0s) ---
            if t >= 22.0 {
                if let Some(bez) = spline_true.to_bezier(&fitted_visual, t) {
                    s.add(bez);
                }

                // "True function" label
                if t >= 23.0 {
                    let label_opacity = ((t - 23.0) / 0.5).clamp(0.0, 1.0);
                    s.add(
                        Text::new(640.0, 40.0, "True function")
                            .fill(Color::rgb(100, 255, 100))
                            .font_size(20.0)
                            .alignment(eidos::primitives::text::Alignment::Center)
                            .opacity(label_opacity),
                    );
                }
            }

            // --- Phase: Hold (26.0–30.0s) — everything stays visible ---
        },
        "/tmp/gam_fitting.mp4",
    )?;

    println!("Rendered: /tmp/gam_fitting.mp4");
    Ok(())
}
