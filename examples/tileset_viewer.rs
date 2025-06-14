use risky_endevours::{tileset::TileSet, ui::Sdl2UI};
use sdl2::{event::Event, keyboard::Keycode};

pub fn main() -> anyhow::Result<()> {
    let mut ui = Sdl2UI::init(1080, 1000, 50, "Risky Endevours")?;
    ui.ts = TileSet::df_rde()?;

    let mut col: u16 = 0;
    let mut row: u16 = 0;
    render(row, col, &mut ui)?;

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

        render(row, col, &mut ui)?;
    }
}

fn render(row: u16, col: u16, ui: &mut Sdl2UI<'_>) -> anyhow::Result<()> {
    ui.clear();

    // show the tile itself
    let tile = ui.ts.ibm437_tile(row, col);
    ui.blit_tile(&tile, 1, 1)?;

    // show the coords
    let mut x = 1;
    for ch in format!("({row},{col})").chars() {
        ui.blit_tile(&ui.ts.tile(&ch.to_string()).unwrap(), x, 3)?;
        x += 1;
    }

    // show the ident (if there is one)
    if let Some(ident) = ui.ts.tile_name(tile) {
        let mut x = 1;
        for ch in ident.to_string().chars() {
            ui.blit_tile(&ui.ts.tile(&ch.to_string()).unwrap(), x, 4)?;
            x += 1;
        }
    }

    ui.render()
}
