use risky_endevours::{tileset::TileSet, ui::Sdl2UI};
use sdl2::{event::Event, keyboard::Keycode, rect::Rect};

const X: i32 = 40;
const DIM: u32 = X as u32 * 16;

pub fn main() -> anyhow::Result<()> {
    let mut ui = Sdl2UI::init(DIM, DIM, "Risky Endevours")?;
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
    ui.clear();
    let mut r = Rect::new(0, 0, X as u32, X as u32);

    for row in 0..16 {
        for col in 0..16 {
            let tile = ts.ibm437_tile(row, col);
            ui.blit_tile(tile, ts, r)?;
            r.x += X;
        }
        r.x = 0;
        r.y += X;
    }

    ui.render()
}
