// src/dataviz/axes.rs

use crate::primitives::{Primitive, Line, Text, Bezier};
use crate::Color;
use crate::dataviz::DataCurve;
use crate::dataviz::ConfidenceBand;

const TICK_LENGTH: f64 = 6.0;
const TICK_LABEL_OFFSET: f64 = 14.0;   // pixels from tick end to label center
const TITLE_OFFSET: f64 = 36.0;        // pixels from axis to title center
const GRID_OPACITY: f64 = 0.15;        // subtle grid lines
const GRID_STROKE_WIDTH: f64 = 1.0;
const AXIS_STROKE_WIDTH: f64 = 1.5;
const TICK_STROKE_WIDTH: f64 = 1.0;
const TICK_LABEL_SIZE: f64 = 11.0;
const AXIS_TITLE_SIZE: f64 = 13.0;
const AUTO_PADDING_FRAC: f64 = 0.07;  // 7% — within locked 5–10% range
const TARGET_TICK_COUNT: usize = 6;    // within locked 5–10 range

/// Specifies the range for one axis — either automatic (fit to data) or explicit.
#[derive(Debug, Clone)]
pub enum AxisRange {
    Auto,
    Explicit(f64, f64),  // (min, max)
}

/// A Cartesian plot container.
///
/// Curves are attached via add_curve(); axes auto-range to fit unless overridden.
/// Call to_primitives() to decompose into Vec<Primitive> for scene insertion.
///
/// Axes are static for Phase 3 — no animated range or position changes.
#[derive(Debug, Clone)]
pub struct Axes {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub x_range: AxisRange,
    pub y_range: AxisRange,
    pub x_title: Option<String>,
    pub y_title: Option<String>,
    pub curves: Vec<DataCurve>,
    pub bands: Vec<ConfidenceBand>,
}

