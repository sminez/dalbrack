use risky_endevours::{tileset::TileSet, ui::Sdl2UI};
use sdl2::{event::Event, keyboard::Keycode, rect::Rect};

pub fn main() -> anyhow::Result<()> {
    let mut ui = Sdl2UI::init(1080, 1000, "Risky Endevours")?;
    let mut ts = TileSet::df_rde()?;

    let mut col: u16 = 0;
    let mut row: u16 = 0;
    render(row, col, &mut ui, &mut ts)?;

    loop {
        match ui.wait_event() {
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

                _ => continue,
            },

            _ => continue,
        }

        render(row, col, &mut ui, &mut ts)?;
    }
}

fn render(row: u16, col: u16, ui: &mut Sdl2UI<'_>, ts: &mut TileSet<'_>) -> anyhow::Result<()> {
    ui.clear();

    // show the tile itself
    let tile = ts.ibm437_tile(row, col);
    ui.blit_tile(&tile, Rect::new(50, 50, 100, 100), ts)?;

    // show the coords
    let mut r = Rect::new(50, 200, 50, 50);
    for ch in format!("({row},{col})").chars() {
        ui.blit_tile(&ts.tile(&ch.to_string()).unwrap(), r, ts)?;
        r.x += 40;
    }

    // show the ident (if there is one)
    if let Some(ident) = ts.tile_name(tile) {
        let mut r = Rect::new(50, 250, 50, 50);
        for ch in ident.to_string().chars() {
            ui.blit_tile(&ts.tile(&ch.to_string()).unwrap(), r, ts)?;
            r.x += 40;
        }
    }

    ui.render()
}
