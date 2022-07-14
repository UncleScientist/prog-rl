use bevy_ecs::prelude::*;

use bracket_lib::prelude::*;

use crate::components::*;

pub struct Map {
    pub tiles: Vec<TileType>,
    pub memory: Vec<bool>,
    pub width: i32,
    pub height: i32,
    start_x: i32,
    start_y: i32,
}

#[derive(Copy, Clone, PartialEq)]
pub enum TileType {
    Floor,
    Wall,
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

        let mut i = rooms.iter();
        let mut prev_room = i.next().unwrap();
        for room in i {
            let center_prev = prev_room.center();
            let center_cur = room.center();
            match rng.range(0, 2) {
                0 => {
                    for col in center_prev.x.min(center_cur.x)..center_prev.x.max(center_cur.x) {
                        map.tiles[(col + center_cur.y * width) as usize] = TileType::Floor;
                    }
                    for row in center_prev.y.min(center_cur.y)..center_prev.y.max(center_cur.y) {
                        map.tiles[(center_prev.x + row * width) as usize] = TileType::Floor;
                    }
                }
                _ => {
                    for col in center_prev.x.min(center_cur.x)..center_prev.x.max(center_cur.x) {
                        map.tiles[(col + center_prev.y * width) as usize] = TileType::Floor;
                    }
                    for row in center_prev.y.min(center_cur.y)..center_prev.y.max(center_cur.y) {
                        map.tiles[(center_cur.x + row * width) as usize] = TileType::Floor;
                    }
                }
            }
            prev_room = room;
        }

        for room in rooms {
            room.for_each(|point| {
                map.tiles[(point.x + point.y * width) as usize] = TileType::Floor
            });
        }

        map
    }
}

impl Map {
    pub fn new(width: i32, height: i32, start_x: i32, start_y: i32) -> Self {
        Self {
            tiles: vec![TileType::Wall; (width * height) as usize],
            memory: vec![false; (width * height) as usize],
            width,
            height,
            start_x,
            start_y,
        }
    }

    // start_x, start_y: upper left corner of map
    // viewport: where to draw on the screen
    pub fn _draw(&self, ctx: &mut BTerm, offset: &Position, viewport: &Rect) {
        viewport.for_each(|point| {
            let map_x = point.x - viewport.x1 + offset.x;
            let map_y = point.y - viewport.y1 + offset.y;
            let idx = map_x + map_y * self.width;
            if self.tiles[idx as usize] == TileType::Floor {
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
        self.tiles[idx as usize] == TileType::Floor
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn xy_is_opaque(&self, p: &Point) -> bool {
        self.is_opaque((p.x + p.y * self.width) as usize)
    }

    pub fn idx_to_xy_point(&self, idx: usize) -> Point {
        Point {
            x: idx as i32 % self.width,
            y: idx as i32 / self.width,
        }
    }

    pub fn remember(&mut self, p: &Point) {
        self.memory[(p.x + p.y * self.width) as usize] = true;
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        if idx >= (self.width * self.height) as usize {
            panic!("bug in bracket-lib");
        } else {
            self.tiles[idx as usize] != TileType::Floor
        }
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}