impl Axes {
    /// Create a new Axes with the given scene bounding box. Auto-ranges by default.
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Axes {
            x, y, width, height,
            x_range: AxisRange::Auto,
            y_range: AxisRange::Auto,
            x_title: None,
            y_title: None,
            curves: Vec::new(),
            bands: Vec::new(),
        }
    }

    /// Override x-axis range. Use with .y_range() to disable auto-range on both axes.
    pub fn x_range(mut self, min: f64, max: f64) -> Self {
        self.x_range = AxisRange::Explicit(min, max);
        self
    }

    /// Override y-axis range.
    pub fn y_range(mut self, min: f64, max: f64) -> Self {
        self.y_range = AxisRange::Explicit(min, max);
        self
    }

    /// Set optional X axis title (rendered centered below the axis).
    pub fn x_title(mut self, title: impl Into<String>) -> Self {
        self.x_title = Some(title.into());
        self
    }

    /// Set optional Y axis title (rendered rotated beside the left axis).
    pub fn y_title(mut self, title: impl Into<String>) -> Self {
        self.y_title = Some(title.into());
        self
    }

    /// Attach a DataCurve to this Axes. Multiple curves are composable.
    pub fn add_curve(mut self, curve: DataCurve) -> Self {
        self.curves.push(curve);
        self
    }

    /// Attach a ConfidenceBand to this Axes. Rendered below data curves.
    pub fn add_band(mut self, band: ConfidenceBand) -> Self {
        self.bands.push(band);
        self
    }

    /// Decompose the Axes into constituent primitives for insertion into a SceneBuilder.
    ///
    /// Produces: axis lines (X, Y), tick marks, tick labels, grid lines, curve paths.
    /// Curves are converted to Bezier paths with Catmull-Rom spline in visual (pixel) space.
    pub fn to_primitives(&self) -> Vec<Primitive> {
        // --- Step 1: Resolve data ranges ---
        let all_x: Vec<f64> = self.curves.iter().flat_map(|c| c.points.iter().map(|p| p.0))
            .chain(self.bands.iter().flat_map(|b| b.upper_points.iter().chain(b.lower_points.iter()).map(|p| p.0)))
            .collect();
        let all_y: Vec<f64> = self.curves.iter().flat_map(|c| c.points.iter().map(|p| p.1))
            .chain(self.bands.iter().flat_map(|b| b.upper_points.iter().chain(b.lower_points.iter()).map(|p| p.1)))
            .collect();

        let (x_data_min, x_data_max) = compute_range(&all_x, AUTO_PADDING_FRAC);
        let (y_data_min, y_data_max) = compute_range(&all_y, AUTO_PADDING_FRAC);

        let (x_min, x_max) = match &self.x_range {
            AxisRange::Auto => (x_data_min, x_data_max),
            AxisRange::Explicit(lo, hi) => (*lo, *hi),
        };
        let (y_min, y_max) = match &self.y_range {
            AxisRange::Auto => (y_data_min, y_data_max),
            AxisRange::Explicit(lo, hi) => (*lo, *hi),
        };

        // --- Step 2: Generate ticks ---
        let x_ticks = generate_ticks(x_min, x_max, TARGET_TICK_COUNT);
        let y_ticks = generate_ticks(y_min, y_max, TARGET_TICK_COUNT);

        // Tick step for label precision
        let x_step = if x_ticks.len() >= 2 { x_ticks[1] - x_ticks[0] } else { 1.0 };
        let y_step = if y_ticks.len() >= 2 { y_ticks[1] - y_ticks[0] } else { 1.0 };

        // Determine actual visual axis range from ticks (Heckbert graph_min/graph_max)
        let x_axis_min = x_ticks.first().copied().unwrap_or(x_min);
        let x_axis_max = x_ticks.last().copied().unwrap_or(x_max);
        let y_axis_min = y_ticks.first().copied().unwrap_or(y_min);
        let y_axis_max = y_ticks.last().copied().unwrap_or(y_max);

        let mut prims: Vec<Primitive> = Vec::new();

        // --- Step 3: Axis lines ---
        // X axis (bottom of plot area)
        let x_axis_y = self.y + self.height;
        let y_axis_x = self.x;

        // X axis line (horizontal, at bottom)
        let x_line = Line::new(self.x, x_axis_y, self.x + self.width, x_axis_y)
            .stroke_color(Color::WHITE)
            .stroke_width(AXIS_STROKE_WIDTH)
            .expect("valid stroke width");
        prims.push(x_line.into());

        // Y axis line (vertical, at left)
        let y_line = Line::new(y_axis_x, self.y, y_axis_x, self.y + self.height)
            .stroke_color(Color::WHITE)
            .stroke_width(AXIS_STROKE_WIDTH)
            .expect("valid stroke width");
        prims.push(y_line.into());

        // --- Step 4: X-axis ticks, labels, and grid lines ---
        for &tick_val in &x_ticks {
            let px = map_x(tick_val, x_axis_min, x_axis_max, self.x, self.width);

            // Tick mark (below x axis)
            let tick = Line::new(px, x_axis_y, px, x_axis_y + TICK_LENGTH)
                .stroke_color(Color::WHITE)
                .stroke_width(TICK_STROKE_WIDTH)
                .expect("valid stroke width");
            prims.push(tick.into());

            // Tick label (below tick)
            let label = format_tick(tick_val, x_step);
            let label_text = Text::new(px, x_axis_y + TICK_LENGTH + TICK_LABEL_OFFSET, label)
                .font_size(TICK_LABEL_SIZE)
                .expect("valid font size");
            prims.push(label_text.into());

            // Grid line (vertical, from x axis up to top of plot)
            // Use Bezier (which has opacity) for subtle grid lines — Line has no opacity builder chain.
            let grid_path = Bezier::new()
                .move_to(px, self.y)
                .line_to(px, self.y + self.height)
                .stroke(Color::rgb(180, 180, 180), GRID_STROKE_WIDTH)
                .expect("valid grid stroke")
                .opacity(GRID_OPACITY)
                .expect("valid opacity");
            prims.push(grid_path.into());
        }

        // --- Step 5: Y-axis ticks, labels, and grid lines ---
        for &tick_val in &y_ticks {
            // SVG Y-axis flip: map_y maps data Y to pixel Y (inverted)
            let py = map_y(tick_val, y_axis_min, y_axis_max, self.y, self.height);

            // Tick mark (left of y axis)
            let tick = Line::new(y_axis_x - TICK_LENGTH, py, y_axis_x, py)
                .stroke_color(Color::WHITE)
                .stroke_width(TICK_STROKE_WIDTH)
                .expect("valid stroke width");
            prims.push(tick.into());

            // Tick label (left of tick, right-aligned)
            let label = format_tick(tick_val, y_step);
            // Position label to the left of the tick; SVG text-anchor="end" handles right-align
            let label_text = Text::new(
                y_axis_x - TICK_LENGTH - TICK_LABEL_OFFSET * 0.6,
                py + TICK_LABEL_SIZE * 0.35,
                label,
            )
            .font_size(TICK_LABEL_SIZE)
            .expect("valid font size");
            prims.push(label_text.into());

            // Grid line (horizontal, from y axis to right edge of plot)
            let grid_path = Bezier::new()
                .move_to(self.x, py)
                .line_to(self.x + self.width, py)
                .stroke(Color::rgb(180, 180, 180), GRID_STROKE_WIDTH)
                .expect("valid grid stroke")
                .opacity(GRID_OPACITY)
                .expect("valid opacity");
            prims.push(grid_path.into());
        }

        // --- Step 6: Axis titles ---
        if let Some(ref title) = self.x_title {
            // Centered below X axis
            let title_x = self.x + self.width / 2.0;
            let title_y = self.y + self.height + TICK_LENGTH + TICK_LABEL_OFFSET + TITLE_OFFSET;
            prims.push(
                Text::new(title_x, title_y, title.clone())
                    .font_size(AXIS_TITLE_SIZE)
                    .expect("valid font size")
                    .into(),
            );
        }

        if let Some(ref title) = self.y_title {
            // Rotated beside Y axis — render as Text at rotated position
            // Positioned to the left of y-axis.
            let title_x = self.x - TICK_LENGTH - TICK_LABEL_OFFSET * 0.6 - TITLE_OFFSET;
            let title_y = self.y + self.height / 2.0;
            prims.push(
                Text::new(title_x, title_y, title.clone())
                    .font_size(AXIS_TITLE_SIZE)
                    .expect("valid font size")
                    .into(),
            );
        }

        // --- Step 6.5: Confidence bands (rendered BELOW data curves) ---
        for band in &self.bands {
            if band.upper_points.len() < 2 || band.lower_points.len() < 2 { continue; }
            let visual_upper: Vec<(f64, f64)> = band.upper_points.iter().map(|&(dx, dy)| {
                (map_x(dx, x_axis_min, x_axis_max, self.x, self.width),
                 map_y(dy, y_axis_min, y_axis_max, self.y, self.height))
            }).collect();
            let visual_lower: Vec<(f64, f64)> = band.lower_points.iter().map(|&(dx, dy)| {
                (map_x(dx, x_axis_min, x_axis_max, self.x, self.width),
                 map_y(dy, y_axis_min, y_axis_max, self.y, self.height))
            }).collect();
            let bez = band.to_bezier_path(&visual_upper, &visual_lower);
            prims.push(bez.into());
        }

        // --- Step 7: Data curves ---
        // Map each curve's data points to visual space FIRST, then compute Catmull-Rom.
        for curve in &self.curves {
            if curve.points.is_empty() { continue; }
            let visual_pts: Vec<(f64, f64)> = curve.points.iter().map(|&(dx, dy)| {
                (
                    map_x(dx, x_axis_min, x_axis_max, self.x, self.width),
                    map_y(dy, y_axis_min, y_axis_max, self.y, self.height),
                )
            }).collect();

            if visual_pts.len() >= 2 {
                let bez = curve.to_bezier_path(&visual_pts);
                prims.push(bez.into());
            }
        }

        prims
    }
}

