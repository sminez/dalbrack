use dalbrack::{
    Pos,
    map::builders::{BspDungeon, BuildMap},
    player::{FovRange, Player},
    state::State,
    tileset::TileSet,
};
use sdl2::{event::Event, keyboard::Keycode, mouse::MouseButton};
use std::time::Instant;

const DXY: u32 = 20;
const W: i32 = 60;
const H: i32 = 40;

pub fn main() -> anyhow::Result<()> {
    let mut state = State::init(DXY * W as u32, DXY * H as u32, DXY, "Risky Endevours")?;

    let (pos, map) = BspDungeon.new_map(W as usize, H as usize, &state);
    state.set_map(map);

    let player_sprite = state.tile_with_color("@", "white");
    state.e_player = state.world.spawn((
        Player,
        FovRange {
            light_range: 5,
            explore_range: 8,
        },
        pos,
        player_sprite,
    ));

    state.tick()?;
    let mut t1 = Instant::now();

    loop {
        let mut need_render = true;
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

                Keycode::Right => Player::try_move(1, 0, &mut state),
                Keycode::Left => Player::try_move(-1, 0, &mut state),
                Keycode::Up => Player::try_move(0, -1, &mut state),
                Keycode::Down => Player::try_move(0, 1, &mut state),

                Keycode::R => {
                    let (pos, map) = BspDungeon.new_map(W as usize, H as usize, &state);
                    state.set_map(map);
                    Player::set_pos(pos, &mut state);
                }

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

        let t2 = Instant::now();
        if t2.duration_since(t1).as_secs_f64() >= 1.0 {
            t1 = t2;
            need_render = true;
        }

        if need_render {
            state.tick()?;
        }
    }
}
