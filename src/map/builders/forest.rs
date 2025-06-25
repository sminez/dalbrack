use crate::{
    Pos,
    map::{
        Map, MapTile,
        builders::{BuildMap, CellularAutomata, Snapshots, cellular_automata::FILLED},
    },
    mob::{Mob, PIXIE},
    state::State,
};
use rand::{Rng, seq::IndexedRandom};
use sdl2::pixels::Color;

/// Produces a dense, maze-like forest with lots of open areas that you can see through to between
/// the trees.
pub struct Forest {
    ca: CellularAutomata,
}

impl Default for Forest {
    fn default() -> Self {
        Self::new()
    }
}

impl Forest {
    pub fn new() -> Self {
        Self {
            ca: CellularAutomata::walled_cities(),
        }
    }
}

impl BuildMap for Forest {
    fn bg_and_tiles(&self, state: &State<'_>) -> (Color, Vec<MapTile>) {
        let bg = *state.palette.get("forestBG").unwrap();
        let tiles = MapTile::forest_tiles(&state.ts, &state.palette);

        (bg, tiles)
    }

    fn init_map(&mut self, map: &mut Map) {
        self.ca.init_map(map);
    }

    fn build(
        &mut self,
        map: Map,
        state: &mut State<'_>,
        snapshots: &mut Snapshots,
    ) -> Option<(Pos, Map)> {
        let (pos, mut map) = self.ca.build(map, state, snapshots)?;

        // randomise trees
        for tile in map.tiles.cells.iter_mut() {
            if *tile == FILLED {
                *tile = *[0, 2, 3, 4].choose(&mut state.rng).unwrap();
            }
        }

        Some((pos, map))
    }

    fn populate(&mut self, state: &mut State<'_>) {
        for r in self.ca.regions.iter() {
            let p = r[state.rng.random_range(0..r.len())];
            Mob::spawn_spec(PIXIE, p.x, p.y, state);
        }
    }
}
