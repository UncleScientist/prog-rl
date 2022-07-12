use bevy_ecs::prelude::*;

use bracket_lib::prelude::*;

use crate::components::*;

pub struct Map {
    pub tiles: Vec<bool>,
    pub width: i32,
    pub height: i32,
    start_x: i32,
    start_y: i32,
}

pub struct MapGenerator {
    pub _width: i32,
    pub _height: i32,

    _rooms: Vec<Rect>,
}

const MIN_WIDTH: i32 = 3;
const MAX_WIDTH: i32 = 15;
const MIN_HEIGHT: i32 = 3;
const MAX_HEIGHT: i32 = 12;

const ROOM_COUNT: usize = 20;

impl MapGenerator {
    pub fn generate(ecs: &mut World, width: i32, height: i32) -> Map {
        let mut rooms = Vec::new();

        let mut rng = ecs.get_resource_mut::<RandomNumberGenerator>().unwrap();

        'next: while rooms.len() < ROOM_COUNT {
            let x1 = rng.range(0, width - MAX_WIDTH);
            let y1 = rng.range(0, height - MAX_HEIGHT);
            let w = MIN_WIDTH + rng.range(0, MAX_WIDTH - MIN_WIDTH);
            let h = MIN_HEIGHT + rng.range(0, MAX_HEIGHT - MIN_HEIGHT);
            let new_r = Rect::with_size(x1, y1, w, h);

            for r in &rooms {
                if new_r.intersect(r) {
                    // try a new rect instead of saving this one
                    continue 'next;
                }
            }

            rooms.push(new_r);
        }

        let center = rooms[0].center();
        let mut map = Map::new(width, height, center.x, center.y);

        for room in rooms {
            room.for_each(|point| map.tiles[(point.x + point.y * width) as usize] = true);
        }

        map
    }
}

impl Map {
    pub fn new(width: i32, height: i32, start_x: i32, start_y: i32) -> Self {
        Self {
            tiles: vec![false; (width * height) as usize],
            width,
            height,
            start_x,
            start_y,
        }
    }

    // start_x, start_y: upper left corner of map
    // viewport: where to draw on the screen
    pub fn draw(&self, ctx: &mut BTerm, offset: &Position, viewport: &Rect) {
        viewport.for_each(|point| {
            let map_x = point.x - viewport.x1 + offset.x;
            let map_y = point.y - viewport.y1 + offset.y;
            let idx = map_x + map_y * self.width;
            if self.tiles[idx as usize] {
                ctx.print(point.x, point.y, ".");
            }
        });
    }

    pub fn center_of(&self) -> Position {
        Position {
            x: self.start_x,
            y: self.start_y,
        }
    }

    pub fn walkable(&self, x: i32, y: i32) -> bool {
        let idx = x + y * self.width;
        self.tiles[idx as usize]
    }
}
