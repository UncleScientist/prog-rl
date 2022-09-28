use bevy_ecs::prelude::*;
use bracket_lib::prelude::*;

use crate::components::*;
use crate::game_state::*;
use crate::map::*;
use crate::messages::*;

pub struct KeyboardEvent(pub VirtualKeyCode);
pub struct MeleeEvent {
    pub source: Entity,
    pub target: Entity,
}

pub fn handle_key(
    mut reader: EventReader<KeyboardEvent>,
    mut writer: EventWriter<MeleeEvent>,
    mut messages: ResMut<Messages>,
    map: Res<Map>,
    mut runner: ResMut<RunSystems>,
    mut query: Query<(Entity, (&mut Position, With<Player>))>,
) {
    let mut action_performed = false;
    let mut advance_message = false;

    for event in reader.iter() {
        if !messages.is_empty() {
            if event.0 != VirtualKeyCode::Space {
                continue;
            }
        }

        for (source, (mut position, _)) in query.iter_mut() {
            let mut new_position = *position;
            match event.0 {
                VirtualKeyCode::Numpad4 | VirtualKeyCode::H => {
                    new_position.x -= 1;
                    action_performed = true;
                }
                VirtualKeyCode::Numpad6 | VirtualKeyCode::L => {
                    new_position.x += 1;
                    action_performed = true;
                }
                VirtualKeyCode::Numpad2 | VirtualKeyCode::J => {
                    new_position.y += 1;
                    action_performed = true;
                }
                VirtualKeyCode::Numpad8 | VirtualKeyCode::K => {
                    new_position.y -= 1;
                    action_performed = true;
                }
                VirtualKeyCode::Numpad7 | VirtualKeyCode::Y => {
                    new_position.x -= 1;
                    new_position.y -= 1;
                    action_performed = true;
                }
                VirtualKeyCode::Numpad9 | VirtualKeyCode::U => {
                    new_position.x += 1;
                    new_position.y -= 1;
                    action_performed = true;
                }
                VirtualKeyCode::Numpad1 | VirtualKeyCode::B => {
                    new_position.x -= 1;
                    new_position.y += 1;
                    action_performed = true;
                }
                VirtualKeyCode::Numpad3 | VirtualKeyCode::N => {
                    new_position.x += 1;
                    new_position.y += 1;
                    action_performed = true;
                }
                VirtualKeyCode::Space => {
                    advance_message = true;
                }
                VirtualKeyCode::Numpad5 => {
                    action_performed = true;
                }
                _ => {}
            }

            if *position != new_position && map.walkable(new_position.x, new_position.y) {
                if let Some(target) = map.try_walk(&new_position) {
                    writer.send(MeleeEvent {
                        source,
                        target: *target,
                    });
                    messages.add("You attack!");
                } else {
                    *position = new_position;
                }
            }
        }
    }

    if action_performed {
        runner.run_systems = true;
    } else if advance_message {
        messages.advance();
    }
}
