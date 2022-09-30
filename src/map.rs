use bevy_ecs::prelude::*;

use bracket_lib::prelude::*;

use crate::components::*;

pub struct Map {
    pub tiles: Vec<TileType>,
    pub memory: Vec<bool>,
    pub entity: Vec<Vec<Entity>>,
    pub width: i32,
    pub height: i32,
    start_x: i32,
    start_y: i32,
}

pub enum Direction {
    North,
    South,
    East,
    West,
    NorthWest,
    NorthEast,
    SouthWest,
    SouthEast,
}

#[derive(Copy, Clone, PartialEq)]
pub enum TileType {
    Floor,
    Wall,
}

pub struct MapFactory<'a> {
    builders: Vec<&'a dyn MapGenerator>,
}

impl<'a> MapFactory<'a> {
    pub fn new() -> Self {
        Self {
            builders: Vec::new(),
        }
    }

    pub fn create_map(&self, ecs: &mut World, width: i32, height: i32) -> Map {
        let mut rng = ecs.get_resource_mut::<RandomNumberGenerator>().unwrap();
        let room_type = rng.range(0, self.builders.len());
        console::log(room_type);

        self.builders[room_type].generate(ecs, width, height)
    }

    pub fn add_builder(&mut self, builder: &'a dyn MapGenerator) {
        self.builders.push(builder);
    }
}

pub trait MapGenerator: Send + Sync {
    fn generate(&self, ecs: &mut World, width: i32, height: i32) -> Map;
}

pub struct RectRoomMapGenerator;

pub struct RoundRoomMapGenerator;

//const ROOM_TYPES: [&dyn MapGenerator; 2] = [&RectRoomMapGenerator, &RectRoomMapGenerator];

const MIN_WIDTH: i32 = 3;
const MAX_WIDTH: i32 = 15;
const MIN_HEIGHT: i32 = 3;
const MAX_HEIGHT: i32 = 12;

const ROOM_COUNT: usize = 20;

impl MapGenerator for RoundRoomMapGenerator {
    fn generate(&self, ecs: &mut World, width: i32, height: i32) -> Map {
        let mut rng = ecs.get_resource_mut::<RandomNumberGenerator>().unwrap();
        let rooms = non_overlapping_rooms(&mut rng, width, height);

        let Point {
            x: start_x,
            y: start_y,
        } = rooms[0].center();
        let mut map = Map::new(width, height, start_x, start_y);

        for room in &rooms {
            let radius = 2 + ((room.x1 - room.x2).abs()).min((room.y1 - room.y2).abs()) / 2;

            let center_pt = room.center();
            for y in center_pt.y - radius..center_pt.y + radius {
                for x in center_pt.x - radius..center_pt.x + radius {
                    let point = Point::new(x, y);
                    if let Some(idx) = map.point_to_idx(&point) {
                        let distance = DistanceAlg::Pythagoras.distance2d(center_pt, point);
                        if idx > 0
                            && idx < ((map.width * map.height) - 1) as usize
                            && distance <= (radius as f32)
                        {
                            map.tiles[idx] = TileType::Floor;
                        }
                    }
                }
            }
        }

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

        map
    }
}

impl MapGenerator for RectRoomMapGenerator {
    fn generate(&self, ecs: &mut World, width: i32, height: i32) -> Map {
        let mut rng = ecs.get_resource_mut::<RandomNumberGenerator>().unwrap();

        let rooms = non_overlapping_rooms(&mut rng, width, height);
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
            entity: vec![Vec::new(); (width * height) as usize],
            width,
            height,
            start_x,
            start_y,
        }
    }

    pub fn new_position(&self, dir: Direction, old_position: &Position) -> Position {
        let mut newpos = *old_position;

        match dir {
            Direction::North => {
                newpos.y = 0.max(newpos.y - 1);
            }
            Direction::South => {
                newpos.y = (self.height - 1).min(newpos.y + 1);
            }
            Direction::East => {
                newpos.x = (self.width - 1).min(newpos.x + 1);
            }
            Direction::West => {
                newpos.x = 0.max(newpos.x - 1);
            }
            Direction::NorthWest => {
                newpos.y = 0.max(newpos.y - 1);
                newpos.x = 0.max(newpos.x - 1);
            }
            Direction::NorthEast => {
                newpos.y = 0.max(newpos.y - 1);
                newpos.x = (self.width - 1).min(newpos.x + 1);
            }
            Direction::SouthWest => {
                newpos.y = (self.height - 1).min(newpos.y + 1);
                newpos.x = 0.max(newpos.x - 1);
            }
            Direction::SouthEast => {
                newpos.y = (self.height - 1).min(newpos.y + 1);
                newpos.x = (self.width - 1).min(newpos.x + 1);
            }
        }

        newpos
    }

    pub fn add_entity(&mut self, p: &Position, id: Entity) {
        let idx = self.pos_to_idx(p);
        self.entity[idx].push(id);
    }

    pub fn move_entity(&mut self, old_pos: &Position, new_pos: &Position, id: Entity) {
        let old = self.pos_to_idx(old_pos);
        let new = self.pos_to_idx(new_pos);
        self.entity[old].retain(|x| *x != id);
        self.entity[new].push(id);
    }

    pub fn try_walk(&self, p: &Position) -> Option<&Entity> {
        let idx = self.pos_to_idx(p);
        self.entity[idx].last()
    }

    pub fn can_move_mob(&self, world: &World, new_pos: &Position) -> bool {
        let idx = self.pos_to_idx(new_pos);
        if !self.walkable(new_pos) {
            return false;
        }

        for p in self.entity[idx].iter() {
            if let Some(e) = world.get_entity(*p) {
                if e.get::<Mob>().is_some() {
                    return false;
                }
            }
        }
        true
    }

    pub fn center_of(&self) -> Position {
        Position {
            x: self.start_x,
            y: self.start_y,
        }
    }

    pub fn walkable(&self, pos: &Position) -> bool {
        let idx = pos.x + pos.y * self.width;
        self.tiles[idx as usize] == TileType::Floor
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn pos_to_idx(&self, p: &Position) -> usize {
        (p.x + p.y * self.width) as usize
    }

    pub fn point_to_idx(&self, p: &Point) -> Option<usize> {
        if p.x < 0 || p.x >= self.width || p.y < 0 || p.y >= self.height {
            None
        } else {
            Some((p.x + p.y * self.width) as usize)
        }
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

fn non_overlapping_rooms(rng: &mut RandomNumberGenerator, width: i32, height: i32) -> Vec<Rect> {
    let mut rooms = Vec::new();
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
    rooms
}
