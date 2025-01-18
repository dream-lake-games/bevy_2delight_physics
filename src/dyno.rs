use bevy::prelude::*;

#[derive(Component, Clone, Debug, Reflect, Default)]
#[require(crate::pos::Pos)]
pub struct Dyno {
    pub vel: Vec2,
}
impl Dyno {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            vel: Vec2::new(x, y),
        }
    }
}
