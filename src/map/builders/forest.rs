use crate::{
    Pos,
    map::{
        Map, MapTile,
        builders::{
            BuildMap, CellularAutomata, Snapshots,
            cellular_automata::{FILLED, StartingPosition},
        },
    },
    mob::{Mob, PIXIE, SNOOT},
    state::State,
    ui::palette,
};
use hecs::Entity;
use rand::{Rng, seq::IndexedRandom};
use sdl2::pixels::Color;

/// Produces a dense, maze-like forest with lots of open areas that you can see through to between
/// the trees.
pub struct Forest {
    ca: CellularAutomata,
    p: Pos,
}

impl Default for Forest {
    fn default() -> Self {
        Self::new()
    }
}

impl Forest {
    pub fn new() -> Self {
        let mut ca = CellularAutomata::walled_cities();
        ca.start_pos = StartingPosition::South;

        Self {
            ca,
            p: Pos::new(0, 0),
        }
    }
}

impl BuildMap for Forest {
    fn bg_and_tiles(&self, state: &State<'_>) -> (Color, Vec<MapTile>) {
        let bg = palette::FOREST_BG;
        let tiles = MapTile::forest_tiles(&state.ts);

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
        self.p = pos;

        // randomise trees
        for tile in map.tiles.cells.iter_mut() {
            if *tile == FILLED {
                *tile = *[0, 2, 3, 4].choose(&mut state.rng).unwrap();
            }
        }

        Some((pos, map))
    }

    fn populate(&mut self, state: &mut State<'_>) -> Vec<Entity> {
        let mut entities: Vec<_> = self
            .ca
            .regions
            .iter()
            .map(|r| {
                let p = r[state.rng.random_range(0..r.len())];
                Mob::spawn_spec(PIXIE, p.x, p.y, state)
            })
            .collect();

        let p = self
            .ca
            .regions
            .iter()
            .map(|r| r[state.rng.random_range(0..r.len())])
            .max_by(|a, b| a.fdist(self.p).total_cmp(&b.fdist(self.p)))
            .unwrap();

        entities.push(Mob::spawn_spec(SNOOT, p.x, p.y, state));

        entities
    }
}
