use dalbrack::{
    map::{
        Map,
        builders::{BuildMap, SimpleDungeon},
    },
    state::State,
    tileset::TileSet,
};
use sdl2::{event::Event, keyboard::Keycode};
use std::time::Instant;

const DXY: u32 = 30;
const W: i32 = 60;
const H: i32 = 40;
const FRAME_LEN: f64 = 0.5;

pub fn main() -> anyhow::Result<()> {
    let mut state = State::init(DXY * W as u32, DXY * H as u32, DXY, "Risky Endevours")?;

    let mut maps = SimpleDungeon.trace_build(W as usize, H as usize, &state);
    maps.reverse();

    let mut t1 = Instant::now();
    update(&mut maps, &mut state)?;

    loop {
        let mut need_render = false;

        if let Some(event) = state.ui.poll_event() {
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
                        maps = SimpleDungeon.trace_build(W as usize, H as usize, &state);
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
        if t2.duration_since(t1).as_secs_f64() >= FRAME_LEN {
            t1 = t2;
            need_render = true;
        }

        if need_render {
            update(&mut maps, &mut state)?;
        }
    }
}

fn update(maps: &mut Vec<Map>, state: &mut State<'_>) -> anyhow::Result<()> {
    if let Some(map) = maps.pop() {
        state.set_map(map);

        state.ui.clear();
        state.blit_all()?;
        state.ui.render()?;
    }

    Ok(())
}
