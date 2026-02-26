// src/primitives/rect.rs
use crate::Color;
use keyframe_derive::CanTween;

/// A rectangle primitive with top-left origin.
#[derive(Debug, Clone)]
pub struct Rect {
    pub x: f64, // top-left x
    pub y: f64, // top-left y
    pub width: f64,
    pub height: f64,
    pub fill: Option<Color>,
    pub stroke: Option<(Color, f64)>, // (color, width)
    pub opacity: f64,
}

impl Rect {
    /// Create a rectangle with top-left corner at (x, y) with given width and height.
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Rect {
            x,
            y,
            width,
            height,
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

    /// Convert to an svg::node::element::Rectangle node for inclusion in an SVG document.
    pub fn to_svg_element(&self) -> svg::node::element::Rectangle {
        use svg::node::element::Rectangle;

        let mut el = Rectangle::new()
            .set("x", self.x)
            .set("y", self.y)
            .set("width", self.width)
            .set("height", self.height)
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

/// Animatable state for Rect. All fields are f64 for CanTween compatibility.
/// Color channels are 0.0..=255.0; opacity is 0.0..=1.0.
#[derive(Clone, CanTween)]
pub struct RectState {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub fill_r: f64,
    pub fill_g: f64,
    pub fill_b: f64,
    pub opacity: f64,
}

impl RectState {
    /// Construct a RectState from position, dimensions, fill color, and opacity.
    ///
    /// Color channels are stored as f64 (0.0..=255.0) for tween interpolation.
    pub fn new(x: f64, y: f64, width: f64, height: f64, fill: Color, opacity: f64) -> Self {
        RectState {
            x,
            y,
            width,
            height,
            fill_r: fill.r as f64,
            fill_g: fill.g as f64,
            fill_b: fill.b as f64,
            opacity,
        }
    }

    /// Build a Rect from this interpolated state.
    /// Color channels are clamped to [0, 255] then cast to u8.
    /// Opacity is clamped to [0.0, 1.0].
    pub fn to_rect(&self) -> Rect {
        let r = self.fill_r.clamp(0.0, 255.0) as u8;
        let g = self.fill_g.clamp(0.0, 255.0) as u8;
        let b = self.fill_b.clamp(0.0, 255.0) as u8;
        Rect::new(self.x, self.y, self.width, self.height)
            .fill(crate::Color::rgb(r, g, b))
            .opacity(self.opacity.clamp(0.0, 1.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Color;

    #[test]
    fn rect_negative_stroke_is_clamped_to_zero() {
        let result = Rect::new(0.0, 0.0, 100.0, 50.0).stroke(Color::WHITE, -1.0);
        assert_eq!(result.stroke, Some((Color::WHITE, 0.0)));
    }

    #[test]
    fn rect_opacity_clamped() {
        let high = Rect::new(0.0, 0.0, 100.0, 50.0).opacity(1.5);
        assert_eq!(high.opacity, 1.0);
        let low = Rect::new(0.0, 0.0, 100.0, 50.0).opacity(-0.5);
        assert_eq!(low.opacity, 0.0);
    }

    #[test]
    fn rect_state_new_decomposes_color() {
        let s = RectState::new(10.0, 20.0, 200.0, 100.0, Color::BLUE, 0.7);
        assert_eq!(s.fill_r, 0.0);
        assert_eq!(s.fill_g, 0.0);
        assert_eq!(s.fill_b, 255.0);
        assert_eq!(s.opacity, 0.7);
    }

    #[test]
    fn rect_valid_chain_succeeds() {
        let rect = Rect::new(10.0, 20.0, 200.0, 100.0)
            .fill(Color::BLUE)
            .stroke(Color::WHITE, 2.0)
            .opacity(0.8);
        assert_eq!(rect.x, 10.0);
        assert_eq!(rect.fill, Some(Color::BLUE));
    }
}
