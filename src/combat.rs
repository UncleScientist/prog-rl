use std::collections::HashMap;

use bevy_ecs::prelude::*;

use crate::components::*;
use crate::keyboard::*;
use crate::messages::*;

pub struct DealDamage {
    pub source: Entity,
    pub target: Entity,
    pub name: String,
    pub amount: i32,
}

pub fn resolve_combat(
    mut reader: EventReader<MeleeEvent>,
    mut writer: EventWriter<DealDamage>,
    query: Query<(Entity, &Position, &Stats, &Name)>,
) {
    let mut hm = HashMap::new();

    for (entity, _, stats, name) in query.iter() {
        hm.insert(entity, (*stats, &name.name));
    }

    for event in reader.iter() {
        let source_stats = hm.get(&event.source).unwrap();
        let _target_stats = hm.get(&event.target).unwrap();
        writer.send(DealDamage {
            source: event.source,
            target: event.target,
            name: source_stats.1.to_string(),
            amount: 1,
        });
    }
}

pub fn deal_damage(
    mut commands: Commands,
    mut reader: EventReader<DealDamage>,
    mut messages: ResMut<Messages>,
    mut query: Query<(Entity, &mut Stats, &Name)>,
    player_entity: Res<Entity>,
) {
    let mut hm = HashMap::new();
    for event in reader.iter() {
        hm.insert(event.target, (event.source, event.amount, &event.name));
    }

    for (entity, mut stats, name) in query.iter_mut() {
        if let Some(found) = hm.get(&entity) {
            stats.hp.cur -= found.1;
            if found.0 == *player_entity {
                messages.add(format!("You hit {} for {} points", name.name, found.1));
            } else {
                messages.add(format!(
                    "{} hits {} for {} points",
                    found.2, name.name, found.1
                ));
            }
        }

        if stats.hp.cur < 0 {
            commands.entity(entity).despawn();
            messages.add("You killed it!");
        }
    }
}
