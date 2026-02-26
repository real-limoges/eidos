// src/primitives/text.rs
use crate::Color;
use keyframe_derive::CanTween;

/// Text horizontal alignment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Alignment {
    Left,
    Center,
    Right,
}

impl Alignment {
    fn to_svg_anchor(self) -> &'static str {
        match self {
            Alignment::Left => "start",
            Alignment::Center => "middle",
            Alignment::Right => "end",
        }
    }
}

/// A text label positioned at (x, y).
///
/// Multi-line text is handled by splitting `content` on literal `\n` and generating
/// one `<tspan dy="Nem">` per line. SVG `<text>` does not natively handle newlines,
/// so each line must be a separate `<tspan>` with a relative vertical offset.
#[derive(Debug, Clone)]
pub struct Text {
    pub x: f64,
    pub y: f64,
    pub content: String,
    pub fill: Color,
    pub stroke: Option<(Color, f64)>,
    pub opacity: f64,
    pub font_size: f64,
    pub alignment: Alignment,
    /// Line height multiplier in em units (1.0 = normal, 1.5 = 150%).
    pub line_height: f64,
}

impl Text {
    /// Create a text label at (x, y) with the given content.
    pub fn new(x: f64, y: f64, content: impl Into<String>) -> Self {
        Text {
            x,
            y,
            content: content.into(),
            fill: Color::WHITE,
            stroke: None,
            opacity: 1.0,
            font_size: 16.0,
            alignment: Alignment::Left,
            line_height: 1.2,
        }
    }

    /// Set fill color. Returns Self for chaining (no validation needed).
    pub fn fill(mut self, color: Color) -> Self {
        self.fill = color;
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

    /// Set font size in pixels. Values <= 0.0 are clamped to 1.0.
    pub fn font_size(mut self, size: f64) -> Self {
        self.font_size = size.max(1.0);
        self
    }

    /// Set horizontal text alignment. Returns Self for chaining (no validation needed).
    pub fn alignment(mut self, align: Alignment) -> Self {
        self.alignment = align;
        self
    }

    /// Set line height multiplier in em units. Values <= 0.0 are clamped to 0.1.
    pub fn line_height(mut self, lh: f64) -> Self {
        self.line_height = lh.max(0.1);
        self
    }

    /// Convert to an svg::node::element::Text node with multi-line tspan support.
    ///
    /// Content is split on `\n`. The first line has `dy="0"` (no shift); subsequent
    /// lines have `dy="{line_height}em"` for relative vertical offset. Each tspan
    /// also resets `x` so alignment works correctly across all lines.
    ///
    /// TSpan::new(content) in the svg 0.18 crate takes a content string directly.
    pub fn to_svg_element(&self) -> svg::node::element::Text {
        use svg::node::element::{TSpan, Text as SvgText};

        let lines: Vec<&str> = self.content.split('\n').collect();

        // Text::new(content) in svg 0.18 takes a content arg; pass empty string
        // since all content is provided via tspan children below.
        let mut text_el = SvgText::new("")
            .set("x", self.x)
            .set("y", self.y)
            .set("fill", self.fill.to_hex())
            .set("font-size", format!("{}px", self.font_size))
            .set("font-family", "Noto Sans")
            .set("text-anchor", self.alignment.to_svg_anchor())
            .set("opacity", self.opacity);

        if let Some((color, width)) = self.stroke {
            text_el = text_el
                .set("stroke", color.to_hex())
                .set("stroke-width", width);
        }

        for (i, line) in lines.iter().enumerate() {
            let dy = if i == 0 {
                "0".to_string()
            } else {
                format!("{}em", self.line_height)
            };

            // Reset x per tspan so multi-line alignment is correct (each line
            // is independently anchored to self.x via text-anchor).
            let tspan = TSpan::new(line.to_string()).set("x", self.x).set("dy", dy);

            text_el = text_el.add(tspan);
        }

        text_el
    }
}

/// Animatable state for Text. All fields are f64 for CanTween compatibility.
/// Content (string) is not animatable — pass it separately to to_text().
/// Color channels are 0.0..=255.0; opacity is 0.0..=1.0.
#[derive(Clone, CanTween)]
pub struct TextState {
    pub x: f64,
    pub y: f64,
    pub font_size: f64,
    pub fill_r: f64,
    pub fill_g: f64,
    pub fill_b: f64,
    pub opacity: f64,
}

impl TextState {
    /// Construct a TextState from position, font size, fill color, and opacity.
    ///
    /// Color channels are stored as f64 (0.0..=255.0) for tween interpolation.
    pub fn new(x: f64, y: f64, font_size: f64, fill: Color, opacity: f64) -> Self {
        TextState {
            x,
            y,
            font_size,
            fill_r: fill.r as f64,
            fill_g: fill.g as f64,
            fill_b: fill.b as f64,
            opacity,
        }
    }

    /// Build a Text from this interpolated state. Content must be supplied separately
    /// (strings are not interpolatable — use a fixed &str from the animation closure).
    pub fn to_text(&self, content: &str) -> crate::primitives::Text {
        let r = self.fill_r.clamp(0.0, 255.0) as u8;
        let g = self.fill_g.clamp(0.0, 255.0) as u8;
        let b = self.fill_b.clamp(0.0, 255.0) as u8;
        crate::primitives::Text::new(self.x, self.y, content)
            .fill(crate::Color::rgb(r, g, b))
            .font_size(self.font_size.max(1.0))
            .opacity(self.opacity.clamp(0.0, 1.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Color;

    #[test]
    fn text_negative_stroke_is_clamped() {
        let result = Text::new(10.0, 20.0, "hello").stroke(Color::WHITE, -1.0);
        assert_eq!(result.stroke, Some((Color::WHITE, 0.0)));
    }

    #[test]
    fn text_zero_font_size_is_clamped() {
        let result = Text::new(10.0, 20.0, "hello").font_size(0.0);
        assert_eq!(result.font_size, 1.0);
    }

    #[test]
    fn text_zero_line_height_is_clamped() {
        let result = Text::new(10.0, 20.0, "hello").line_height(0.0);
        assert_eq!(result.line_height, 0.1);
    }

    #[test]
    fn text_state_new_decomposes_color() {
        let s = TextState::new(50.0, 100.0, 24.0, Color::YELLOW, 0.5);
        assert_eq!(s.fill_r, 255.0);
        assert_eq!(s.fill_g, 255.0);
        assert_eq!(s.fill_b, 0.0);
        assert_eq!(s.font_size, 24.0);
        assert_eq!(s.opacity, 0.5);
    }

    #[test]
    fn text_valid_chain_succeeds() {
        let t = Text::new(50.0, 100.0, "Hello\nWorld")
            .fill(Color::YELLOW)
            .font_size(24.0)
            .alignment(Alignment::Center)
            .line_height(1.5)
            .opacity(0.9);
        assert_eq!(t.font_size, 24.0);
        assert_eq!(t.alignment, Alignment::Center);
        assert_eq!(t.line_height, 1.5);
    }

    #[test]
    fn text_multiline_split() {
        let t = Text::new(0.0, 0.0, "line1\nline2\nline3");
        let lines: Vec<&str> = t.content.split('\n').collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "line1");
        assert_eq!(lines[2], "line3");
    }
}
