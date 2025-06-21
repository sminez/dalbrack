use dalbrack::{
    Pos, TITLE,
    map::{
        Map,
        builders::{BuildMap, CellularAutomata},
        fov::{FovRange, LightSource},
    },
    player::Player,
    state::State,
};
use rand::Rng;
use sdl2::{event::Event, keyboard::Keycode, mouse::MouseButton, pixels::Color};
use std::{
    thread::sleep,
    time::{Duration, Instant},
};

const DXY: u32 = 25;
const W: i32 = 70;
const H: i32 = 40;

pub fn main() -> anyhow::Result<()> {
    let mut state = State::init(DXY * W as u32, DXY * H as u32, DXY, TITLE)?;

    let mut builder = CellularAutomata::walled_cities();
    // let mut builder = BspDungeon;

    let (pos, map) = builder.new_map(W as usize, H as usize, &state);
    state.set_map(map);

    let player_sprite = state.tile_with_named_color("@", "white");
    state.e_player = state.world.spawn((
        Player,
        FovRange(30),
        LightSource {
            range: 12,
            color: Color::RGB(80, 50, 20),
        },
        pos,
        player_sprite,
    ));

    let mut rng = rand::rng();

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
                Keycode::L | Keycode::Right => Player::try_move(1, 0, &mut state),
                Keycode::H | Keycode::Left => Player::try_move(-1, 0, &mut state),
                Keycode::K | Keycode::Up => Player::try_move(0, -1, &mut state),
                Keycode::J | Keycode::Down => Player::try_move(0, 1, &mut state),
                Keycode::Y => Player::try_move(-1, -1, &mut state),
                Keycode::U => Player::try_move(1, -1, &mut state),
                Keycode::B => Player::try_move(-1, 1, &mut state),
                Keycode::N => Player::try_move(1, 1, &mut state),

                Keycode::R => {
                    let (pos, map) = builder.new_map(W as usize, H as usize, &state);
                    state.set_map(map);
                    Player::set_pos(pos, &mut state);
                    let lights: Vec<_> = state
                        .world
                        .query::<&LightSource>()
                        .without::<&Player>()
                        .iter()
                        .map(|(e, _)| e)
                        .collect();

                    for entity in lights.into_iter() {
                        state.world.despawn(entity)?;
                    }
                }

                Keycode::Space => {
                    let map = state.world.query_one_mut::<&mut Map>(state.e_map).unwrap();
                    if map.explored.len() == map.tiles.len() {
                        map.clear_explored();
                    } else {
                        map.explore_all();
                    }
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
                let color = Color::RGB(
                    rng.random_range(60..150),
                    rng.random_range(60..150),
                    rng.random_range(60..150),
                );

                state.world.spawn((
                    Pos::new(x / state.ui.dxy as i32, y / state.ui.dxy as i32),
                    state.tile_with_color("star", color),
                    LightSource {
                        range: rng.random_range(5..12),
                        color,
                    },
                ));
            }

            Event::MouseButtonDown {
                mouse_btn: MouseButton::Middle,
                x,
                y,
                ..
            } => {
                let pos = Pos::new(x / state.ui.dxy as i32, y / state.ui.dxy as i32);
                let map = state.world.query_one_mut::<&mut Map>(state.e_map).unwrap();
                map.tiles[pos] = if map.tiles[pos] == 0 { 1 } else { 0 };
            }

            Event::MouseButtonDown {
                mouse_btn: MouseButton::Right,
                x,
                y,
                ..
            } => {
                let target = Pos::new(x / state.ui.dxy as i32, y / state.ui.dxy as i32);
                let from = *state.world.query_one_mut::<&Pos>(state.e_player).unwrap();
                let map = state.world.query_one_mut::<&mut Map>(state.e_map).unwrap();
                let path = map.a_star_in_player_explored(from, target);

                for new_pos in path.into_iter() {
                    Player::try_move_pos(new_pos, &mut state);
                    state.tick()?;
                    sleep(Duration::from_millis(50));
                }
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
