use dalbrack::{
    TITLE,
    map::builders::{BspDungeon, BuildMap},
    state::State,
    tileset::TileSet,
};
use sdl2::{event::Event, keyboard::Keycode};
use std::time::Instant;

const DXY: u32 = 30;
const W: i32 = 50;
const H: i32 = 30;
const FRAME_LEN: u128 = 100;

pub fn main() -> anyhow::Result<()> {
    let mut state = State::init(DXY * W as u32, DXY * H as u32, DXY, TITLE)?;
    let mut maps = BspDungeon.trace_build(W as usize, H as usize, &state);
    maps.reverse();
    if let Some(map) = maps.pop() {
        state.set_map(map);
    }

    let mut t1 = Instant::now();
    update(&mut state)?;

    loop {
        let mut need_render = false;

        if let Some(event) = state.ui.wait_event_timeout(FRAME_LEN as u32) {
            need_render = true;
            match event {
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

                    Keycode::R => {
                        maps = BspDungeon.trace_build(W as usize, H as usize, &state);
                        maps.reverse();
                    }

                    Keycode::RightBracket => state.ui.dxy += 5,
                    Keycode::LeftBracket => state.ui.dxy -= 5,

                    Keycode::Q | Keycode::Escape => return Ok(()),

                    _ => need_render = false,
                },

                _ => need_render = false,
            }
        }

        let t2 = Instant::now();
        if t2.duration_since(t1).as_millis() >= FRAME_LEN {
            t1 = t2;
            if let Some(map) = maps.pop() {
                state.set_map(map);
                need_render = true;
            }
        }

        if need_render {
            update(&mut state)?;
        }
    }
}

fn update(state: &mut State<'_>) -> anyhow::Result<()> {
    state.ui.clear();
    state.blit_all()?;
    state.ui.render()?;

    Ok(())
}
