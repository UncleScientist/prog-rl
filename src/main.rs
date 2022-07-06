use std::fmt::Display;

use bevy_ecs::prelude::*;

use bracket_lib::prelude::*;

mod map;
use map::*;

embedded_resource!(WIDE_FONT, "../resources/terminal_10x16.png");
embedded_resource!(VGA_FONT, "../resources/vga8x16.png");
embedded_resource!(CHEEP_FONT, "../resources/cheepicus8x8.png");

const WIDTH: i32 = 40;
const HEIGHT: i32 = 25;

type Viewport = Rect;
type MapOffset = Point;

pub enum RunState {
    WelcomeScreen,
    StartGame,
}

struct State {
    ecs: World,
    display: RunState,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls();

        match self.display {
            RunState::WelcomeScreen => {
                self.center_at_row(ctx, 2, "Welcome to \"Prog-Rog\"");
                self.center_at_row(ctx, 3, "A Programmable Roguelike");

                self.center_at_row(ctx, 5, "Press ENTER to Start");
                if let Some(VirtualKeyCode::Return) = ctx.key {
                    self.display = RunState::StartGame;
                }
            }

            RunState::StartGame => {
                let cell = self.ecs.cell();

                ctx.print(0, 0, format!("{}", ctx.fps));

                let map = &*cell.get_resource::<Map>().unwrap();
                let vp = &*cell.get_resource::<Viewport>().unwrap();
                {
                    let p = &*cell.get_resource::<MapOffset>().unwrap();
                    map.draw(ctx, p.x, p.y, vp);
                }

                if let Some(key) = ctx.key {
                    match key {
                        VirtualKeyCode::Right => {
                            let mut mo = cell.get_resource_mut::<MapOffset>().unwrap();
                            if mo.x + vp.x2 - vp.x1 < map.width {
                                mo.x += 1;
                            }
                        }
                        VirtualKeyCode::Left => {
                            let mut mo = cell.get_resource_mut::<MapOffset>().unwrap();
                            if mo.x > 0 {
                                mo.x -= 1;
                            }
                        }
                        VirtualKeyCode::Down => {
                            let mut mo = cell.get_resource_mut::<MapOffset>().unwrap();
                            if mo.y + vp.y2 - vp.y1 < map.height {
                                mo.y += 1;
                            }
                        }
                        VirtualKeyCode::Up => {
                            let mut mo = cell.get_resource_mut::<MapOffset>().unwrap();
                            if mo.y > 0 {
                                mo.y -= 1;
                            }
                        }
                        _ => {}
                    }
                }

                let mo = &*cell.get_resource::<MapOffset>().unwrap();
                ctx.print(5, 0, format!("{},{}", mo.x, mo.y));
            }
        }
    }
}

impl State {
    fn center_at_row<D: Display>(&self, ctx: &mut BTerm, row: i32, message: D) {
        let s = format!("{message}");
        let col = WIDTH / 2 - s.len() as i32 / 2;
        ctx.print(col, row, s);
    }
}

fn main() -> BError {
    link_resource!(WIDE_FONT, "../resources/terminal_10x16.png");
    link_resource!(VGA_FONT, "../resources/vga8x16.png");
    link_resource!(CHEEP_FONT, "../resources/cheepicus8x8.png");

    let context = BTermBuilder::new()
        .with_resource_path("../resources")
        .with_simple_console(WIDTH, HEIGHT, "cheepicus8x8.png")
        .with_title("ProgRog")
        .with_font("cheepicus8x8.png", 8, 8)
        .with_fps_cap(10.0)
        .build()?;

    let mut gs = State {
        ecs: World::new(),
        display: RunState::WelcomeScreen,
    };

    gs.ecs.insert_resource(RandomNumberGenerator::new());
    gs.ecs.insert_resource(Viewport::with_size(1, 1, 38, 23));
    gs.ecs.insert_resource(MapOffset::new(0, 0));

    let map = Map::generate(&mut gs.ecs, WIDTH * 5, HEIGHT * 5);
    gs.ecs.insert_resource(map);

    main_loop(context, gs)
}
