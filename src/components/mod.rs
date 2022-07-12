use bevy_ecs::prelude::*;

#[derive(Debug, Component)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Component)]
pub struct Mob {
    pub appearance: char,
}
