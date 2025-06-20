//! https://en.wikipedia.org/wiki/Cellular_automaton
//! https://www.roguebasin.com/index.php?title=Cellular_Automata_Method_for_Generating_Random_Cave-Like_Levels
use crate::{
    Pos,
    map::{
        FLOOR, Map, WALL,
        builders::{BuildMap, Snapshots},
    },
    rng::RngHandle,
    state::State,
};

const P_INITIAL_FLOOR: u16 = 55;

#[derive(Debug)]
pub struct CACave {
    iterations: usize,
    rule: CARule,
}

impl Default for CACave {
    fn default() -> Self {
        Self {
            iterations: 15,
            rule: CARule::rogue_basin(),
        }
    }
}

impl BuildMap for CACave {
    fn build(
        &mut self,
        map_w: usize,
        map_h: usize,
        state: &State<'_>,
        snapshots: &mut Snapshots,
    ) -> (Pos, Map) {
        let mut map = Map::new(map_w, map_h, state);
        let mut rng = RngHandle::new();

        // randomly initialise the map with %P_INITIAL_FLOOR floor tiles
        for i in 0..map.tiles.len() {
            if rng.percentile() > P_INITIAL_FLOOR {
                map.tiles[i] = FLOOR;
            }
        }
        snapshots.push(&map);

        // run the automata
        for _ in 0..self.iterations {
            let mut new = map.tiles.clone();

            for y in 1..map_h - 1 {
                for x in 1..map_w - 1 {
                    let p = Pos::new(x as i32, y as i32);
                    let on = map[p] == FLOOR;
                    let n = map
                        .neighbouring_tiles(p)
                        .filter(|q| map[*q] == WALL)
                        .count();
                    new[p] = if self.rule.is_on(n, on) { FLOOR } else { WALL };
                }
            }

            map.tiles = new;
            snapshots.push(&map);
        }

        (Pos::new(0, 0), map)
    }
}

/// Specify cellular automata rules in terms of a bitmask defining whether or not the cell is alive
/// based on the number of neighbours it has. The bitmask is reversed before being stored so it can
/// be written with the least significant bit on the left.
macro_rules! rule {
    ($on_mask:expr, $off_mask:expr, $name:ident) => {
        impl CARule {
            pub fn $name() -> Self {
                Self {
                    on_mask: ($on_mask as u16).reverse_bits(),
                    off_mask: ($off_mask as u16).reverse_bits(),
                }
            }
        }

        impl CACave {
            pub fn $name(iterations: usize) -> Self {
                Self {
                    iterations,
                    rule: CARule::$name(),
                }
            }
        }
    };
}

#[derive(Debug)]
pub struct CARule {
    on_mask: u16,
    off_mask: u16,
}

impl CARule {
    #[inline]
    pub fn is_on(&self, n_neighbours: usize, on_now: bool) -> bool {
        let mask = if on_now { self.on_mask } else { self.off_mask };

        (mask >> n_neighbours) & 1 == 1
    }
}

rule!(0b0111100000000000, 0b0111100000000000, simple);
rule!(0b0000011110000000, 0b0000111110000000, rogue_basin);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_alive_works() {
        let rule = CARule::simple();

        for n in [1, 2, 3, 4] {
            assert!(rule.is_on(n, true), "{n} should be alive");
        }

        for n in [0, 5, 6, 7, 8] {
            assert!(!rule.is_on(n, true), "{n} should not be alive");
        }
    }
}
