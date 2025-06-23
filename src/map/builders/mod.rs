//! Map building algorithms
use crate::{Pos, map::Map, state::State};

mod bsp;
mod cellular_automata;
mod voronoi;

pub use bsp::BspDungeon;
pub use cellular_automata::{CaRule, CellularAutomata};
pub use voronoi::{voronoi_regions, voronoi_regions_from_seeds, voronoi_seeds};

pub trait BuildMap: Send + Sync {
    #[allow(unused_variables)]
    fn init_map(&mut self, map: &mut Map) {}

    fn build(
        &mut self,
        map: Map,
        state: &mut State<'_>,
        snapshots: &mut Snapshots,
    ) -> Option<(Pos, Map)>;

    fn populate(&mut self, state: &mut State<'_>);

    fn new_map(
        &mut self,
        map_w: usize,
        map_h: usize,
        config: BuildConfig,
        state: &mut State<'_>,
    ) -> (Pos, Map) {
        let mut snapshots = Snapshots {
            inner: Vec::new(),
            active: false,
        };

        loop {
            let mut map = Map::new(map_w, map_h, state);
            self.init_map(&mut map);
            snapshots.push(&map);

            if let Some(output) = self.build(map, state, &mut snapshots) {
                if config.populated {
                    self.populate(state);
                }
                return output;
            }

            snapshots.inner.clear();
        }
    }

    fn trace_build(&mut self, map_w: usize, map_h: usize, state: &mut State<'_>) -> Vec<Map> {
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
}

#[derive(Debug, Copy, Clone)]
pub struct BuildConfig {
    pub populated: bool,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self { populated: true }
    }
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
