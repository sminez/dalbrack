//! Map building algorithms
use crate::{Pos, map::Map, state::State};

mod bsp;
mod cellular_automata;
mod simple_dungeon;

pub use bsp::BspDungeon;
pub use cellular_automata::CACave;
pub use simple_dungeon::SimpleDungeon;

pub trait BuildMap {
    fn new_map(&mut self, map_w: usize, map_h: usize, state: &State<'_>) -> (Pos, Map) {
        let mut snapshots = Snapshots {
            inner: Vec::new(),
            active: false,
        };

        self.build(map_w, map_h, state, &mut snapshots)
    }

    fn trace_build(&mut self, map_w: usize, map_h: usize, state: &State<'_>) -> Vec<Map> {
        let mut snapshots = Snapshots {
            inner: Vec::new(),
            active: true,
        };

        let (_, mut map) = self.build(map_w, map_h, state, &mut snapshots);
        map.explore_all();
        snapshots.inner.push(map);

        snapshots.inner
    }

    fn build(
        &mut self,
        map_w: usize,
        map_h: usize,
        state: &State<'_>,
        snapshots: &mut Snapshots,
    ) -> (Pos, Map);
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
