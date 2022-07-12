use bevy_ecs::prelude::*;

#[derive(Debug, Component, Clone)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}
