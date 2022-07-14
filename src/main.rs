use std::fmt::Display;

use bevy_ecs::event::Events;
use bevy_ecs::prelude::*;
use bevy_ecs::schedule::ShouldRun;

use bracket_lib::prelude::*;

mod components;
use components::*;

mod map;
use map::*;

mod system;
use system::*;

embedded_resource!(WIDE_FONT, "../resources/terminal_10x16.png");
embedded_resource!(VGA_FONT, "../resources/vga8x16.png");
embedded_resource!(CHEEP_FONT, "../resources/cheepicus8x8.png");

const WIDTH: i32 = 40;
const HEIGHT: i32 = 25;

type Viewport = Rect;

pub enum RunState {
    WelcomeScreen,
    StartGame,
}

struct State {
    ecs: World,
    display: RunState,
    schedule: Schedule,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        self.schedule.run(&mut self.ecs);

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
                if let Some(key) = ctx.key {
                    let mut events = self
                        .ecs
                        .get_resource_mut::<Events<KeyboardEvent>>()
                        .unwrap();
                    events.send(KeyboardEvent(key));
                }

                let vp = self.ecs.get_resource::<Viewport>().unwrap();
                let player = self.ecs.get_resource::<Entity>().unwrap();
                let player_ref = self.ecs.entity(*player);
                let p = player_ref.get::<Position>().unwrap();
                let offset = Position {
                    x: 0.max(p.x - WIDTH / 2),
                    y: 0.max(p.y - HEIGHT / 2),
                };

                let stats = player_ref.get::<Stats>().unwrap();
                ctx.print(
                    0,
                    HEIGHT - 1,
                    format!("HP:{} MP:{}", stats.hp.cur, stats.mp.cur),
                );

                let drawables = self.ecs.get_resource::<DrawList>().unwrap();
                for d in &drawables.items {
                    let point = Point {
                        x: d.x - offset.x + vp.x1,
                        y: d.y - offset.y + vp.y1,
                    };
                    if vp.point_in_rect(point) {
                        ctx.print(point.x, point.y, d.glyph);
                    }
                }

                ctx.print(p.x - offset.x + vp.x1, p.y - offset.y + vp.y1, '@');
            }
        }
    }
}

struct DrawList {
    items: Vec<Drawable>,
}

struct Drawable {
    x: i32,
    y: i32,
    glyph: char,
}

impl Drawable {
    pub fn new(pos: &Position, glyph: char) -> Self {
        Self {
            x: pos.x,
            y: pos.y,
            glyph,
        }
    }
}

struct RunSystems {
    run_systems: bool,
}

fn run_if_player_performed_an_action(rs: Res<RunSystems>) -> ShouldRun {
    if rs.run_systems {
        ShouldRun::Yes
    } else {
        ShouldRun::No
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

    let schedule = Schedule::default()
        .with_stage(
            "clear",
            SystemStage::parallel().with_system(Events::<KeyboardEvent>::update_system),
        )
        .with_stage("player", SystemStage::parallel().with_system(handle_key))
        .with_stage(
            "viewshed",
            SystemStage::parallel()
                .with_run_criteria(run_if_player_performed_an_action)
                .with_system(visibility_system)
                .with_system(map_update_system),
        )
        .with_stage(
            "update",
            SystemStage::single_threaded()
                .with_run_criteria(run_if_player_performed_an_action)
                .with_system(move_mobs)
                .with_system(draw_map)
                .with_system(draw_mobs),
        )
        .with_stage(
            "cleanup",
            SystemStage::parallel().with_system(clear_run_flag),
        );

    let mut gs = State {
        ecs: World::new(),
        display: RunState::WelcomeScreen,
        schedule,
    };

    gs.ecs.init_resource::<Events<KeyboardEvent>>();
    gs.ecs.insert_resource(RandomNumberGenerator::new());
    gs.ecs.insert_resource(Viewport::with_size(1, 1, 37, 22));
    gs.ecs.insert_resource(DrawList { items: Vec::new() });
    gs.ecs.insert_resource(RunSystems { run_systems: true });

    let map = MapGenerator::generate(&mut gs.ecs, WIDTH * 2, HEIGHT * 2);
    let starting_position = map.center_of();

    // temporarly spawn some mobs
    let mut count = 0;
    let mut rng = RandomNumberGenerator::new();
    while count < 10 {
        let x = rng.range(0, map.width());
        let y = rng.range(0, map.height());
        if map.walkable(x, y) {
            gs.ecs
                .spawn()
                .insert(Mob { glyph: 'r' })
                .insert(Stats::new(2, 2))
                .insert(Position { x, y });
            count += 1;
        }
    }

    gs.ecs.insert_resource(map);

    let player = gs
        .ecs
        .spawn()
        .insert(Player {})
        .insert(starting_position)
        .insert(Viewshed::new(5))
        .insert(Stats::new(10, 10))
        .id();
    gs.ecs.insert_resource(player);

    main_loop(context, gs)
}

struct KeyboardEvent(VirtualKeyCode);

fn handle_key(
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

fn draw_mobs(mut draw_list: ResMut<DrawList>, query: Query<(&Position, &Mob)>) {
    for (position, mob) in query.iter() {
        draw_list.items.push(Drawable::new(position, mob.glyph));
    }
}

fn draw_map(mut draw_list: ResMut<DrawList>, map: Res<Map>, query: Query<(&Player, &Viewshed)>) {
    draw_list.items.clear();
    for (_, viewshed) in query.iter() {
        for point in &viewshed.visible_tiles {
            draw_list.items.push(Drawable::new(
                &point.into(),
                if map.xy_is_opaque(point) { '#' } else { '.' },
            ));
        }
    }
    for (e, &v) in map.memory.iter().enumerate() {
        if v {
            let p = map.idx_to_xy_point(e);
            draw_list.items.push(Drawable::new(
                &(&p).into(),
                if map.xy_is_opaque(&p) { '#' } else { '.' },
            ));
        }
    }
}

fn move_mobs(
    mut rng: ResMut<RandomNumberGenerator>,
    map: Res<Map>,
    mut query: Query<(&mut Position, &Mob)>,
) {
    for (mut position, _) in query.iter_mut() {
        if let Some(new_pos) = match rng.range(0, 4) {
            0 => {
                if position.x > 0 {
                    Some(Position {
                        x: position.x - 1,
                        y: position.y,
                    })
                } else {
                    None
                }
            }
            1 => {
                if position.x < map.width - 1 {
                    Some(Position {
                        x: position.x + 1,
                        y: position.y,
                    })
                } else {
                    None
                }
            }
            2 => {
                if position.y > 0 {
                    Some(Position {
                        x: position.x,
                        y: position.y - 1,
                    })
                } else {
                    None
                }
            }
            3 => {
                if position.y < map.height - 1 {
                    Some(Position {
                        x: position.x,
                        y: position.y + 1,
                    })
                } else {
                    None
                }
            }
            _ => panic!("rng failure"),
        } {
            if map.walkable(new_pos.x, new_pos.y) {
                *position = new_pos;
            }
        }
    }
}

fn clear_run_flag(mut rs: ResMut<RunSystems>) {
    rs.run_systems = false;
}
