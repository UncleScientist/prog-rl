use bevy_ecs::prelude::*;
use bracket_lib::prelude::*;

mod player;
pub use player::Player;

#[derive(Debug, Component)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn point(&self) -> Point {
        Point {
            x: self.x,
            y: self.y,
        }
    }
}

impl From<&Point> for Position {
    fn from(p: &Point) -> Self {
        Self { x: p.x, y: p.y }
    }
}

#[derive(Debug, Component)]
pub struct Mob {
    pub appearance: char,
}
