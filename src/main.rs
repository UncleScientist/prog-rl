use bevy_ecs::event::Events;
use bevy_ecs::prelude::*;
use bevy_ecs::schedule::ShouldRun;

use bracket_lib::prelude::*;

mod combat;
use combat::*;

mod game_state;
use game_state::*;

mod keyboard;
use keyboard::*;

mod drawable;
use drawable::*;

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

fn run_if_player_performed_an_action(rs: Res<RunSystems>) -> ShouldRun {
    if rs.run_systems {
        ShouldRun::Yes
    } else {
        ShouldRun::No
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
            SystemStage::parallel()
                .with_system(Events::<KeyboardEvent>::update_system)
                .with_system(Events::<DealDamage>::update_system)
                .with_system(Events::<MeleeEvent>::update_system),
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
            "reset",
            SystemStage::parallel()
                .with_run_criteria(run_if_player_performed_an_action)
                .with_system(clear_screen),
        )
        .with_stage(
            "AI",
            SystemStage::parallel()
                .with_run_criteria(run_if_player_performed_an_action)
                .with_system(move_mobs),
        )
        .with_stage(
            "combat",
            SystemStage::parallel()
                .with_system(resolve_combat)
                .with_system(deal_damage.after(resolve_combat)),
        )
        .with_stage(
            "update",
            SystemStage::single_threaded()
                .with_run_criteria(run_if_player_performed_an_action)
                .with_system(draw_map)
                .with_system(draw_mobs.after(move_mobs)),
        )
        .with_stage(
            "cleanup",
            SystemStage::parallel().with_system(clear_run_flag),
        );

    let mut gs = crate::game_state::State::new(
        World::new(),
        RunState::WelcomeScreen,
        schedule,
        WIDTH,
        HEIGHT,
    );

    gs.ecs.init_resource::<Events<KeyboardEvent>>();
    gs.ecs.init_resource::<Events<MeleeEvent>>();
    gs.ecs.init_resource::<Events<DealDamage>>();
    gs.ecs.insert_resource(RandomNumberGenerator::new());
    gs.ecs.insert_resource(Viewport::with_size(1, 1, 37, 22));
    gs.ecs.insert_resource(DrawList { items: Vec::new() });
    gs.ecs.insert_resource(RunSystems { run_systems: true });

    let mut map = MapGenerator::generate(&mut gs.ecs, WIDTH * 2, HEIGHT * 2);
    let starting_position = map.center_of();

    // temporarly spawn some mobs
    let mut count = 0;
    let mut rng = RandomNumberGenerator::new();
    while count < 10 {
        let x = rng.range(0, map.width());
        let y = rng.range(0, map.height());
        if map.walkable(x, y) {
            let p = Position { x, y };
            let id = gs
                .ecs
                .spawn()
                .insert(Mob { glyph: 'r' })
                .insert(Stats::new(2, 2))
                .insert(Viewshed::new(2))
                .insert(p)
                .id();
            map.add_entity(&p, id);
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

fn draw_mobs(
    mut draw_list: ResMut<DrawList>,
    query: Query<(&Position, Option<&Mob>, Option<&Player>, &Viewshed)>,
) {
    let mut vs = None;
    let mut to_draw = Vec::new();

    for (position, mob, player, viewshed) in query.iter() {
        if player.is_some() {
            vs = Some(viewshed);
        } else if let Some(mob) = mob {
            to_draw.push(Drawable::new(position, mob.glyph, 1));
        }
    }

    if let Some(vs) = vs {
        to_draw.retain(|x| vs.visible_tiles.contains(&x.pos));
        draw_list.items.append(&mut to_draw);
    }
}

fn clear_screen(mut draw_list: ResMut<DrawList>) {
    draw_list.items.clear();
}

fn draw_map(mut draw_list: ResMut<DrawList>, map: Res<Map>, query: Query<(&Player, &Viewshed)>) {
    for (_, viewshed) in query.iter() {
        for point in &viewshed.visible_tiles {
            draw_list.items.push(Drawable::new(
                &point.into(),
                if map.xy_is_opaque(point) { '#' } else { '.' },
                0,
            ));
        }
    }
    for (e, &v) in map.memory.iter().enumerate() {
        if v {
            let p = map.idx_to_xy_point(e);
            draw_list.items.push(Drawable::new(
                &(&p).into(),
                if map.xy_is_opaque(&p) { '#' } else { '.' },
                0,
            ));
        }
    }
}

fn move_mobs(
    mut rng: ResMut<RandomNumberGenerator>,
    mut map: ResMut<Map>,
    mut query: Query<(Entity, &mut Position, &Mob)>,
) {
    for (id, mut position, _) in query.iter_mut() {
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
                map.move_entity(&*position, &new_pos, id);
                *position = new_pos;
            }
        }
    }
}

fn clear_run_flag(mut rs: ResMut<RunSystems>) {
    rs.run_systems = false;
}
