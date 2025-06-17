//! Compute the FOV from a given point in terms of tile indices
//!
//! See:
//!   https://www.roguebasin.com/index.php/FOV_using_recursive_shadowcasting
//!   https://www.roguebasin.com/index.php/Line_of_Sight_-_Tobias_Downer
//!   https://www.roguebasin.com/index.php/Computing_LOS_for_Large_Areas
use crate::{Pos, map::Map};
use sdl2::pixels::Color;
use std::collections::HashSet;

const MULTIPLIERS: [[i32; 8]; 4] = [
    [1, 0, 0, -1, -1, 0, 0, 1],
    [0, 1, -1, 0, 0, -1, 1, 0],
    [0, 1, 1, 0, 0, -1, -1, 0],
    [1, 0, 0, 1, -1, 0, 0, -1],
];

pub struct Fov {
    pub points: HashSet<Pos>,
    pub center: Pos,
    pub light_range: u32,
    pub full_range: u32,
}

impl Fov {
    pub fn apply_light_level(&self, p: Pos, color: &mut Color, black: Color) {
        if !self.points.contains(&p) {
            *color = black;
            return;
        }

        // FIXME: hacky inverse square law
        // - This can end up going below the BG color so it needs a bit of tuning and possibly
        //   clamping to behave correctly
        // let step = 0.12;
        let step = 0.2;
        let d = self.center.fdist(p);
        let mut falloff = (d * step).powi(2);
        if falloff < 1.0 {
            // This _kind_ of works but is definitely a hack
            falloff = falloff.powf(0.11);
        }

        color.r = (color.r as f32 / falloff) as u8;
        color.g = (color.g as f32 / falloff) as u8;
        color.b = (color.b as f32 / falloff) as u8;
    }
}

pub(super) fn determine_fov(map: &Map, from: Pos, light_range: u32, range: u32) -> Fov {
    let mut points = HashSet::with_capacity(4 * (range * range) as usize);
    points.insert(from);

    for octant in 0..8 {
        Caster::new(from, range, octant, &mut points, map).cast(1, 1.0, 0.0, false);
    }

    Fov {
        points,
        center: from,
        light_range,
        full_range: range,
    }
}

struct Caster<'a> {
    from: Pos,
    range: u32,
    xx: i32,
    xy: i32,
    yx: i32,
    yy: i32,
    fov: &'a mut HashSet<Pos>,
    map: &'a Map,
}

impl<'a> Caster<'a> {
    fn new(from: Pos, range: u32, octant: usize, fov: &'a mut HashSet<Pos>, map: &'a Map) -> Self {
        let xx = MULTIPLIERS[0][octant];
        let xy = MULTIPLIERS[1][octant];
        let yx = MULTIPLIERS[2][octant];
        let yy = MULTIPLIERS[3][octant];

        Self {
            from,
            range,
            xx,
            xy,
            yx,
            yy,
            fov,
            map,
        }
    }

    fn cast(&mut self, row: i32, mut start: f32, end: f32, mut prev_blocked: bool) {
        let r2 = (self.range * self.range) as i32;
        let mut new_start = -1.0;

        for i in row..=self.range as i32 {
            let dx = i;
            for dy in (0..=i).rev() {
                // map dx, dy coords to map coords
                let x = self.from.x + dx * self.xx + dy * self.xy;
                let y = self.from.y + dx * self.yx + dy * self.yy;

                // If we're out of bounds then skip this cell
                if x < 0 || y < 0 || x >= self.map.w as i32 || y >= self.map.h as i32 {
                    continue;
                }

                // slopes from our origin to the top-left & bottom-right corners of this cell
                let l_slope = (dy as f32 + 0.5) / (dx as f32 - 0.5);
                let r_slope = (dy as f32 - 0.5) / (dx as f32 + 0.5);

                // > / < for a more permissive viewing angle
                // >= / <= for a more restricted viewing angle
                if r_slope > start {
                    continue; // before our sector: skip
                } else if l_slope < end {
                    break; // past our sector: we're done
                }

                let pos = Pos::new(x, y);
                let idx = self.map.pos_idx(pos);

                if dx * dx + dy * dy <= r2 {
                    self.fov.insert(pos);
                }

                let cur_blocked = self.map.tile_defs[self.map.tiles[idx]].block_sight;
                if prev_blocked {
                    if cur_blocked {
                        // still scanning a run of blocking cells
                        new_start = r_slope;
                    } else {
                        // found the end of a run of blocking cells so set the left edge of our
                        // sector to the right corner of the last blocking cell
                        prev_blocked = false;
                        start = new_start;
                    }
                } else if cur_blocked {
                    if l_slope <= start {
                        self.cast(i + 1, start, l_slope, cur_blocked);
                    }
                    prev_blocked = true;
                    new_start = r_slope;
                }
            }

            // row is scanned: check next row until the last cell is blocked
            if prev_blocked {
                break;
            }
        }
    }
}
