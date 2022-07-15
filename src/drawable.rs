use crate::components::*;
use bracket_lib::prelude::*;

pub struct DrawList {
    pub items: Vec<Drawable>,
}

#[derive(Copy, Clone)]
pub struct Drawable {
    pub pos: Point,
    pub glyph: char,
    pub priority: usize,
}

impl Drawable {
    pub fn new(pos: &Position, glyph: char, priority: usize) -> Self {
        Self {
            pos: pos.into(),
            glyph,
            priority,
        }
    }
}
