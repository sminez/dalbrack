use risky_endevours::{tileset::TileSet, ui::Sdl2UI};
use sdl2::{event::Event, keyboard::Keycode, pixels::Color, rect::Rect};

pub fn main() -> anyhow::Result<()> {
    let mut ui = Sdl2UI::init(1080, 1000, "Risky Endevours")?;
    let mut ts = TileSet::df_rde()?;
    // let mut ts = TileSet::urizen()?;

    let mut col: u16 = 0;
    let mut row: u16 = 0;

    loop {
        if let Some(evt) = ui.next_event() {
            match evt {
                Event::Quit { .. } => return Ok(()),

                Event::KeyDown {
                    keycode: Some(k),
                    repeat: false,
                    ..
                } => match k {
                    Keycode::Right => col += 1,
                    Keycode::Left => col = col.saturating_sub(1),
                    Keycode::Up => row = row.saturating_sub(1),
                    Keycode::Down => row += 1,

                    Keycode::Space => ui.toggle_debug_bg(),
                    Keycode::Q | Keycode::Escape => return Ok(()),

                    _ => (),
                },

                _ => {}
            }
        }

        ui.clear();

        // show the tile itself
        let pos = ts.pos(row, col);
        ui.blit_tile(pos, Color::WHITE, &mut ts, Rect::new(50, 50, 100, 100))?;

        // show the coords
        let mut r = Rect::new(50, 200, 50, 50);
        for ch in format!("({row},{col})").chars() {
            ui.blit_tile(ts.tile(&ch.to_string()).unwrap(), Color::WHITE, &mut ts, r)?;
            r.x += 40;
        }

        // show the ident (if there is one)
        if let Some(ident) = ts.tile_name(pos) {
            let mut r = Rect::new(50, 250, 50, 50);
            for ch in ident.to_string().chars() {
                ui.blit_tile(ts.tile(&ch.to_string()).unwrap(), Color::WHITE, &mut ts, r)?;
                r.x += 40;
            }
        }

        ui.render()?;
    }
}
