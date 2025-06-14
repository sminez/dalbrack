use risky_endevours::{mob::Mob, tileset::TileSet, ui::Sdl2UI};
use sdl2::{event::Event, keyboard::Keycode, mouse::MouseButton};
use std::time::Instant;

const DXY: u32 = 50;
const W: i32 = 20;
const H: i32 = 15;

pub fn main() -> anyhow::Result<()> {
    let mut ui = Sdl2UI::init(DXY * W as u32, DXY * H as u32, DXY, "Risky Endevours")?;
    ui.ts = TileSet::df_kruggsmash()?;

    let mut dorf1 = Mob::new(":)", "bright_blue", 3, 5, &ui);
    let mut dorf2 = Mob::new(":)", "bright_yellow", 15, 12, &ui);
    let mut player = Mob::new(":D", "bright_purple", 9, 9, &ui);

    let mut ping = Mob::new("ring-inv", "bright_aqua", 0, 0, &ui);
    ping.tile.color.a = 0;

    ui.clear();
    dorf1.blit(&mut ui)?;
    dorf2.blit(&mut ui)?;
    player.blit(&mut ui)?;
    ui.render()?;

    let mut t1 = Instant::now();
    let mut need_render = true;

    loop {
        if let Some(event) = ui.poll_event() {
            need_render = true;
            match event {
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

                    Keycode::Right => player.pos.x += 1,
                    Keycode::Left => player.pos.x -= 1,
                    Keycode::Up => player.pos.y -= 1,
                    Keycode::Down => player.pos.y += 1,

                    Keycode::RightBracket => ui.dxy += 5,
                    Keycode::LeftBracket => ui.dxy -= 5,

                    Keycode::Q | Keycode::Escape => return Ok(()),

                    _ => need_render = false,
                },

                Event::MouseButtonDown {
                    mouse_btn: MouseButton::Left,
                    x,
                    y,
                    ..
                } => {
                    ping.pos.x = x / ui.dxy as i32;
                    ping.pos.y = y / ui.dxy as i32;
                    ping.tile.color.a = 255;
                }

                _ => need_render = false,
            }
        }

        let t2 = Instant::now();
        if t2.duration_since(t1).as_secs_f64() >= 0.2 {
            dorf1.random_move(W - 1, H - 1);
            dorf2.random_move(W - 1, H - 1);
            t1 = t2;
            if ping.tile.color.a > 0 {
                ping.tile.color.a = ping.tile.color.a.saturating_sub(60);
            }
            need_render = true;
        }

        if need_render {
            ui.clear();
            dorf1.blit(&mut ui)?;
            dorf2.blit(&mut ui)?;
            player.blit(&mut ui)?;

            if ping.tile.color.a > 0 {
                ping.blit(&mut ui)?;
            }

            ui.render()?;
            need_render = false;
        }
    }
}
