use dalbrack::{
    Grid, Pos, TITLE,
    grid::dijkstra_map,
    input::map_event_in_game_state,
    map::{
        Map,
        builders::{
            BuildConfig, BuildMap, CaRule, CellularAutomata, voronoi_regions_from_seeds,
            voronoi_seeds,
        },
    },
    player::Player,
    rng::RngHandle,
    state::State,
    tileset::Tile,
    ui::blend,
};
use notify_debouncer_full::{DebounceEventResult, new_debouncer, notify::RecursiveMode};
use rand::Rng;
use sdl2::{event::Event, keyboard::Keycode, pixels::Color, rect::Rect};
use std::{
    fs,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

static FILE_CHANGED: AtomicBool = AtomicBool::new(false);
const DXY: u32 = 25;
const W: i32 = 80;
const H: i32 = 50;
const CFG: BuildConfig = BuildConfig { populated: false };
const N_GROUPS: usize = 16;

macro_rules! set {
    ($builder:expr, $new:expr, $state:expr) => {{
        $builder = Box::new($new);
        let (pos, mut map) = $builder.new_map(W as usize, H as usize, CFG, &mut $state);
        map.explore_all();
        $state.set_map(map);
        Player::warp(pos, &$state);
    }};
}

pub fn main() -> anyhow::Result<()> {
    let mut state = State::init(DXY * W as u32, DXY * H as u32, DXY, TITLE)?;
    let mut builder = Box::new(parse_ca_rule()?) as Box<dyn BuildMap>;
    let (pos, mut map) = builder.new_map(W as usize, H as usize, CFG, &mut state);
    map.explore_all();
    state.set_map(map);

    state.e_player = state
        .world
        .spawn(Player::new_bundle_without_fov(pos, &state).build());

    // data for voronoi groups
    let mut rng = RngHandle::new();
    let mut seeds = voronoi_seeds(N_GROUPS, W as usize, H as usize, &mut rng);
    let colors: Vec<Color> = (0..N_GROUPS)
        .map(|_| {
            Color::RGB(
                rng.random_range(60..150),
                rng.random_range(60..150),
                rng.random_range(60..150),
            )
        })
        .collect();
    let mut use_voronoi = true;

    // set up file watcher for the rules file
    let mut watcher = new_debouncer(
        Duration::from_millis(500),
        None,
        |res: DebounceEventResult| match res {
            Err(errs) => errs.iter().for_each(|error| println!("{error:?}")),
            Ok(evts) => {
                if evts
                    .iter()
                    .filter(|e| e.kind.is_create())
                    .any(|e| e.paths.iter().any(|p| p.ends_with("ca.rules")))
                {
                    FILE_CHANGED.store(true, Ordering::Relaxed);
                }
            }
        },
    )?;
    watcher.watch(Path::new("data"), RecursiveMode::NonRecursive)?;

    state.tick_with(|state| update_ui(&seeds, &colors, use_voronoi, state))?;

    while state.running {
        if FILE_CHANGED.swap(false, Ordering::Relaxed) {
            match parse_ca_rule() {
                Ok(ca) => {
                    set!(builder, ca, state);
                    seeds = voronoi_seeds(N_GROUPS, W as usize, H as usize, &mut rng);
                    update_ui(&seeds, &colors, use_voronoi, &mut state)?;
                    continue;
                }
                Err(e) => println!("ERROR {e}"),
            }
        }

        if let Some(event) = state.ui.poll_event() {
            match map_event_in_game_state(&event, &state) {
                Some(action) => state.action_queue.push_back(action),
                None => match event {
                    Event::KeyDown {
                        keycode: Some(Keycode::R),
                        repeat: false,
                        ..
                    } => match parse_ca_rule() {
                        Ok(ca) => {
                            seeds = voronoi_seeds(N_GROUPS, W as usize, H as usize, &mut rng);
                            set!(builder, ca, state);
                        }
                        Err(e) => println!("ERROR {e}"),
                    },

                    Event::KeyDown {
                        keycode: Some(Keycode::V),
                        repeat: false,
                        ..
                    } => use_voronoi = true,

                    Event::KeyDown {
                        keycode: Some(Keycode::D),
                        repeat: false,
                        ..
                    } => use_voronoi = false,

                    Event::MouseMotion { .. } => continue,
                    _ => (),
                },
            }
        }

        state.tick_with(|state| update_ui(&seeds, &colors, use_voronoi, state))?;
    }

    Ok(())
}

fn update_ui(
    seeds: &[Pos],
    colors: &[Color],
    use_voronoi: bool,
    state: &mut State<'_>,
) -> anyhow::Result<()> {
    state.ui.clear();

    let mut r = Rect::new(0, 0, state.ui.dxy, state.ui.dxy);
    let dxy = state.ui.dxy as i32;
    r.x = 0;
    r.y = 0;

    if use_voronoi {
        for (pos, tile) in color_regions(seeds, colors, state).into_iter() {
            r.x = pos.x * dxy;
            r.y = pos.y * dxy;

            state.ts.blit_tile(&tile, r, &mut state.ui.buf)?;
        }
    } else {
        let dmap = update_dmap(state);
        for (y, line) in dmap.cells.chunks(dmap.w).enumerate() {
            for (x, tile) in line.iter().enumerate() {
                r.x = x as i32 * dxy;
                r.y = y as i32 * dxy;

                state.ts.blit_tile(tile, r, &mut state.ui.buf)?;
            }
        }
    }

    state.blit_map()?;
    state.blit_tiles()?;
    state.ui.render()?;

    Ok(())
}

// Parse the rules file which is expected to contain lines of the form:
//   pfloor iterations | bXXsXXX # some comment
//
// Only the first non comment line is used when parsing so the rest of the file can contain
// arbitrary notes
fn parse_ca_rule() -> anyhow::Result<CellularAutomata> {
    use anyhow::anyhow;

    let s = fs::read_to_string("data/ca.rules")?;
    let mut raw = s
        .lines()
        .find(|line| !line.is_empty() && !line.starts_with('#'))
        .ok_or(anyhow!("empty file"))?;

    raw = raw.split_once('#').map(|(s, _)| s).unwrap_or(raw);

    let (meta, bs) = raw.split_once(" | ").ok_or(anyhow!("invalid rule meta"))?;
    let (pfloor, it) = meta.split_once(' ').ok_or(anyhow!("invalid rule meta"))?;
    let (p_initial_floor, iterations): (u16, usize) = (pfloor.parse()?, it.trim().parse()?);

    let (b, s) = bs.split_once("s").ok_or(anyhow!("invalid rule"))?;
    let born = as_u8s(b.strip_prefix('b').ok_or(anyhow!("invalid rule"))?)?;
    let survive = as_u8s(s.trim())?;

    Ok(CellularAutomata {
        p_initial_floor,
        iterations,
        rule: CaRule::LifeLike { born, survive },
        regions: Default::default(),
    })
}

fn as_u8s(s: &str) -> anyhow::Result<Vec<u8>> {
    s.chars()
        .map(|ch| {
            ch.to_digit(10)
                .ok_or(anyhow::anyhow!("invalid digit"))
                .map(|d| d as u8)
        })
        .collect()
}

fn update_dmap(state: &mut State<'_>) -> Grid<Tile> {
    let pos = *state.world.query_one_mut::<&Pos>(state.e_player).unwrap();
    let map = state.world.query_one_mut::<&mut Map>(state.e_map).unwrap();
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

fn color_regions(seeds: &[Pos], colors: &[Color], state: &State<'_>) -> Vec<(Pos, Tile)> {
    let map = state.world.get::<&Map>(state.e_map).unwrap();
    let points = map.cells.iter().enumerate().flat_map(|(i, idx)| {
        if *idx > 0 {
            let x = i % map.w;
            let y = i / map.w;
            Some(Pos::new(x as i32, y as i32))
        } else {
            None
        }
    });

    voronoi_regions_from_seeds(seeds, points)
        .into_iter()
        .enumerate()
        .flat_map(|(i, group)| {
            group
                .into_iter()
                .map(move |p| (p, state.tile_with_color("square", colors[i])))
        })
        .collect()
}
