//! Map building algorithms
use crate::{Pos, map::Map, state::State};
use rand::{Rng, rngs::ThreadRng};
use sdl2::rect::Rect;

mod bsp;
mod simple_dungeon;

pub use bsp::BspDungeon;
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

fn random_point(r: Rect, offset: i32, rng: &mut ThreadRng) -> (i32, i32) {
    let rx = (r.x + offset)..(r.x + r.w - offset);
    let ry = (r.y + offset)..(r.y + r.h - offset);

    let x = if rx.is_empty() {
        r.x
    } else {
        rng.random_range(rx)
    };
    let y = if ry.is_empty() {
        r.y
    } else {
        rng.random_range(ry)
    };

    (x, y)
}
