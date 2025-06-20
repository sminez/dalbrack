use dalbrack::{
    Grid, Pos, TITLE,
    grid::dijkstra_map,
    map::{
        Map,
        builders::{BspDungeon, BuildMap, CACave, SimpleDungeon},
    },
    player::Player,
    state::State,
    tileset::Tile,
    ui::blend,
};
use sdl2::{event::Event, keyboard::Keycode, mouse::MouseButton, rect::Rect};
use std::{
    thread::sleep,
    time::{Duration, Instant},
};

const DXY: u32 = 25;
const W: i32 = 60;
const H: i32 = 40;

macro_rules! set {
    ($builder:expr, $new:expr, $state:expr) => {{
        $builder = Box::new($new);
        let (pos, mut map) = $builder.new_map(W as usize, H as usize, &$state);
        map.explore_all();
        $state.set_map(map);
        Player::set_pos(pos, &mut $state);
    }};
}

pub fn main() -> anyhow::Result<()> {
    let mut state = State::init(DXY * W as u32, DXY * H as u32, DXY, TITLE)?;
    let mut builder = Box::new(CACave::default()) as Box<dyn BuildMap>;
    let (pos, mut map) = builder.new_map(W as usize, H as usize, &state);
    map.explore_all();
    state.set_map(map);

    let player_sprite = state.tile_with_named_color("@", "white");
    state.e_player = state.world.spawn((Player, pos, player_sprite));

    tick(&mut state)?;
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
                Keycode::Num1 => set!(builder, SimpleDungeon, state),
                Keycode::Num2 => set!(builder, BspDungeon, state),
                Keycode::Num3 => set!(builder, CACave::simple(15), state),
                Keycode::Num4 => set!(builder, CACave::rogue_basin(15), state),

                Keycode::L | Keycode::Right => Player::try_move(1, 0, &mut state),
                Keycode::H | Keycode::Left => Player::try_move(-1, 0, &mut state),
                Keycode::K | Keycode::Up => Player::try_move(0, -1, &mut state),
                Keycode::J | Keycode::Down => Player::try_move(0, 1, &mut state),
                Keycode::Y => Player::try_move(-1, -1, &mut state),
                Keycode::U => Player::try_move(1, -1, &mut state),
                Keycode::B => Player::try_move(-1, 1, &mut state),
                Keycode::N => Player::try_move(1, 1, &mut state),

                Keycode::R => {
                    let (pos, mut map) = builder.new_map(W as usize, H as usize, &state);
                    map.explore_all();
                    state.set_map(map);
                    Player::set_pos(pos, &mut state);
                }

                Keycode::RightBracket => state.ui.dxy += 5,
                Keycode::LeftBracket => state.ui.dxy -= 5,

                Keycode::Q | Keycode::Escape => return Ok(()),

                _ => need_render = false,
            },

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
                    Player::try_move_pos(new_pos, &mut state);
                    tick(&mut state)?;
                    sleep(Duration::from_millis(10));
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
            tick(&mut state)?;
        }
    }
}

fn tick(state: &mut State<'_>) -> anyhow::Result<()> {
    let dmap = update_dmap(state);
    state.ui.clear();

    let mut r = Rect::new(0, 0, state.ui.dxy, state.ui.dxy);
    let dxy = state.ui.dxy as i32;
    r.x = 0;
    r.y = 0;

    for (y, line) in dmap.cells.chunks(dmap.w).enumerate() {
        for (x, tile) in line.iter().enumerate() {
            r.x = x as i32 * dxy;
            r.y = y as i32 * dxy;

            state.ts.blit_tile(tile, r, &mut state.ui.buf)?;
        }
    }
    state.blit_tiles()?;

    state.ui.render()?;

    Ok(())
}

fn update_dmap(state: &mut State<'_>) -> Grid<Tile> {
    let pos = *state.world.query_one_mut::<&Pos>(state.e_player).unwrap();
    let map = state.world.query_one_mut::<&mut Map>(state.e_map).unwrap();
    let raw = dijkstra_map(&map.tiles, &[(pos, 0)], |p| map.tile_at(p).path_cost);

    let near = *state.palette.get("autumn_red").unwrap();
    let far = *state.palette.get("wave_blue_2").unwrap();
    let hidden = *state.palette.get("hidden").unwrap();

    let min = *raw.cells.iter().min().unwrap();
    let max = *raw.cells.iter().filter(|&&i| i != i32::MAX).max().unwrap() as f32;

    let cells: Vec<_> = raw
        .cells
        .iter()
        .map(|i| {
            if *i == i32::MAX {
                state.tile_with_color("square", hidden)
            } else {
                let perc = ((*i - min) as f32) / max;
                let color = blend(far, near, perc);
                state.tile_with_color("square", color)
            }
        })
        .collect();

    Grid {
        cells,
        w: raw.w,
        h: raw.h,
    }
}
