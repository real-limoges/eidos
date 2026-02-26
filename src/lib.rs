// src/lib.rs
pub mod animation;
pub mod color;
pub mod dataviz;
pub mod error;
pub mod primitives;
pub mod scene;
pub mod svg_gen;

pub use animation::{Easing, Tween};
pub use color::Color;
pub use dataviz::{Axes, AxisRange, Camera, ConfidenceBand, DataCurve, Point2D, Point3D, RenderMode, SplineFit, SurfacePlot, Vector3D};
pub use error::EidosError;
pub use primitives::{Arrow, Bezier, Circle, Line, Primitive, Rect, Text};
pub use primitives::circle::CircleState;
pub use primitives::rect::RectState;
pub use primitives::line::LineState;
pub use primitives::text::TextState;
pub use scene::{Scene, SceneBuilder};
