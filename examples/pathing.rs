use dalbrack::{
    Pos, TITLE,
    grid::WeightedGrid,
    map::{
        Map,
        builders::{BspDungeon, BuildMap},
    },
    player::Player,
    state::State,
};
use sdl2::{event::Event, keyboard::Keycode, mouse::MouseButton, pixels::Color};
use std::{
    thread::sleep,
    time::{Duration, Instant},
};

const DXY: u32 = 25;
const W: i32 = 60;
const H: i32 = 40;

struct PathTile;

pub fn main() -> anyhow::Result<()> {
    let mut state = State::init(DXY * W as u32, DXY * H as u32, DXY, TITLE)?;
    let (pos, mut map) = BspDungeon.new_map(W as usize, H as usize, &state);
    map.explore_all();
    state.set_map(map);

    let player_sprite = state.tile_with_named_color("@", "white");
    state.e_player = state.world.spawn((Player, pos, player_sprite));

    state.tick()?;
    let mut t1 = Instant::now();

    let mut path_to_cursor = false;
    let mut path_target: Option<Pos> = None;

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
                    let (pos, mut map) = BspDungeon.new_map(W as usize, H as usize, &state);
                    map.explore_all();
                    state.set_map(map);
                    Player::set_pos(pos, &mut state);
                }

                Keycode::P => {
                    path_to_cursor = !path_to_cursor;
                    if !path_to_cursor {
                        clear_path(&mut state);
                    }
                }

                Keycode::RightBracket => state.ui.dxy += 5,
                Keycode::LeftBracket => state.ui.dxy -= 5,

                Keycode::Q | Keycode::Escape => return Ok(()),

                _ => need_render = false,
            },

            Event::MouseMotion { x, y, .. } if path_to_cursor => {
                let target = Pos::new(x / state.ui.dxy as i32, y / state.ui.dxy as i32);
                if path_target != Some(target) {
                    clear_path(&mut state);
                    path_target = Some(target);

                    let from = *state.world.query_one_mut::<&Pos>(state.e_player).unwrap();
                    let map = state.world.query_one_mut::<&mut Map>(state.e_map).unwrap();
                    let path = map.a_star(from, target);

                    for pos in path.into_iter() {
                        state.world.spawn((
                            PathTile,
                            pos,
                            state.tile_with_color("circle", Color::CYAN),
                        ));
                    }
                } else {
                    need_render = false;
                }
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
                let path = map.a_star(from, target);

                for new_pos in path.into_iter() {
                    let pos = state
                        .world
                        .query_one_mut::<&mut Pos>(state.e_player)
                        .unwrap();
                    *pos = new_pos;
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

fn clear_path(state: &mut State<'_>) {
    let path_tiles: Vec<_> = state
        .world
        .query::<&Pos>()
        .with::<&PathTile>()
        .iter()
        .map(|(e, _)| e)
        .collect();

    for entity in path_tiles.into_iter() {
        state.world.despawn(entity).unwrap();
    }
}