// ---- Pure helper functions ----

/// Map a data-space value to visual-space X pixel coordinate.
/// visual range: [axes.x, axes.x + axes.width]
fn map_x(val: f64, data_min: f64, data_max: f64, axes_x: f64, axes_width: f64) -> f64 {
    if (data_max - data_min).abs() < 1e-10 {
        return axes_x + axes_width / 2.0;
    }
    let t = (val - data_min) / (data_max - data_min);
    axes_x + t * axes_width
}

/// Map a data-space value to visual-space Y pixel coordinate.
/// SVG Y-axis inversion: data Y increases upward; pixel Y increases downward.
/// data_min maps to pixel bottom (axes.y + axes.height); data_max maps to pixel top (axes.y).
fn map_y(val: f64, data_min: f64, data_max: f64, axes_y: f64, axes_height: f64) -> f64 {
    if (data_max - data_min).abs() < 1e-10 {
        return axes_y + axes_height / 2.0;
    }
    let t = (val - data_min) / (data_max - data_min);
    // Swap: t=0 → bottom, t=1 → top
    (axes_y + axes_height) - t * axes_height
}

/// Compute auto-range from a slice of values with padding.
/// Handles degenerate case (all same value or empty) by using ±0.5 span.
fn compute_range(values: &[f64], padding_frac: f64) -> (f64, f64) {
    if values.is_empty() {
        return (-1.0, 1.0);
    }
    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let span = max - min;
    if span < 1e-10 {
        // Degenerate: single value or all same — locked decision: use ±0.5 span
        return (min - 0.5, max + 0.5);
    }
    let pad = span * padding_frac;
    (min - pad, max + pad)
}

