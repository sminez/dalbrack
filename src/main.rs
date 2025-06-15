use dalbrack::{
    Pos,
    map::{MapBuilder, builders::SimpleDungeon},
    state::State,
    tileset::TileSet,
};
use sdl2::{event::Event, keyboard::Keycode, mouse::MouseButton};
use std::time::Instant;

const DXY: u32 = 16;
const W: i32 = 80;
const H: i32 = 50;

pub fn main() -> anyhow::Result<()> {
    let mut state = State::init(DXY * W as u32, DXY * H as u32, DXY, "Risky Endevours")?;

    let (pos, map) = SimpleDungeon.build(W as usize, H as usize, &state);
    state.world.spawn((map,));

    state.ui.clear();
    state.blit_all()?;
    state.ui.render()?;

    let mut t1 = Instant::now();
    let mut need_render = true;

    loop {
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

                    Keycode::RightBracket => state.ui.dxy += 5,
                    Keycode::LeftBracket => state.ui.dxy -= 5,

                    Keycode::Q | Keycode::Escape => return Ok(()),

                    _ => need_render = false,
                },

                Event::MouseButtonDown {
                    mouse_btn: MouseButton::Left,
                    x,
                    y,
                    ..
                } => {
                    let pos = Pos::new(x / state.ui.dxy as i32, y / state.ui.dxy as i32);
                    println!("CLICK {pos:?}");
                }

                _ => need_render = false,
            }
        }

        let t2 = Instant::now();
        if t2.duration_since(t1).as_secs_f64() >= 1.0 {
            t1 = t2;
            need_render = true;
        }

        if need_render {
            state.ui.clear();
            state.blit_all()?;
            state.ui.render()?;
            need_render = false;
        }
    }
}
