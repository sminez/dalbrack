//! About as simple as you can get for ensuring a connected map
use crate::{
    Pos,
    map::{
        Map,
        builders::{BuildMap, Snapshots},
    },
    state::State,
};
use rand::Rng;
use sdl2::rect::Rect;

pub struct SimpleDungeon;

impl BuildMap for SimpleDungeon {
    fn build(
        &mut self,
        map_w: usize,
        map_h: usize,
        state: &State<'_>,
        snapshots: &mut Snapshots,
    ) -> (Pos, Map) {
        let mut map = Map::new(map_w, map_h, state);
        let mut rooms: Vec<Rect> = Vec::new();
        let mut rng = rand::rng();

        let max_rooms = 30;
        let min_size = 6;
        let max_size = 10;

        for _ in 0..max_rooms {
            let w = rng.random_range(min_size..max_size);
            let h = rng.random_range(min_size..max_size);
            let x = rng.random_range(1..map_w - w - 1) - 1;
            let y = rng.random_range(1..map_h - h - 1) - 1;
            let r_new = Rect::new(x as i32, y as i32, w as u32, h as u32);

            // Ensure that we also don't have rooms that are adjacent without a dividing wall
            let test = Rect::new(
                r_new.x - 1,
                r_new.y - 1,
                r_new.w as u32 + 2,
                r_new.h as u32 + 2,
            );

            if rooms.iter().any(|r| r.intersection(test).is_some()) {
                continue;
            }

            map.carve_rect(r_new);
            snapshots.push(&map);

            if !rooms.is_empty() {
                let new = r_new.center();
                let prev = rooms[rooms.len() - 1].center();
                if rng.random_bool(0.5) {
                    map.carve_h_tunnel(prev.x, new.x, prev.y);
                    map.carve_v_tunnel(prev.y, new.y, new.x);
                } else {
                    map.carve_v_tunnel(prev.y, new.y, prev.x);
                    map.carve_h_tunnel(prev.x, new.x, new.y);
                }
                snapshots.push(&map);
            }

            rooms.push(r_new);
        }

        let p = rooms[0].center();

        (Pos::new(p.x, p.y), map)
    }
}
