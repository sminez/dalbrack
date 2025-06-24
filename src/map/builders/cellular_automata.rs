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
    grid::dijkstra_map,
    map::{
        FLOOR, Map, WALL,
        builders::{BuildMap, Snapshots, voronoi_regions},
    },
    mob::{Mob, PIXIE},
    rng::RngHandle,
    state::State,
};
use rand::Rng;

const MIN_FLOOR_PERC: f32 = 0.45;
const N_SEEDS: usize = 16;

pub struct CellularAutomata {
    pub p_initial_floor: u16,
    pub iterations: usize,
    pub rule: CaRule,
    pub regions: Vec<Vec<Pos>>,
}

impl Default for CellularAutomata {
    fn default() -> Self {
        Self::rogue_basin()
    }
}

impl BuildMap for CellularAutomata {
    fn init_map(&mut self, map: &mut Map) {
        let mut rng = RngHandle::new();

        for i in 0..map.tiles.len() {
            if rng.percentile() > self.p_initial_floor {
                map.tiles[i] = FLOOR;
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

        let mut pos = Pos::new(map.w as i32 / 2, map.h as i32 / 2);
        while pos.x >= 0 {
            if map[pos] != FLOOR {
                pos.x -= 1;
                continue;
            }

            // Fill in unreachable regions
            let dmap = dijkstra_map(&map.tiles, &[(pos, 0)], |p| map.tile_at(p).path_cost);
            for (i, cost) in dmap.cells.into_iter().enumerate() {
                if cost == i32::MAX {
                    map[i] = WALL;
                }
            }

            let n_floor = map.tiles.cells.iter().filter(|&&t| t == FLOOR).count();
            let p_floor = n_floor as f32 / map.tiles.cells.len() as f32;

            if p_floor < MIN_FLOOR_PERC {
                return None;
            }

            let points = map.cells.iter().enumerate().flat_map(|(i, idx)| {
                if *idx > 0 {
                    let x = i % map.w;
                    let y = i / map.w;
                    Some(Pos::new(x as i32, y as i32))
                } else {
                    None
                }
            });

            self.regions = voronoi_regions(N_SEEDS, map.w, map.h, points, &mut state.rng);

            return Some((pos, map));
        }

        None
    }

    fn populate(&mut self, state: &mut State<'_>) {
        for r in self.regions.iter() {
            let p = r[state.rng.random_range(0..r.len())];
            Mob::spawn_spec(PIXIE, p.x, p.y, state);
        }
    }
}

/// for cell p, how many WALL cells can be reached within a distance of n.
fn n_walls(p: Pos, n: i32, map: &Map) -> u8 {
    let mut count = 0;

    for dy in -n..=n {
        for dx in -n..=n {
            let q = p + Pos::new(dx, dy);
            if ((dy == 0) && (dx == 0)) || !map.contains_pos(q) {
                continue;
            }
            if map.tiles[q] == WALL {
                count += 1;
            }
        }
    }

    count
}

// Rules

pub enum CaRule {
    Fn(fn(Pos, usize, &Map) -> bool),
    LifeLike { born: Vec<u8>, survive: Vec<u8> },
}

impl CaRule {
    pub fn run(&self, i: usize, map: &Map) -> Grid<usize> {
        let mut new = map.tiles.clone();

        for y in 1..map.h - 1 {
            for x in 1..map.w - 1 {
                let p = Pos::new(x as i32, y as i32);
                let wall = self.is_wall(p, i, map);
                new[p] = if wall { WALL } else { FLOOR };
            }
        }

        new
    }

    pub fn is_wall(&self, p: Pos, i: usize, map: &Map) -> bool {
        match self {
            Self::Fn(f) => (f)(p, i, map),
            Self::LifeLike { born, survive } => life_like_rule(p, map, born, survive),
        }
    }
}

/// See https://en.wikipedia.org/wiki/Life-like_cellular_automaton
fn life_like_rule(p: Pos, map: &Map, born: &[u8], survive: &[u8]) -> bool {
    let n = n_walls(p, 1, map);
    let alive = map[p] == WALL;

    (alive && survive.contains(&n)) || (!alive && born.contains(&n))
}

macro_rules! rule {
    ($name:ident, $p_floor:expr, $iterations:expr, $impl:expr) => {
        pub fn $name(pos: Pos, i: usize, map: &Map) -> bool {
            $impl(pos, i, map)
        }

        impl CaRule {
            pub fn $name() -> Self {
                Self::Fn($name)
            }
        }

        impl CellularAutomata {
            pub fn $name() -> Self {
                Self {
                    p_initial_floor: $p_floor,
                    iterations: $iterations,
                    rule: CaRule::$name(),
                    regions: Default::default(),
                }
            }
        }
    };

    (@life $name:ident, $p_floor:expr, $iterations:expr, [$($born:expr),*], [$($survive:expr),*]) => {
        pub fn $name(pos: Pos, _: usize, map: &Map) -> bool {
            life_like_rule(pos, map, &[$($born),*], &[$($survive),*])
        }

        impl CaRule {
            pub fn $name() -> Self {
                Self::Fn($name)
            }
        }

        impl CellularAutomata {
            pub fn $name() -> Self {
                Self {
                    p_initial_floor: $p_floor,
                    iterations: $iterations,
                    rule: CaRule::Fn($name),
                    regions: Default::default(),
                }
            }
        }
    };
}

rule!(simple, 55, 15, |p: Pos, _i: usize, map: &Map| {
    let n = n_walls(p, 1, map);

    [0, 5, 6, 7, 8].contains(&n)
});

rule!(rogue_basin, 60, 7, |p: Pos, i: usize, map: &Map| {
    let n1 = n_walls(p, 1, map);
    let n2 = n_walls(p, 2, map);

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
