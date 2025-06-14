use risky_endevours::{data_files::parse_ibm437_prefab, tileset::TileSet, ui::Sdl2UI};
use sdl2::{event::Event, keyboard::Keycode};
use std::env::args;

const X: i32 = 1;
const Y: i32 = 1;

pub fn main() -> anyhow::Result<()> {
    let path = match args().nth(1) {
        Some(path) => path,
        None => "assets/prefabs/room.prefab".to_string(),
    };

    let mut ui = Sdl2UI::init(1280, 1000, "Risky Endevours")?;
    let mut dxy: u32 = 50;

    render(&path, dxy, &mut ui)?;

    loop {
        match ui.wait_event() {
            Event::Quit { .. } => return Ok(()),

            Event::KeyDown {
                keycode: Some(k),
                repeat: false,
                ..
            } => match k {
                Keycode::Num1 => ui.ts = TileSet::df_classic()?,
                Keycode::Num2 => ui.ts = TileSet::df_buddy()?,
                Keycode::Num3 => ui.ts = TileSet::df_sb()?,
                Keycode::Num4 => ui.ts = TileSet::df_nordic()?,
                Keycode::Num5 => ui.ts = TileSet::df_rde()?,
                Keycode::Num6 => ui.ts = TileSet::df_yayo()?,
                Keycode::Num7 => ui.ts = TileSet::df_kruggsmash()?,

                Keycode::RightBracket => dxy += 5,
                Keycode::LeftBracket => dxy -= 5,

                Keycode::Space => ui.toggle_debug_bg(),
                Keycode::Q | Keycode::Escape => return Ok(()),

                _ => continue,
            },

            _ => continue,
        }

        render(&path, dxy, &mut ui)?;
    }
}

fn render(path: &str, dxy: u32, ui: &mut Sdl2UI<'_>) -> anyhow::Result<()> {
    let grid = parse_ibm437_prefab(path, &ui.ts, &ui.palette)?;
    ui.clear();
    ui.blit_grid(&grid, X * dxy as i32, Y * dxy as i32, dxy)?;
    ui.render()
}
