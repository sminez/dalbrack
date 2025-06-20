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
    rule: fn(Pos, &Map) -> bool,
}

impl Default for CACave {
    fn default() -> Self {
        Self {
            iterations: 15,
            rule: rogue_basin,
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
        let mut rng = RngHandle::new();

        loop {
            let mut map = Map::new(map_w, map_h, state);

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
                        let on = (self.rule)(p, &map);
                        new[p] = if on { FLOOR } else { WALL };
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

macro_rules! rule {
    ($name:ident, $impl:expr) => {
        pub fn $name(pos: Pos, map: &Map) -> bool {
            $impl(pos, map)
        }

        impl CACave {
            pub fn $name(iterations: usize) -> Self {
                Self {
                    iterations,
                    rule: $name,
                }
            }
        }
    };
}

rule!(simple, |pos: Pos, map: &Map| {
    let n = map
        .neighbouring_tiles(pos)
        .filter(|q| map[*q] == WALL)
        .count();

    [1, 2, 3, 4].contains(&n)
});

rule!(rogue_basin, |pos: Pos, map: &Map| {
    let on = map[pos] == FLOOR;
    let n = map
        .neighbouring_tiles(pos)
        .filter(|q| map[*q] == WALL)
        .count();

    (on && n >= 5) || (!on && n >= 4)
});
