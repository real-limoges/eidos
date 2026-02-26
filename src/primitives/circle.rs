// src/primitives/circle.rs
use crate::Color;
use keyframe_derive::CanTween;

/// A circle primitive defined by its center and radius.
#[derive(Debug, Clone)]
pub struct Circle {
    pub cx: f64,
    pub cy: f64,
    pub r: f64,
    pub fill: Option<Color>,
    pub stroke: Option<(Color, f64)>, // (color, width)
    pub opacity: f64,
}

impl Circle {
    /// Create a circle centered at (cx, cy) with radius r.
    pub fn new(cx: f64, cy: f64, r: f64) -> Self {
        Circle {
            cx,
            cy,
            r,
            fill: None,
            stroke: None,
            opacity: 1.0,
        }
    }

    /// Set fill color. Returns Self for chaining (fill accepts any Color, no validation needed).
    pub fn fill(mut self, color: Color) -> Self {
        self.fill = Some(color);
        self
    }

    /// Set stroke color and width. Negative widths are clamped to 0.0.
    pub fn stroke(mut self, color: Color, width: f64) -> Self {
        self.stroke = Some((color, width.max(0.0)));
        self
    }

    /// Set opacity in [0.0, 1.0]. Values outside [0.0, 1.0] are clamped.
    pub fn opacity(mut self, value: f64) -> Self {
        self.opacity = value.clamp(0.0, 1.0);
        self
    }

    /// Convert to an svg::node::element::Circle node for inclusion in an SVG document.
    pub fn to_svg_element(&self) -> svg::node::element::Circle {
        use svg::node::element::Circle as SvgCircle;

        let mut el = SvgCircle::new()
            .set("cx", self.cx)
            .set("cy", self.cy)
            .set("r", self.r)
            .set("opacity", self.opacity);

        el = match self.fill {
            Some(c) => el.set("fill", c.to_hex()),
            None => el.set("fill", "none"),
        };

        if let Some((color, width)) = self.stroke {
            el = el.set("stroke", color.to_hex()).set("stroke-width", width);
        }

        el
    }
}

/// Animatable state for Circle. All fields are f64 for CanTween compatibility.
/// Color channels are 0.0..=255.0; opacity is 0.0..=1.0.
#[derive(Clone, CanTween)]
pub struct CircleState {
    pub cx: f64,
    pub cy: f64,
    pub r: f64,
    pub fill_r: f64,
    pub fill_g: f64,
    pub fill_b: f64,
    pub opacity: f64,
}

impl CircleState {
    /// Construct a CircleState from position, radius, fill color, and opacity.
    ///
    /// Color channels are stored as f64 (0.0..=255.0) for tween interpolation.
    pub fn new(cx: f64, cy: f64, r: f64, fill: Color, opacity: f64) -> Self {
        CircleState {
            cx,
            cy,
            r,
            fill_r: fill.r as f64,
            fill_g: fill.g as f64,
            fill_b: fill.b as f64,
            opacity,
        }
    }

    /// Build a Circle from this interpolated state.
    /// Color channels are clamped to [0, 255] then cast to u8.
    /// Opacity is clamped to [0.0, 1.0].
    pub fn to_circle(&self) -> Circle {
        let r = self.fill_r.clamp(0.0, 255.0) as u8;
        let g = self.fill_g.clamp(0.0, 255.0) as u8;
        let b = self.fill_b.clamp(0.0, 255.0) as u8;
        Circle::new(self.cx, self.cy, self.r)
            .fill(crate::Color::rgb(r, g, b))
            .opacity(self.opacity.clamp(0.0, 1.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Color;

    #[test]
    fn circle_negative_stroke_is_clamped_to_zero() {
        let result = Circle::new(50.0, 50.0, 30.0).stroke(Color::WHITE, -0.1);
        assert_eq!(result.stroke, Some((Color::WHITE, 0.0)));
    }

    #[test]
    fn circle_opacity_clamped() {
        let high = Circle::new(50.0, 50.0, 30.0).opacity(2.0);
        assert_eq!(high.opacity, 1.0);
        let low = Circle::new(50.0, 50.0, 30.0).opacity(-0.5);
        assert_eq!(low.opacity, 0.0);
    }

    #[test]
    fn circle_state_new_decomposes_color() {
        let s = CircleState::new(10.0, 20.0, 30.0, Color::RED, 0.8);
        assert_eq!(s.fill_r, 255.0);
        assert_eq!(s.fill_g, 0.0);
        assert_eq!(s.fill_b, 0.0);
        assert_eq!(s.opacity, 0.8);
    }

    #[test]
    fn circle_valid_chain_succeeds() {
        let c = Circle::new(100.0, 100.0, 50.0)
            .fill(Color::RED)
            .stroke(Color::WHITE, 2.0)
            .opacity(0.9);
        assert_eq!(c.cx, 100.0);
        assert_eq!(c.fill, Some(Color::RED));
    }
}
