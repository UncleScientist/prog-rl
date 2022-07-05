use specs::prelude::*;
use specs_derive::Component;

#[derive(Debug, Component, Clone)]
pub struct Attributes {
    _laziness: i32,
    _hubris: i32,
}

pub fn register_components(world: &mut World) {
    world.register::<Attributes>();
}
