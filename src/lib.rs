// src/lib.rs
pub mod animation;
pub mod color;
pub mod error;
pub mod primitives;
pub mod scene;
pub mod svg_gen;

pub use animation::{Easing, Tween};
pub use color::Color;
pub use error::EidosError;
pub use scene::Scene;
