//! Map building algorithms
use crate::{Pos, map::Map, state::State};

mod bsp;
mod cellular_automata;
mod simple_dungeon;

pub use bsp::BspDungeon;
pub use cellular_automata::{CaRule, CellularAutomata};
pub use simple_dungeon::SimpleDungeon;

pub trait BuildMap {
    fn new_map(&mut self, map_w: usize, map_h: usize, state: &State<'_>) -> (Pos, Map) {
        let mut snapshots = Snapshots {
            inner: Vec::new(),
            active: false,
        };

        loop {
            let mut map = Map::new(map_w, map_h, state);
            self.init_map(&mut map);
            snapshots.push(&map);

            if let Some(output) = self.build(map, state, &mut snapshots) {
                return output;
            }

            snapshots.inner.clear();
        }
    }

    fn trace_build(&mut self, map_w: usize, map_h: usize, state: &State<'_>) -> Vec<Map> {
        let mut snapshots = Snapshots {
            inner: Vec::new(),
            active: true,
        };

        loop {
            let mut map = Map::new(map_w, map_h, state);
            self.init_map(&mut map);
            snapshots.push(&map);

            if let Some((_, mut map)) = self.build(map, state, &mut snapshots) {
                map.explore_all();
                snapshots.inner.push(map);
                return snapshots.inner;
            }

            snapshots.inner.clear();
        }
    }

    #[allow(unused_variables)]
    fn init_map(&mut self, map: &mut Map) {}

    fn build(
        &mut self,
        map: Map,
        state: &State<'_>,
        snapshots: &mut Snapshots,
    ) -> Option<(Pos, Map)>;
}

#[derive(Debug)]
pub struct Snapshots {
    inner: Vec<Map>,
    active: bool,
}

impl Snapshots {
    fn push(&mut self, map: &Map) {
        if self.active {
            let mut map = map.clone();
            map.explore_all();
            self.inner.push(map.clone());
        }
    }
}
