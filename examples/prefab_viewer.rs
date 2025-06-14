use risky_endevours::{data_files::Prefab, tileset::TileSet, ui::Sdl2UI};
use sdl2::{event::Event, keyboard::Keycode, pixels::Color, rect::Rect};

const X: i32 = 50;

pub fn main() -> anyhow::Result<()> {
    let mut ui = Sdl2UI::init(1080, 1000, "Risky Endevours")?;
    let mut ts = TileSet::df_classic()?;

    render(&mut ui, &mut ts)?;

    loop {
        match ui.wait_event() {
            Event::Quit { .. } => return Ok(()),

            Event::KeyDown {
                keycode: Some(k),
                repeat: false,
                ..
            } => match k {
                Keycode::Num1 => ts = TileSet::df_classic()?,
                Keycode::Num2 => ts = TileSet::df_buddy()?,
                Keycode::Num3 => ts = TileSet::df_sb()?,
                Keycode::Num4 => ts = TileSet::df_nordic()?,
                Keycode::Num5 => ts = TileSet::df_rde()?,
                Keycode::Num6 => ts = TileSet::df_yayo()?,
                Keycode::Num7 => ts = TileSet::df_kruggsmash()?,

                Keycode::Space => ui.toggle_debug_bg(),
                Keycode::Q | Keycode::Escape => return Ok(()),

                _ => continue,
            },

            _ => continue,
        }

        render(&mut ui, &mut ts)?;
    }
}

fn render(ui: &mut Sdl2UI<'_>, ts: &mut TileSet<'_>) -> anyhow::Result<()> {
    let prefab = Prefab::parse_ibm437("assets/prefabs/room.prefab", ts)?;

    ui.clear();
    let mut r = Rect::new(0, 0, X as u32, X as u32);

    for line in prefab.cells.chunks(prefab.w) {
        for idx in line {
            let pos = prefab.tiles[*idx];
            ui.blit_tile(pos, Color::WHITE, ts, r)?;
            r.x += X;
        }
        r.x = 0;
        r.y += X;
    }

    ui.render()
}
