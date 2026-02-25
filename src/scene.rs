// src/scene.rs
// Implemented in plan 01-02
use crate::EidosError;

pub struct Scene;

impl Scene {
    pub fn new(_width: u32, _height: u32, _fps: u32) -> Result<Self, EidosError> {
        Ok(Scene)
    }
}
