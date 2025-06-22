use dalbrack::{
    Grid, Pos, TITLE,
    grid::dijkstra_map,
    input::map_event_in_game_state,
    map::{
        Map,
        builders::{BuildMap, CaRule, CellularAutomata},
    },
    player::Player,
    state::State,
    tileset::Tile,
    ui::blend,
};
use notify_debouncer_full::{DebounceEventResult, new_debouncer, notify::RecursiveMode};
use sdl2::{event::Event, keyboard::Keycode, rect::Rect};
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

macro_rules! set {
    ($builder:expr, $new:expr, $state:expr) => {{
        $builder = Box::new($new);
        let (pos, mut map) = $builder.new_map(W as usize, H as usize, &mut $state);
        map.explore_all();
        $state.set_map(map);
        Player::set_pos(pos, &mut $state);
    }};
}

pub fn main() -> anyhow::Result<()> {
    let mut state = State::init(DXY * W as u32, DXY * H as u32, DXY, TITLE)?;
    let mut builder = Box::new(parse_ca_rule()?) as Box<dyn BuildMap>;
    let (pos, mut map) = builder.new_map(W as usize, H as usize, &mut state);
    map.explore_all();
    state.set_map(map);

    let player_sprite = state.tile_with_named_color("@", "white");
    state.e_player = state.world.spawn((Player, pos, player_sprite));

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

    state.tick_with(update_ui)?;

    while state.running {
        if FILE_CHANGED.swap(false, Ordering::Relaxed) {
            match parse_ca_rule() {
                Ok(ca) => {
                    set!(builder, ca, state);
                    update_ui(&mut state)?;
                    continue;
                }
                Err(e) => println!("ERROR {e}"),
            }
        }

        if let Some(event) = state.ui.wait_event_timeout(500) {
            match map_event_in_game_state(&event) {
                Some(action) => state.action_queue.push_back(action),
                None => {
                    if let Event::KeyDown {
                        keycode: Some(Keycode::R),
                        repeat: false,
                        ..
                    } = event
                    {
                        match parse_ca_rule() {
                            Ok(ca) => set!(builder, ca, state),
                            Err(e) => println!("ERROR {e}"),
                        }
                    } else {
                        continue;
                    }
                }
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
