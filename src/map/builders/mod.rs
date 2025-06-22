//! Map building algorithms
use crate::{Pos, map::Map, state::State};

mod bsp;
mod cellular_automata;

pub use bsp::BspDungeon;
pub use cellular_automata::{CaRule, CellularAutomata};

pub trait BuildMap: Send + Sync {
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

pub struct MapBuilder(pub Box<dyn Fn() -> Box<dyn BuildMap> + Send + Sync + 'static>);

impl MapBuilder {
    pub fn get(&self) -> Box<dyn BuildMap> {
        (self.0)()
    }
}

impl<F, B> From<F> for MapBuilder
where
    F: Fn() -> B + Send + Sync + 'static,
    B: BuildMap + 'static,
{
    fn from(builder: F) -> Self {
        Self(Box::new(move || Box::new((builder)())))
    }
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
