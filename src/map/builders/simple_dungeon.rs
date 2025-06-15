use crate::{
    Pos,
    map::{Map, MapBuilder},
    state::State,
};
use rand::Rng;
use sdl2::rect::Rect;

pub struct SimpleDungeon;

impl MapBuilder for SimpleDungeon {
    fn build(&mut self, map_w: usize, map_h: usize, state: &State<'_>) -> (Pos, Map) {
        let mut map = Map::new(map_w, map_h, state);
        let mut rooms: Vec<Rect> = Vec::new();

        let max_rooms = 30;
        let min_size = 6;
        let max_size = 10;

        let mut rng = rand::rng();

        for _ in 0..max_rooms {
            let w = rng.random_range(min_size..max_size);
            let h = rng.random_range(min_size..max_size);
            let x = rng.random_range(1..map_w - w - 1) - 1;
            let y = rng.random_range(1..map_h - h - 1) - 1;
            let new_room = Rect::new(x as i32, y as i32, w as u32, h as u32);

            for other_room in rooms.iter() {
                if new_room.intersection(*other_room).is_some() {
                    continue;
                }
            }

            map.carve_room(new_room);

            if !rooms.is_empty() {
                let new = new_room.center();
                let prev = rooms[rooms.len() - 1].center();
                if rng.random_bool(0.5) {
                    map.carve_h_tunnel(prev.x, new.x, prev.y);
                    map.carve_v_tunnel(prev.y, new.y, new.x);
                } else {
                    map.carve_v_tunnel(prev.y, new.y, prev.x);
                    map.carve_h_tunnel(prev.x, new.x, new.y);
                }
            }

            rooms.push(new_room);
        }

        let p = rooms[0].center();

        (Pos::new(p.x, p.y), map)
    }
}
