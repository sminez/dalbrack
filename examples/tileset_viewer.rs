use risky_endevours::{tileset::TileSet, ui::Sdl2UI};
use sdl2::{event::Event, keyboard::Keycode, pixels::Color, rect::Rect};

pub fn main() -> anyhow::Result<()> {
    let mut ui = Sdl2UI::init(1080, 1000, "Risky Endevours")?;
    let mut ts = TileSet::urizen()?;

    let mut col: u32 = 0;
    let mut row: u32 = 0;

    loop {
        if let Some(evt) = ui.next_event() {
            match evt {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape | Keycode::Q),
                    ..
                } => return Ok(()),

                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    repeat: false,
                    ..
                } => {
                    row = 0;
                    col = 0;
                }

                Event::KeyDown {
                    keycode: Some(k),
                    repeat: false,
                    ..
                } => match k {
                    Keycode::Right => col += 1,
                    Keycode::Left => col = col.saturating_sub(1),
                    Keycode::Up => row = row.saturating_sub(1),
                    Keycode::Down => row += 1,
                    _ => (),
                },

                _ => {}
            }
        }

        ui.blit_pos(
            ts.pos(row, col),
            Color::RGB(255, 0, 0),
            &mut ts,
            Rect::new(50, 50, 100, 100),
        )?;
        ui.render()?;
    }
}