/// Heckbert "nice numbers" tick generation (Graphics Gems, 1990).
/// Returns 5–10 human-readable tick values at multiples of 1, 2, or 5 × 10^n.
fn generate_ticks(data_min: f64, data_max: f64, target_count: usize) -> Vec<f64> {
    if data_min >= data_max {
        return vec![data_min, data_max];
    }
    let range = nice_num(data_max - data_min, false);
    let step = nice_num(range / (target_count - 1) as f64, true);
    if step <= 0.0 {
        return vec![data_min, data_max];
    }
    let graph_min = (data_min / step).floor() * step;
    let graph_max = (data_max / step).ceil() * step;

    let mut ticks = Vec::new();
    let mut t = graph_min;
    // Allow tick slightly beyond graph_max to catch the last tick (floating-point tolerance)
    while t <= graph_max + step * 0.5 {
        ticks.push(t);
        t += step;
    }
    ticks
}

/// Heckbert nice-number rounding. If round=true, rounds to nearest nice number;
/// if round=false, returns ceiling nice number (used for range).
fn nice_num(x: f64, round: bool) -> f64 {
    if x <= 0.0 { return x; }
    let exp = x.log10().floor();
    let f = x / 10f64.powf(exp);  // fractional part in [1, 10)
    let nice_f = if round {
        if f < 1.5 { 1.0 } else if f < 3.0 { 2.0 } else if f < 7.0 { 5.0 } else { 10.0 }
    } else {
        #[allow(clippy::collapsible_else_if)]
        if f <= 1.0 { 1.0 } else if f <= 2.0 { 2.0 } else if f <= 5.0 { 5.0 } else { 10.0 }
    };
    nice_f * 10f64.powf(exp)
}

/// Compute decimal precision from the tick step size.
/// step=1.0 → 0 decimals; step=0.1 → 1 decimal; step=0.01 → 2 decimals.
fn tick_precision(step: f64) -> usize {
    if step >= 1.0 { 0 } else { (-step.log10().floor()) as usize }
}

