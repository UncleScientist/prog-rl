use bevy_ecs::prelude::*;
use bracket_lib::prelude::*;

use crate::components::*;
use crate::game_state::*;
use crate::map::*;

pub struct KeyboardEvent(pub VirtualKeyCode);

pub fn handle_key(
    mut reader: EventReader<KeyboardEvent>,
    map: Res<Map>,
    mut runner: ResMut<RunSystems>,
    mut query: Query<&mut Position, With<Player>>,
) {
    let mut action_performed = false;

    for (event, _id) in reader.iter_with_id() {
        for mut position in query.iter_mut() {
            match event.0 {
                VirtualKeyCode::H => {
                    if map.walkable(position.x - 1, position.y) {
                        position.x -= 1;
                        action_performed = true;
                    }
                }
                VirtualKeyCode::L => {
                    if map.walkable(position.x + 1, position.y) {
                        position.x += 1;
                        action_performed = true;
                    }
                }
                VirtualKeyCode::J => {
                    if map.walkable(position.x, position.y + 1) {
                        position.y += 1;
                        action_performed = true;
                    }
                }
                VirtualKeyCode::K => {
                    if map.walkable(position.x, position.y - 1) {
                        position.y -= 1;
                        action_performed = true;
                    }
                }
                VirtualKeyCode::Space => {
                    action_performed = true;
                }
                _ => {}
            }
        }
    }

    if action_performed {
        runner.run_systems = true;
    }
}
