//! https://www.roguebasin.com/index.php?title=Cellular_Automata_Method_for_Generating_Random_Cave-Like_Levels
//! https://en.wikipedia.org/wiki/Cellular_automaton
//! https://en.wikipedia.org/wiki/Elementary_cellular_automaton
//! https://en.wikipedia.org/wiki/Life-like_cellular_automaton
//! https://en.wikipedia.org/wiki/Life-like_cellular_automaton#A_selection_of_Life-like_rules
//! https://conwaylife.com/wiki/List_of_Life-like_rules
//! https://mcell.ca/pages/rules.html
//! https://catagolue.hatsya.com/rules/lifelike
//! https://conwaylife.com/wiki/Isotropic_non-totalistic_rule
use crate::{
    Grid, Pos,
    map::{
        Map,
        builders::{BuildMap, Snapshots, voronoi_regions},
        map_tile::MapTile,
    },
    rng::RngHandle,
    state::State,
};
use hecs::Entity;
use sdl2::pixels::Color;

const MIN_OPEN_PERC: f32 = 0.45;
const N_SEEDS: usize = 16;

pub const FILLED: usize = 0;
pub const OPEN: usize = 1;

#[derive(Debug)]
pub enum StartingPosition {
    North,
    South,
    East,
    West,
    Center,
}

impl StartingPosition {
    pub fn locate(&self, map: &Map) -> Option<Pos> {
        let (w, h) = (map.w as i32, map.h as i32);

        let (x, y, dx, dy) = match self {
            Self::North => (w / 2, 0, 1, 0),
            Self::South => (w / 2, h - 1, 1, 0),
            Self::East => (w - 1, h / 2, 0, -1),
            Self::West => (0, h / 2, 0, -1),
            Self::Center => (w / 2, h / 2, -1, 0),
        };

        for i in [1, -1] {
            let mut pos = Pos::new(x, y);
            let delta = Pos::new(dx * i, dy * i);

            while map.contains_pos(pos) {
                if map[pos] == OPEN {
                    return Some(pos);
                }

                pos += delta;
            }
        }

        None
    }
}

/// Helper struct that can construct a map from a given cellular automata rule.
/// When used directly via [BuildMap] this will return maps where filled cells are walls and open
/// cells are floor but no other features.
pub struct CellularAutomata {
    pub start_pos: StartingPosition,
    pub p_initial_open: u16,
    pub iterations: usize,
    pub rule: CaRule,
    pub regions: Vec<Vec<Pos>>,
}

impl Default for CellularAutomata {
    fn default() -> Self {
        Self::walled_cities()
    }
}

impl CellularAutomata {
    fn has_sufficient_open(&self, map: &Map) -> bool {
        let n_open = map.tiles.cells.iter().filter(|&&t| t == OPEN).count();
        let p_open = n_open as f32 / map.tiles.cells.len() as f32;

        p_open >= MIN_OPEN_PERC
    }

    fn assign_regions(&mut self, map: &Map, rng: &mut RngHandle) {
        let points = map.cells.iter().enumerate().flat_map(|(i, idx)| {
            if *idx > 0 {
                let x = i % map.w;
                let y = i / map.w;
                Some(Pos::new(x as i32, y as i32))
            } else {
                None
            }
        });

        self.regions = voronoi_regions(N_SEEDS, map.w, map.h, points, rng);
    }
}

impl BuildMap for CellularAutomata {
    fn bg_and_tiles(&self, state: &State<'_>) -> (Color, Vec<MapTile>) {
        let bg = *state.palette.get("black").unwrap();
        let tiles = MapTile::dungeon_tiles(&state.ts, &state.palette);

        (bg, tiles)
    }

    fn init_map(&mut self, map: &mut Map) {
        let mut rng = RngHandle::new();

        for i in 0..map.tiles.len() {
            if rng.percentile() > self.p_initial_open {
                map.tiles[i] = OPEN;
            }
        }
    }

    fn build(
        &mut self,
        mut map: Map,
        state: &mut State<'_>,
        snapshots: &mut Snapshots,
    ) -> Option<(Pos, Map)> {
        for i in 0..self.iterations {
            map.tiles = self.rule.run(i, &map);
            snapshots.push(&map);
        }

        let pos = self.start_pos.locate(&map)?;
        self.fill_unreachable_from(&[(pos, 0)], FILLED, &mut map);

        if !self.has_sufficient_open(&map) {
            return None;
        }

        self.assign_regions(&map, &mut state.rng);

        Some((pos, map))
    }

    fn populate(&mut self, _state: &mut State<'_>) -> Vec<Entity> {
        Vec::new()
    }
}

/// for cell p, how many FILLED cells can be reached within a distance of n.
fn n_filled(p: Pos, n: i32, current: &Grid<usize>) -> u8 {
    let mut count = 0;

    for dy in -n..=n {
        for dx in -n..=n {
            let q = p + Pos::new(dx, dy);
            if ((dy == 0) && (dx == 0)) || !current.contains_pos(q) {
                continue;
            }
            if current[q] == FILLED {
                count += 1;
            }
        }
    }

    count
}

