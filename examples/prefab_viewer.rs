use risky_endevours::{data_files::parse_ibm437_prefab, tileset::TileSet, ui::Sdl2UI};
use sdl2::{event::Event, keyboard::Keycode};

const X: i32 = 1;
const Y: i32 = 1;
const DXY: u32 = 50;

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
    let grid = parse_ibm437_prefab("assets/prefabs/room.prefab", ts)?;
    ui.clear();
    ui.blit_grid(&grid, X * DXY as i32, Y * DXY as i32, DXY, ts)?;
    ui.render()
}
