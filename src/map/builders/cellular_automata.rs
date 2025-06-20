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
    Pos,
    map::{
        FLOOR, Map, WALL,
        builders::{BuildMap, Snapshots},
    },
    rng::RngHandle,
    state::State,
};

#[derive(Debug)]
pub struct CellularAutomata {
    p_initial_floor: u16,
    iterations: usize,
    rule: fn(Pos, usize, &Map) -> bool,
}

impl Default for CellularAutomata {
    fn default() -> Self {
        Self::rogue_basin()
    }
}

impl BuildMap for CellularAutomata {
    fn build(
        &mut self,
        map_w: usize,
        map_h: usize,
        state: &State<'_>,
        snapshots: &mut Snapshots,
    ) -> (Pos, Map) {
        let mut rng = RngHandle::new();

        loop {
            let mut map = Map::new(map_w, map_h, state);

            // randomly initialise the map with floor tiles
            for i in 0..map.tiles.len() {
                if rng.percentile() > self.p_initial_floor {
                    map.tiles[i] = FLOOR;
                }
            }
            snapshots.push(&map);

            // run the automata
            for i in 0..self.iterations {
                let mut new = map.tiles.clone();

                for y in 1..map_h - 1 {
                    for x in 1..map_w - 1 {
                        let p = Pos::new(x as i32, y as i32);
                        let wall = (self.rule)(p, i, &map);
                        new[p] = if wall { WALL } else { FLOOR };
                    }
                }

                map.tiles = new;
                snapshots.push(&map);
            }

            let mut pos = Pos::new(map_w as i32 / 2, map_h as i32 / 2);
            while pos.x >= 0 {
                if map[pos] == FLOOR {
                    return (pos, map);
                }
                pos.x -= 1;
            }
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

        impl CellularAutomata {
            pub fn $name() -> Self {
                Self {
                    p_initial_floor: $p_floor,
                    iterations: $iterations,
                    rule: $name,
                }
            }
        }
    };

    (@life $name:ident, $p_floor:expr, $iterations:expr, [$($born:expr),*], [$($survive:expr),*]) => {
        pub fn $name(pos: Pos, _: usize, map: &Map) -> bool {
            life_like_rule(pos, map, &[$($born),*], &[$($survive),*])
        }

        impl CellularAutomata {
            pub fn $name() -> Self {
                Self {
                    p_initial_floor: $p_floor,
                    iterations: $iterations,
                    rule: $name,
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