// Rules

pub enum CaRule {
    Fn(fn(Pos, usize, &Grid<usize>) -> bool),
    LifeLike { born: Vec<u8>, survive: Vec<u8> },
}

impl CaRule {
    pub fn run(&self, i: usize, current: &Grid<usize>) -> Grid<usize> {
        let mut new = current.clone();

        for y in 1..current.h - 1 {
            for x in 1..current.w - 1 {
                let p = Pos::new(x as i32, y as i32);
                new[p] = self.state_for(p, i, current);
            }
        }

        new
    }

    pub fn state_for(&self, p: Pos, i: usize, current: &Grid<usize>) -> usize {
        let filled = match self {
            Self::Fn(f) => (f)(p, i, current),
            Self::LifeLike { born, survive } => life_like_rule(p, current, born, survive),
        };

        if filled { FILLED } else { OPEN }
    }
}

/// See https://en.wikipedia.org/wiki/Life-like_cellular_automaton
fn life_like_rule(p: Pos, current: &Grid<usize>, born: &[u8], survive: &[u8]) -> bool {
    let n = n_filled(p, 1, current);
    let alive = current[p] == FILLED;

    (alive && survive.contains(&n)) || (!alive && born.contains(&n))
}

macro_rules! rule {
    ($name:ident, $p_open:expr, $iterations:expr, $impl:expr) => {
        pub fn $name(pos: Pos, i: usize, current: &Grid<usize>) -> bool {
            $impl(pos, i, current)
        }

        impl CaRule {
            pub fn $name() -> Self {
                Self::Fn($name)
            }
        }

        impl CellularAutomata {
            pub fn $name() -> Self {
                Self {
                    start_pos: StartingPosition::Center,
                    p_initial_open: $p_open,
                    iterations: $iterations,
                    rule: CaRule::$name(),
                    regions: Default::default(),
                }
            }
        }
    };

    (@life $name:ident, $p_open:expr, $iterations:expr, [$($born:expr),*], [$($survive:expr),*]) => {
        pub fn $name(pos: Pos, _: usize, current: &Grid<usize>) -> bool {
            life_like_rule(pos, current, &[$($born),*], &[$($survive),*])
        }

        impl CaRule {
            pub fn $name() -> Self {
                Self::Fn($name)
            }
        }

        impl CellularAutomata {
            pub fn $name() -> Self {
                Self {
                    start_pos: StartingPosition::Center,
                    p_initial_open: $p_open,
                    iterations: $iterations,
                    rule: CaRule::Fn($name),
                    regions: Default::default(),
                }
            }
        }
    };
}

rule!(simple, 55, 15, |p: Pos, _i: usize, g: &Grid<usize>| {
    let n = n_filled(p, 1, g);

    [0, 5, 6, 7, 8].contains(&n)
});

rule!(rogue_basin, 60, 7, |p: Pos, i: usize, g: &Grid<usize>| {
    let n1 = n_filled(p, 1, g);
    let n2 = n_filled(p, 2, g);

    if i < 4 { n1 >= 5 || n2 <= 2 } else { n1 >= 5 }
});

// 45 7  | b5678s45678     # vote - https://conwaylife.com/wiki/OCA:Vote
rule!(@life vote, 45, 7, [5,6,7,8], [4,5,6,7,8]);

rule!(@life conway, 55, 7, [3], [2, 3]);
rule!(@life day_night, 70, 7, [3,6,7,8], [3,4,6,7,8]);
rule!(@life anneal, 55, 12, [4,6,7,8], [3,5,6,7,8]);
rule!(@life morley, 80, 7, [3,6,8], [2,4,5]);
rule!(@life invertamaze, 60, 7, [0,2,8], [0,1,2,4]);
rule!(@life h_trees, 10, 2, [1], [0,1,2,3,4,5,6,7,8]);
rule!(@life walled_cities, 60, 5, [4,5,6,7,8], [2,3,4,5]);

rule!(@life maze,      60, 10, [3], [1,2,3,4,5]);
rule!(@life mazectric, 60, 10, [3], [1,2,3,4]);
rule!(@life corrosion, 60, 10, [3], [1,2,4]);

rule!(@life diamoeba,  45, 6, [3,5,6,7,8], [5,6,7,8]);
rule!(@life ice_balls, 45, 6, [2,5,6,7,8], [5,6,7,8]);

rule!(@life coagulations, 60, 6, [2,3,5,6,7,8], [3,7,8]);
rule!(@life serviettes, 50, 3, [], [2,3,4]);
rule!(@life gnarl, 1, 30, [1], [1]);

rule!(@life stains, 21, 9, [3,6,7,8], [2,3,5,6,7,8]);
