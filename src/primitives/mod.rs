// src/primitives/mod.rs
pub mod arrow;
pub mod bezier;
pub mod circle;
pub mod line;
pub mod rect;
pub mod text;

pub use arrow::Arrow;
pub use bezier::Bezier;
pub use circle::Circle;
pub use line::Line;
pub use rect::Rect;
pub use text::Text;

/// The union of all drawable primitives.
/// svg_gen::build_svg_document() matches on this enum to produce SVG nodes.
#[derive(Debug, Clone)]
pub enum Primitive {
    Circle(Circle),
    Rect(Rect),
    Line(Line),
    Arrow(Arrow),
    Text(Text),
    Bezier(Bezier),
}

impl From<Circle> for Primitive {
    fn from(c: Circle) -> Self {
        Primitive::Circle(c)
    }
}

impl From<Rect> for Primitive {
    fn from(r: Rect) -> Self {
        Primitive::Rect(r)
    }
}

impl From<Line> for Primitive {
    fn from(l: Line) -> Self {
        Primitive::Line(l)
    }
}

impl From<Arrow> for Primitive {
    fn from(a: Arrow) -> Self {
        Primitive::Arrow(a)
    }
}

impl From<Text> for Primitive {
    fn from(t: Text) -> Self {
        Primitive::Text(t)
    }
}

impl From<Bezier> for Primitive {
    fn from(b: Bezier) -> Self {
        Primitive::Bezier(b)
    }
}
