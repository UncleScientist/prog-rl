use bevy_ecs::prelude::*;
use bracket_lib::prelude::*;

use crate::components::*;
use crate::keyboard::*;

pub struct DealDamage {
    pub entity: Entity,
    pub amount: i32,
}

pub fn resolve_combat(
    mut reader: EventReader<MeleeEvent>,
    mut writer: EventWriter<DealDamage>,
    player: Res<Entity>,
    query: Query<(Entity, &Position, &Stats)>,
) {
    for event in reader.iter() {
        // find the player
        let mut player_stats = None;
        for (entity, _, stats) in query.iter() {
            if entity == *player {
                player_stats = Some(*stats);
                break;
            }
        }

        if let Some(pstat) = player_stats {
            for (entity, _position, _stats) in query.iter() {
                if entity == event.target {
                    writer.send(DealDamage {
                        entity,
                        amount: pstat.hp.cur / 10,
                    });
                }
            }
        }
    }
}

pub fn deal_damage(
    mut commands: Commands,
    mut reader: EventReader<DealDamage>,
    mut query: Query<(Entity, &mut Stats)>,
) {
    for event in reader.iter() {
        for (entity, mut stats) in query.iter_mut() {
            if event.entity == entity {
                stats.hp.cur -= event.amount;
                console::log(format!("mob attacked for {} points", event.amount));
            }

            if stats.hp.cur < 0 {
                commands.entity(entity).despawn();
                console::log("mob despawned");
            }
        }
    }
}
