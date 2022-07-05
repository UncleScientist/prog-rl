use specs::prelude::*;

use bracket_lib::prelude::*;

pub struct Map {
    pub width: i32,
    pub height: i32,

    rooms: Vec<Rect>,
}

const MIN_WIDTH: i32 = 3;
const MAX_WIDTH: i32 = 15;
const MIN_HEIGHT: i32 = 3;
const MAX_HEIGHT: i32 = 12;

const ROOM_COUNT: usize = 20;

impl Map {
    pub fn generate(ecs: &mut World, width: i32, height: i32) -> Self {
        let mut rooms = Vec::new();

        let rng = ecs.get_mut::<RandomNumberGenerator>().unwrap();

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

        Self {
            width,
            height,
            rooms,
        }
    }

    // start_x, start_y: upper left corner of map
    // viewport: where to draw on the screen
    pub fn draw(&self, ctx: &mut BTerm, start_x: i32, start_y: i32, viewport: &Rect) {
        for (e, room) in self.rooms.iter().enumerate() {
            let offset = Rect::with_exact(
                room.x1 - start_x,
                room.y1 - start_y,
                room.x2 - start_x,
                room.y2 - start_y,
            );

            if !offset.intersect(viewport) {
                continue;
            }

            let top_row = offset.y1.max(viewport.y1);
            let bot_row = offset.y2.min(viewport.y2);

            let left_col = offset.x1.max(viewport.x1);
            let right_col = offset.x2.min(viewport.x2);

            for row in top_row..bot_row {
                ctx.print(left_col, row, ".".repeat((right_col - left_col) as usize));
            }
            ctx.print(left_col, top_row, &format!("{e}"));
        }
    }
}
