use bracket_lib::prelude::*;

embedded_resource!(WIDE_FONT, "../resources/terminal_10x16.png");
embedded_resource!(VGA_FONT, "../resources/vga8x16.png");
embedded_resource!(CHEEP_FONT, "../resources/cheepicus8x8.png");

const WIDTH: u32 = 100;
const HEIGHT: u32 = 75;

struct State {}
impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        for col in 0..WIDTH {
            ctx.print(col, 0, format!("{}", (col + 1) % 10));
        }
        for row in 1..HEIGHT {
            ctx.print(0, row, format!("{row}"));
        }
    }
}

fn main() -> BError {
    link_resource!(WIDE_FONT, "../resources/terminal_10x16.png");
    link_resource!(VGA_FONT, "../resources/vga8x16.png");
    link_resource!(CHEEP_FONT, "../resources/cheepicus8x8.png");

    let context = BTermBuilder::new()
        .with_resource_path("../resources")
        .with_simple_console(WIDTH, HEIGHT, "cheepicus8x8.png")
        .with_title("Roguelike Tutorial")
        .with_font("cheepicus8x8.png", 8, 8)
        .with_tile_dimensions(10, 12)
        .build()?;

    let gs = State {};
    main_loop(context, gs)
}
