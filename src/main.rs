use risky_endevours::{tileset::TileSet, ui::Sdl2UI};
use sdl2::{event::Event, keyboard::Keycode, pixels::Color, rect::Rect};

pub fn main() -> anyhow::Result<()> {
    let mut ui = Sdl2UI::init(1080, 1000, "Risky Endevours")?;
    let mut ts = TileSet::df_kruggsmash()?;

    let mut player = ts.tile(":)").unwrap();
    let tile_w: u32 = 30;
    let tile_h: u32 = 30;

    player.color = Color::RED;
    let mut r = Rect::new(0, 0, tile_w, tile_h);

    ui.clear();
    ui.blit_tile(&player, r, &mut ts)?;
    ui.render()?;

    loop {
        match ui.wait_event() {
            Event::Quit { .. } => return Ok(()),

            Event::KeyDown {
                keycode: Some(k),
                repeat: false,
                ..
            } => match k {
                Keycode::Right => r.x += tile_w as i32,
                Keycode::Left => r.x -= tile_w as i32,
                Keycode::Up => r.y -= tile_h as i32,
                Keycode::Down => r.y += tile_h as i32,

                Keycode::Space => {
                    r.x = 0;
                    r.y = 0;
                }

                Keycode::R => player.color = Color::RED,
                Keycode::G => player.color = Color::GREEN,
                Keycode::B => player.color = Color::BLUE,

                Keycode::Q | Keycode::Escape => return Ok(()),

                _ => continue,
            },

            _ => continue,
        }

        ui.clear();
        ui.blit_tile(&player, r, &mut ts)?;
        ui.render()?;
    }
}
