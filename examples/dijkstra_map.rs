use dalbrack::{
    Grid, Pos, TITLE,
    grid::dijkstra_map,
    input::map_event_in_game_state,
    map::builders::{BspDungeon, BuildConfig, BuildMap, CellularAutomata},
    player::Player,
    state::State,
    tileset::Tile,
    ui::blend,
};
use sdl2::{event::Event, keyboard::Keycode, rect::Rect};

const DXY: u32 = 25;
const W: i32 = 60;
const H: i32 = 40;
const CFG: BuildConfig = BuildConfig { populated: false };

macro_rules! set {
    ($builder:expr, $new:expr, $state:expr) => {{
        $builder = Box::new($new);
        let (pos, mut map) = $builder.new_map(W as usize, H as usize, CFG, &mut $state);
        map.explore_all();
        $state.set_map(map);
        Player::warp(pos, &mut $state);
    }};
}

pub fn main() -> anyhow::Result<()> {
    let mut state = State::init(DXY * W as u32, DXY * H as u32, DXY, TITLE)?;
    let mut builder = Box::new(BspDungeon::default()) as Box<dyn BuildMap>;
    let (pos, mut map) = builder.new_map(W as usize, H as usize, CFG, &mut state);
    map.explore_all();
    state.set_map(map);

    state.e_player = state
        .world
        .spawn(Player::new_bundle_without_fov(pos, &state).build());

    state.tick_with(update_ui)?;

    while state.running {
        if let Some(event) = state.ui.poll_event() {
            match map_event_in_game_state(&event, &state) {
                Some(action) => state.action_queue.push_back(action),
                None => match event {
                    Event::KeyDown {
                        keycode: Some(k),
                        repeat: false,
                        ..
                    } => match k {
                        Keycode::Num1 => set!(builder, BspDungeon::default(), state),
                        Keycode::Num2 => set!(builder, CellularAutomata::simple(), state),
                        Keycode::Num3 => set!(builder, CellularAutomata::rogue_basin(), state),
                        Keycode::Num4 => set!(builder, CellularAutomata::diamoeba(), state),
                        Keycode::Num5 => set!(builder, CellularAutomata::invertamaze(), state),
                        Keycode::Num6 => set!(builder, CellularAutomata::mazectric(), state),

                        Keycode::R => {
                            let (pos, mut map) =
                                builder.new_map(W as usize, H as usize, CFG, &mut state);
                            map.explore_all();
                            state.set_map(map);
                            Player::warp(pos, &state);
                        }
                        _ => (),
                    },

                    Event::MouseMotion { .. } => continue,
                    _ => (),
                },
            }
        }

        state.tick_with(update_ui)?;
    }

    Ok(())
}

fn update_ui(state: &mut State<'_>) -> anyhow::Result<()> {
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
    state.blit_map()?;
    state.blit_tiles()?;
    state.ui.render()?;

    Ok(())
}

fn update_dmap(state: &State<'_>) -> Grid<Tile> {
    let map = state.mapset.current();
    let pos = *state.world.get::<&Pos>(state.e_player).unwrap();
    let raw = dijkstra_map(&map.tiles, &[(pos, 0)], |p| map.tile_at(p).path_cost);

    let near = *state.palette.get("autumnRed").unwrap();
    let far = *state.palette.get("waveBlue2").unwrap();
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
