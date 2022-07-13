use bevy_ecs::prelude::*;
use bracket_lib::prelude::*;

use crate::components::*;
use crate::map::Map;

#[derive(Debug, Component)]
pub struct Viewshed {
    pub visible_tiles: Vec<Point>,
    range: i32,
}

impl Viewshed {
    pub fn new(range: i32) -> Self {
        Self {
            visible_tiles: Vec::new(),
            range,
        }
    }
}

pub fn visibility_system(map: Res<Map>, mut query: Query<(&mut Viewshed, &Position)>) {
    for mut vs in query.iter_mut() {
        vs.0.visible_tiles.clear();
        vs.0.visible_tiles = field_of_view(vs.1.point(), vs.0.range, &*map);
        vs.0.visible_tiles
            .retain(|p| p.x >= 0 && p.x < map.width && p.y >= 0 && p.y < map.height);
    }
}

pub fn map_update_system(mut map: ResMut<Map>, query: Query<(&Viewshed, &Player)>) {
    for q in query.iter() {
        for tile in q.0.visible_tiles.iter() {
            if map.xy_is_opaque(&tile) {
                map.remember(&tile);
            }
        }
    }
}
