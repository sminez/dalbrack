use dalbrack::{
    Pos,
    mob::{Mob, RandomMoveAI},
    state::State,
    tileset::{Tile, TileSet},
};
use sdl2::{event::Event, keyboard::Keycode, mouse::MouseButton};
use std::time::Instant;

const DXY: u32 = 30;
const W: i32 = 20;
const H: i32 = 15;

struct Player;
struct Ping;

pub fn main() -> anyhow::Result<()> {
    let mut state = State::init(DXY * W as u32, DXY * H as u32, DXY, "Risky Endevours")?;

    let dorf1 = Mob::new(":)", "bright_blue", 3, 5, &state);
    let dorf2 = Mob::new(":)", "bright_yellow", 15, 12, &state);
    let player_sprite = state.tile_with_color(":D", "bright_purple");
    let mut ping_sprite = state.tile_with_color("ring-inv", "bright_aqua");
    ping_sprite.color.a = 0;

    let e_player = state.world.spawn((Player, Pos::new(9, 9), player_sprite));
    let e_ping = state.world.spawn((Ping, ping_sprite));
    state.world.spawn(dorf1);
    state.world.spawn(dorf2);

    state.ui.clear();
    state.blit_all()?;
    state.ui.render()?;

    let mut t1 = Instant::now();
    let mut need_render = true;
    let mut x = 9;
    let mut y = 9;

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

                    Keycode::Right => x += 1,
                    Keycode::Left => x -= 1,
                    Keycode::Up => y -= 1,
                    Keycode::Down => y += 1,

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
                    state.world.insert_one(e_ping, pos)?;

                    let (_, tile) = state.world.query_one_mut::<(&Ping, &mut Tile)>(e_ping)?;
                    tile.color.a = 255;
                }

                _ => need_render = false,
            }
        }

        let (_, pos) = state.world.query_one_mut::<(&Player, &mut Pos)>(e_player)?;
        pos.x = x;
        pos.y = y;

        let t2 = Instant::now();
        if t2.duration_since(t1).as_secs_f64() >= 0.2 {
            for (_e, (ai, pos)) in state.world.query_mut::<(&RandomMoveAI, &mut Pos)>() {
                ai.random_move(pos, W - 1, H - 1);
            }

            let (_, tile) = state.world.query_one_mut::<(&Ping, &mut Tile)>(e_ping)?;
            if tile.color.a > 0 {
                tile.color.a = tile.color.a.saturating_sub(60);
                if tile.color.a == 0 {
                    state.world.remove_one::<Pos>(e_ping)?;
                }
            }

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
