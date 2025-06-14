use risky_endevours::{tileset::TileSet, ui::Sdl2UI};
use sdl2::{event::Event, keyboard::Keycode, pixels::Color};

const DXY: u32 = 30;

pub fn main() -> anyhow::Result<()> {
    let mut ui = Sdl2UI::init(1080, 1000, DXY, "Risky Endevours")?;
    ui.ts = TileSet::df_kruggsmash()?;

    let mut player = ui.ts.tile(":)").unwrap();

    player.color = Color::RED;
    let mut x = 0;
    let mut y = 0;

    ui.clear();
    ui.blit_tile(&player, x, y)?;
    ui.render()?;

    loop {
        match ui.wait_event() {
            Event::Quit { .. } => return Ok(()),

            Event::KeyDown {
                keycode: Some(k),
                repeat: false,
                ..
            } => match k {
                Keycode::Right => x += 1,
                Keycode::Left => x -= 1,
                Keycode::Up => y -= 1,
                Keycode::Down => y += 1,

                Keycode::Space => {
                    x = 0;
                    y = 0;
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
        ui.blit_tile(&player, x, y)?;
        ui.render()?;
    }
}