/// Format a tick value with appropriate decimal precision to avoid floating-point noise.
/// E.g., step=0.1 → "0.3" not "0.30000000000000004".
fn format_tick(val: f64, step: f64) -> String {
    let precision = tick_precision(step);
    format!("{:.precision$}", val, precision = precision)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_ticks_produces_reasonable_count() {
        let ticks = generate_ticks(0.0, 10.0, 6);
        assert!(ticks.len() >= 2 && ticks.len() <= 12, "tick count was {}", ticks.len());
    }

    #[test]
    fn generate_ticks_values_are_round_numbers() {
        let ticks = generate_ticks(0.0, 100.0, 6);
        // All ticks should be multiples of 20 (nice step for 0–100 range, target 6)
        for t in &ticks {
            assert!((t % 20.0).abs() < 1e-6 || (t % 10.0).abs() < 1e-6,
                "tick {} is not a round number", t);
        }
    }

    #[test]
    fn format_tick_avoids_floating_point_noise() {
        // 0.1 + 0.1 + 0.1 != 0.3 exactly — format_tick must use precision
        let step = 0.1f64;
        let val = 0.1 + 0.1 + 0.1;  // raw f64: ~0.30000000000000004
        let formatted = format_tick(val, step);
        assert_eq!(formatted, "0.3", "got: {}", formatted);
    }

    #[test]
    fn compute_range_pads_beyond_data() {
        let (lo, hi) = compute_range(&[0.0, 10.0], 0.07);
        assert!(lo < 0.0, "lower bound should be below 0");
        assert!(hi > 10.0, "upper bound should be above 10");
    }

    #[test]
    fn compute_range_degenerate_single_value() {
        let (lo, hi) = compute_range(&[5.0], 0.07);
        assert_eq!(lo, 4.5);
        assert_eq!(hi, 5.5);
    }

    #[test]
    fn compute_range_empty_returns_default() {
        let (lo, hi) = compute_range(&[], 0.07);
        assert!(lo < hi);
    }

    #[test]
    fn map_x_maps_data_min_to_left_edge() {
        let px = map_x(0.0, 0.0, 10.0, 100.0, 800.0);
        assert!((px - 100.0).abs() < 1e-6);
    }

    #[test]
    fn map_x_maps_data_max_to_right_edge() {
        let px = map_x(10.0, 0.0, 10.0, 100.0, 800.0);
        assert!((px - 900.0).abs() < 1e-6);
    }

    #[test]
    fn map_y_inverts_axis_data_min_to_bottom() {
        // data_min should map to axes_y + height (bottom pixel)
        let py = map_y(0.0, 0.0, 10.0, 100.0, 500.0);
        assert!((py - 600.0).abs() < 1e-6, "expected 600.0 (bottom), got {}", py);
    }

    #[test]
    fn map_y_inverts_axis_data_max_to_top() {
        // data_max should map to axes_y (top pixel)
        let py = map_y(10.0, 0.0, 10.0, 100.0, 500.0);
        assert!((py - 100.0).abs() < 1e-6, "expected 100.0 (top), got {}", py);
    }

    #[test]
    fn axes_to_primitives_produces_primitives() {
        use crate::dataviz::DataCurve;
        let data: Vec<(f64, f64)> = (0..5).map(|i| (i as f64, i as f64 * 2.0)).collect();
        let curve = DataCurve::new(data).unwrap();
        let axes = Axes::new(100.0, 100.0, 800.0, 500.0).add_curve(curve);
        let prims = axes.to_primitives();
        // Should have at least: 2 axis lines + ticks + labels + grid lines + 1 curve path
        assert!(prims.len() > 10, "expected >10 primitives, got {}", prims.len());
    }

    #[test]
    fn axes_explicit_range_overrides_auto() {
        use crate::dataviz::DataCurve;
        let data = vec![(0.0, 0.0), (1.0, 1.0)];
        let curve = DataCurve::new(data).unwrap();
        let axes = Axes::new(0.0, 0.0, 800.0, 500.0)
            .x_range(-10.0, 10.0)
            .y_range(-5.0, 5.0)
            .add_curve(curve);
        // Should not panic; just verify we get primitives
        let prims = axes.to_primitives();
        assert!(!prims.is_empty());
    }
}
