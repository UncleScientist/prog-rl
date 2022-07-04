use bracket_lib::prelude::*;

embedded_resource!(WIDE_FONT, "resources/terminal_10x16.png");
embedded_resource!(VGA_FONT, "resources/vga8x16.png");

struct State {}
impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print(1, 1, "hello, world");
    }
}

fn main() -> BError {
    link_resource!(WIDE_FONT, "resources/terminal_10x16.png");
    link_resource!(VGA_FONT, "resources/vga8x16.png");

    let context = BTermBuilder::new()
        .with_simple_console(80, 50, "terminal_10x16.png")
        .with_title("Roguelike Tutorial")
        .with_font("terminal_10x16.png", 10, 16)
        .with_tile_dimensions(10, 16)
        .build()?;
    let gs = State {};
    main_loop(context, gs)
}
