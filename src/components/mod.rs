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
    pub glyph: char,
}

#[derive(Debug, Component, Copy, Clone)]
pub struct Stat {
    pub max: i32,
    pub cur: i32,
}

#[derive(Debug, Component, Copy, Clone)]
pub struct Stats {
    pub hp: Stat,
    pub mp: Stat,
}

impl Stats {
    pub fn new(hp: i32, mp: i32) -> Self {
        Self {
            hp: Stat { max: hp, cur: hp },
            mp: Stat { max: mp, cur: mp },
        }
    }
}
