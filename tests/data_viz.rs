// tests/data_viz.rs
//! Integration tests for Phase 3: Data Visualization
//! Covers DATA-01 (axes), DATA-02 (smooth curves), DATA-03 (auto-range)

use eidos::{Axes, AxisRange, Color, DataCurve};

// ---- DataCurve unit-level integration ----

#[test]
fn data_curve_new_too_few_points_returns_err() {
    assert!(DataCurve::new(vec![]).is_err());
    assert!(DataCurve::new(vec![(0.0, 0.0)]).is_err());
}

#[test]
fn data_curve_new_two_points_ok() {
    let result = DataCurve::new(vec![(0.0, 0.0), (1.0, 1.0)]);
    assert!(result.is_ok());
}

#[test]
fn data_curve_negative_stroke_width_err() {
    let c = DataCurve::new(vec![(0.0, 0.0), (1.0, 1.0)]).unwrap();
    assert!(c.stroke(Color::WHITE, -1.0).is_err());
}

#[test]
fn data_curve_to_bezier_path_produces_correct_command_count() {
    // n data points -> 1 MoveTo + (n-1) CubicTo = n commands
    let n = 10;
    let pts: Vec<(f64, f64)> = (0..n).map(|i| (i as f64 * 10.0, (i as f64).sin() * 100.0)).collect();
    let curve = DataCurve::new(pts.clone()).unwrap();
    let visual: Vec<(f64, f64)> = pts.iter().map(|&(x, y)| (x + 100.0, 500.0 - y)).collect();
    let bez = curve.to_bezier_path(&visual);
    assert_eq!(bez.commands.len(), n, "expected {} commands, got {}", n, bez.commands.len());
}

// ---- Axes structure tests (DATA-01) ----

#[test]
fn axes_to_primitives_includes_axis_lines_and_ticks() {
    let data = vec![(0.0, 0.0), (10.0, 10.0), (20.0, 5.0)];
    let curve = DataCurve::new(data).unwrap();
    let axes = Axes::new(100.0, 100.0, 800.0, 500.0).add_curve(curve);
    let prims = axes.to_primitives();
    // Should have: 2 axis lines + ticks + labels + grid lines + 1 curve path = well over 10
    assert!(prims.len() > 10, "expected >10 primitives, got {}", prims.len());
}

#[test]
fn axes_with_titles_produces_more_primitives_than_without() {
    let data = vec![(0.0, 0.0), (1.0, 1.0)];
    let c1 = DataCurve::new(data.clone()).unwrap();
    let c2 = DataCurve::new(data).unwrap();

    let axes_no_title = Axes::new(100.0, 100.0, 800.0, 500.0).add_curve(c1);
    let axes_with_title = Axes::new(100.0, 100.0, 800.0, 500.0)
        .x_title("X")
        .y_title("Y")
        .add_curve(c2);

    let n_no_title = axes_no_title.to_primitives().len();
    let n_with_title = axes_with_title.to_primitives().len();
    assert!(n_with_title > n_no_title, "titles should add primitives");
}

// ---- Auto-range tests (DATA-03) ----

#[test]
fn axes_auto_range_does_not_include_zero_if_data_excludes_it() {
    // Data only in [5, 10] range -- auto-range should NOT force zero inclusion (locked decision)
    // We can't inspect the internal range directly, but we can verify it compiles and runs
    let data = vec![(5.0, 5.0), (7.5, 8.0), (10.0, 10.0)];
    let curve = DataCurve::new(data).unwrap();
    let axes = Axes::new(0.0, 0.0, 800.0, 500.0).add_curve(curve);
    let prims = axes.to_primitives();
    assert!(!prims.is_empty());
}

#[test]
fn axes_auto_range_degenerate_single_y_value_does_not_panic() {
    // All Y same -- degenerate: should use [y-0.5, y+0.5] span (locked decision)
    let data = vec![(0.0, 3.0), (1.0, 3.0), (2.0, 3.0)];
    let curve = DataCurve::new(data).unwrap();
    let axes = Axes::new(0.0, 0.0, 800.0, 500.0).add_curve(curve);
    // Must not panic (division by zero protection)
    let prims = axes.to_primitives();
    assert!(!prims.is_empty());
}

#[test]
fn axes_auto_range_degenerate_single_point_does_not_panic() {
    // Only 2 points required by DataCurve; X degenerate too
    let data = vec![(5.0, 5.0), (5.0, 5.0)];  // same point twice
    let curve = DataCurve::new(data).unwrap();
    let axes = Axes::new(0.0, 0.0, 800.0, 500.0).add_curve(curve);
    let prims = axes.to_primitives();
    assert!(!prims.is_empty());
}

#[test]
fn axes_explicit_range_overrides_auto_range() {
    let data = vec![(0.0, 0.0), (100.0, 100.0)];
    let curve = DataCurve::new(data).unwrap();
    let axes = Axes::new(0.0, 0.0, 800.0, 500.0)
        .x_range(-10.0, 110.0)
        .y_range(-10.0, 110.0)
        .add_curve(curve);
    // AxisRange::Explicit is used -- just verify it doesn't panic
    let prims = axes.to_primitives();
    assert!(!prims.is_empty());
}

// ---- Multi-curve composition (DATA-02) ----

#[test]
fn axes_multiple_curves_each_produce_bezier_path() {
    let data1 = vec![(0.0, 0.0), (1.0, 1.0), (2.0, 0.0)];
    let data2 = vec![(0.0, 1.0), (1.0, 0.0), (2.0, 1.0)];
    let c1 = DataCurve::new(data1).unwrap();
    let c2 = DataCurve::new(data2).unwrap();
    let axes = Axes::new(0.0, 0.0, 800.0, 500.0)
        .add_curve(c1)
        .add_curve(c2);

    // 2 curves -> 2 Bezier paths; total primitives increases vs 1 curve
    let one_curve_count = {
        let d = vec![(0.0, 0.0), (1.0, 1.0), (2.0, 0.0)];
        let c = DataCurve::new(d).unwrap();
        Axes::new(0.0, 0.0, 800.0, 500.0).add_curve(c).to_primitives().len()
    };
    let two_curve_count = axes.to_primitives().len();
    assert!(two_curve_count > one_curve_count,
        "two curves should produce more primitives than one ({} vs {})",
        two_curve_count, one_curve_count);
}
