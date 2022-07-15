use std::fmt::Display;

use bevy_ecs::event::Events;
use bevy_ecs::prelude::*;
use bracket_lib::prelude::*;

use crate::components::*;
use crate::drawable::*;
use crate::keyboard::*;

pub struct RunSystems {
    pub run_systems: bool,
}

pub enum RunState {
    WelcomeScreen,
    StartGame,
}

pub type Viewport = Rect;

pub struct State {
    pub ecs: World,
    display: RunState,
    schedule: Schedule,
    screen_width: i32,
    screen_height: i32,
}

impl State {
    pub fn new(
        ecs: World,
        display: RunState,
        schedule: Schedule,
        screen_width: i32,
        screen_height: i32,
    ) -> Self {
        Self {
            ecs,
            display,
            schedule,
            screen_width,
            screen_height,
        }
    }

    pub fn center_at_row<D: Display>(&self, ctx: &mut BTerm, row: i32, message: D) {
        let s = format!("{message}");
        let col = self.screen_width / 2 - s.len() as i32 / 2;
        ctx.print(col, row, s);
    }
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
                    x: 0.max(p.x - self.screen_width / 2),
                    y: 0.max(p.y - self.screen_height / 2),
                };

                let stats = player_ref.get::<Stats>().unwrap();
                ctx.print(
                    0,
                    self.screen_height - 1,
                    format!("HP:{} MP:{}", stats.hp.cur, stats.mp.cur),
                );

                let drawables = self.ecs.get_resource::<DrawList>().unwrap();
                let mut draw = drawables.items.to_vec();
                draw.sort_by(|a, b| a.priority.cmp(&b.priority));

                for d in &draw {
                    let point = Point {
                        x: d.pos.x - offset.x + vp.x1,
                        y: d.pos.y - offset.y + vp.y1,
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
