// src/primitives/text.rs
use crate::{Color, EidosError};

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

    /// Set stroke color and width. Returns Err if width is negative.
    pub fn stroke(mut self, color: Color, width: f64) -> Result<Self, EidosError> {
        if width < 0.0 {
            return Err(EidosError::InvalidConfig(
                "stroke width must be non-negative".into(),
            ));
        }
        self.stroke = Some((color, width));
        Ok(self)
    }

    /// Set opacity in [0.0, 1.0]. Returns Err if outside range.
    pub fn opacity(mut self, value: f64) -> Result<Self, EidosError> {
        if !(0.0..=1.0).contains(&value) {
            return Err(EidosError::InvalidConfig(
                "opacity must be in range [0.0, 1.0]".into(),
            ));
        }
        self.opacity = value;
        Ok(self)
    }

    /// Set font size in pixels. Returns Err if size is not positive.
    pub fn font_size(mut self, size: f64) -> Result<Self, EidosError> {
        if size <= 0.0 {
            return Err(EidosError::InvalidConfig(
                "font size must be positive".into(),
            ));
        }
        self.font_size = size;
        Ok(self)
    }

    /// Set horizontal text alignment. Returns Self for chaining (no validation needed).
    pub fn alignment(mut self, align: Alignment) -> Self {
        self.alignment = align;
        self
    }

    /// Set line height multiplier in em units. Returns Err if not positive.
    pub fn line_height(mut self, lh: f64) -> Result<Self, EidosError> {
        if lh <= 0.0 {
            return Err(EidosError::InvalidConfig(
                "line height must be positive".into(),
            ));
        }
        self.line_height = lh;
        Ok(self)
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
            let tspan = TSpan::new(line.to_string())
                .set("x", self.x)
                .set("dy", dy);

            text_el = text_el.add(tspan);
        }

        text_el
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Color;

    #[test]
    fn text_negative_stroke_returns_err() {
        let result = Text::new(10.0, 20.0, "hello").stroke(Color::WHITE, -1.0);
        assert!(result.is_err());
    }

    #[test]
    fn text_zero_font_size_returns_err() {
        let result = Text::new(10.0, 20.0, "hello").font_size(0.0);
        assert!(result.is_err());
    }

    #[test]
    fn text_zero_line_height_returns_err() {
        let result = Text::new(10.0, 20.0, "hello").line_height(0.0);
        assert!(result.is_err());
    }

    #[test]
    fn text_valid_chain_succeeds() {
        let t = Text::new(50.0, 100.0, "Hello\nWorld")
            .fill(Color::YELLOW)
            .font_size(24.0)
            .unwrap()
            .alignment(Alignment::Center)
            .line_height(1.5)
            .unwrap()
            .opacity(0.9)
            .unwrap();
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
