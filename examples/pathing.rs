use dalbrack::{
    Pos, TITLE,
    input::map_event_in_game_state,
    map::{
        Map,
        builders::{BspDungeon, BuildMap},
    },
    player::Player,
    state::State,
};
use sdl2::{event::Event, keyboard::Keycode, pixels::Color};
use std::time::Instant;

const DXY: u32 = 25;
const W: i32 = 60;
const H: i32 = 40;

struct PathTile;

pub fn main() -> anyhow::Result<()> {
    let mut state = State::init(DXY * W as u32, DXY * H as u32, DXY, TITLE)?;
    let (pos, mut map) = BspDungeon::default().new_map(W as usize, H as usize, &mut state);
    map.explore_all();
    state.set_map(map);

    let player_sprite = state.tile_with_named_color("@", "white");
    state.e_player = state.world.spawn((Player, pos, player_sprite));

    state.tick()?;
    let mut t1 = Instant::now();

    let mut path_to_cursor = false;
    let mut path_target: Option<Pos> = None;

    while state.running {
        let mut need_render = true;

        if let Some(event) = state.ui.wait_event_timeout(500) {
            match map_event_in_game_state(&event) {
                Some(action) => state.action_queue.push_back(action),
                None => match event {
                    Event::KeyDown {
                        keycode: Some(k),
                        repeat: false,
                        ..
                    } => match k {
                        Keycode::R => {
                            let (pos, mut map) =
                                BspDungeon::default().new_map(W as usize, H as usize, &mut state);
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

                    _ => need_render = false,
                },
            }
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

    Ok(())
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
