use risky_endevours::{tileset::TileSet, ui::Sdl2UI};
use sdl2::{event::Event, keyboard::Keycode, pixels::Color, rect::Rect};

pub fn main() -> anyhow::Result<()> {
    let mut ui = Sdl2UI::init(1080, 1000, "Risky Endevours")?;
    let mut ts = TileSet::urizen()?;

    let player_tile = ts.map_tile("player", 17, 0);
    let tile_w: u32 = 30;
    let tile_h: u32 = 30;

    let mut color = Color::RED;
    let mut r = Rect::new(0, 0, tile_w, tile_h);

    loop {
        if let Some(evt) = ui.next_event() {
            match evt {
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

                    Keycode::R => color = Color::RED,
                    Keycode::G => color = Color::GREEN,
                    Keycode::B => color = Color::BLUE,

                    Keycode::Q | Keycode::Escape => return Ok(()),

                    _ => (),
                },

                _ => {}
            }
        }

        ui.clear();
        ui.blit_tile(player_tile, color, &mut ts, r)?;
        ui.render()?;
    }
}
