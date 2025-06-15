use dalbrack::{Pos, state::State, tileset::TileSet};
use sdl2::{event::Event, keyboard::Keycode};

const X: i32 = 40;
const DIM: u32 = X as u32 * 16;

pub fn main() -> anyhow::Result<()> {
    let mut state = State::init(DIM, DIM, X as u32, "Risky Endevours")?;

    for y in 0..16 {
        for x in 0..16 {
            let tile = state.ts.cp437_tile(y, x);
            state.world.spawn((Pos::new(x as i32, y as i32), tile));
        }
    }

    state.blit_all()?;
    state.ui.render()?;

    loop {
        match state.ui.wait_event() {
            Event::Quit { .. } => return Ok(()),

            Event::KeyDown {
                keycode: Some(k),
                repeat: false,
                ..
            } => match k {
                Keycode::Num1 => state.ts = TileSet::df_classic()?,
                Keycode::Num2 => state.ts = TileSet::df_buddy()?,
                Keycode::Num3 => state.ts = TileSet::df_sb()?,
                Keycode::Num4 => state.ts = TileSet::df_nordic()?,
                Keycode::Num5 => state.ts = TileSet::df_rde()?,
                Keycode::Num6 => state.ts = TileSet::df_yayo()?,
                Keycode::Num7 => state.ts = TileSet::df_kruggsmash()?,

                Keycode::Space => state.ui.toggle_debug_bg(),
                Keycode::Q | Keycode::Escape => return Ok(()),

                _ => continue,
            },

            _ => continue,
        }

        state.ui.clear();
        state.blit_all()?;
        state.ui.render()?;
    }
}
