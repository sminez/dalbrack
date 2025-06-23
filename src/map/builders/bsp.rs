use crate::{
    Pos,
    map::{
        Map,
        builders::{BuildMap, Snapshots},
    },
    mob::Mob,
    rng::RngHandle,
    state::State,
};
use rand::{Rng, rngs::ThreadRng};
use sdl2::rect::Rect;
use std::cmp::{max, min};

/// min split position as a %
const SPLIT_FROM: f32 = 0.35;
/// max split position as a %
const SPLIT_TO: f32 = 0.7;
/// max number of splits
const MAX_DEPTH: usize = 4;
/// minimum room ratio
const MIN_RAT: f32 = 0.45;

#[derive(Default, Debug)]
pub struct BspDungeon {
    rooms: Vec<Rect>,
}

impl BuildMap for BspDungeon {
    fn build(
        &mut self,
        mut map: Map,
        _: &State<'_>,
        snapshots: &mut Snapshots,
    ) -> Option<(Pos, Map)> {
        let mut rng = RngHandle::new();
        let starting_room = self.split_and_connect(
            Rect::new(0, 0, map.w as u32, map.h as u32),
            0,
            &mut rng,
            &mut map,
            snapshots,
        );

        let p = starting_room.center();

        Some((Pos::new(p.x, p.y), map))
    }

    fn populate(&mut self, state: &mut State<'_>) {
        for r in self.rooms.iter() {
            let c = r.center();
            Mob::spawn("f", "faded_green", c.x, c.y, state);
        }
    }
}

impl BspDungeon {
    fn split_and_connect(
        &mut self,
        r: Rect,
        depth: usize,
        rng: &mut RngHandle,
        map: &mut Map,
        snapshots: &mut Snapshots,
    ) -> Rect {
        if depth == MAX_DEPTH {
            let r = position_and_carve(r, rng, map, snapshots);
            self.rooms.push(r);
            return r;
        }

        let (r1, r2) = split(r, rng);
        let r2 = self.split_and_connect(r2, depth + 1, rng, map, snapshots);
        let r1 = self.split_and_connect(r1, depth + 1, rng, map, snapshots);

        connect(r1, r2, rng, map, snapshots);

        if rng.random_bool(0.5) { r1 } else { r2 }
    }
}

fn aspect_ratio(w: i32, h: i32) -> f32 {
    (min(w, h) as f32) / (max(w, h) as f32)
}

fn split(r: Rect, rng: &mut ThreadRng) -> (Rect, Rect) {
    loop {
        if rng.random_bool(0.5) {
            let split_point = rng.random_range(SPLIT_FROM..SPLIT_TO);
            let w = (r.w as f32 * split_point) as u32;
            if aspect_ratio(w as i32, r.h) < MIN_RAT {
                continue;
            }

            return (
                Rect::new(r.x, r.y, w, r.h as u32),
                Rect::new(r.x + w as i32, r.y, r.w as u32 - w, r.h as u32),
            );
        } else {
            let split_point = rng.random_range(SPLIT_FROM..SPLIT_TO);
            let h = (r.h as f32 * split_point) as u32;
            if aspect_ratio(r.w, h as i32) < MIN_RAT {
                continue;
            }

            return (
                Rect::new(r.x, r.y, r.w as u32, h),
                Rect::new(r.x, r.y + h as i32, r.w as u32, r.h as u32 - h),
            );
        }
    }
}

fn position_and_carve(
    mut r: Rect,
    rng: &mut ThreadRng,
    map: &mut Map,
    snapshots: &mut Snapshots,
) -> Rect {
    let p = r.top_left();

    r.x += rng.random_range(1..max(2, r.w / 3));
    r.y += rng.random_range(1..max(2, r.h / 3));
    r.w -= r.x - p.x;
    r.w -= rng.random_range(1..max(2, r.w / 3));
    r.h -= r.y - p.y;
    r.h -= rng.random_range(1..max(2, r.h / 3));

    map.carve_rect(r);
    snapshots.push(map);

    r
}

fn connect(r1: Rect, r2: Rect, rng: &mut RngHandle, map: &mut Map, snapshots: &mut Snapshots) {
    let Pos { x: x1, y: y1 } = rng.random_point(r1, 1);
    let Pos { x: x2, y: y2 } = rng.random_point(r2, 1);

    if rng.random_bool(0.5) {
        map.carve_h_tunnel(x1, x2, y1);
        map.carve_v_tunnel(y1, y2, x2);
    } else {
        map.carve_v_tunnel(y1, y2, x1);
        map.carve_h_tunnel(x1, x2, y2);
    }
    snapshots.push(map);
}
