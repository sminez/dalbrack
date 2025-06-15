//! Compute the FOV from a given point in terms of tile indices
//!
//! See https://www.roguebasin.com/index.php/FOV_using_recursive_shadowcasting
use crate::{Pos, map::Map};
use std::collections::HashSet;

const MULTIPLIERS: [[i32; 8]; 4] = [
    [1, 0, 0, -1, -1, 0, 0, 1],
    [0, 1, -1, 0, 0, -1, 1, 0],
    [0, 1, 1, 0, 0, -1, -1, 0],
    [1, 0, 0, 1, -1, 0, 0, -1],
];

pub(super) fn determine_fov(map: &Map, from: Pos, range: u32) -> HashSet<Pos> {
    let mut fov = HashSet::with_capacity(4 * (range * range) as usize);
    fov.insert(from);

    for octant in 0..8 {
        cast_light(from, 1, 1.0, 0.0, range, octant, &mut fov, map);
    }

    fov
}

#[allow(clippy::too_many_arguments)]
fn cast_light(
    from: Pos,
    row: i32,
    mut start: f32,
    end: f32,
    range: u32,
    octant: usize,
    fov: &mut HashSet<Pos>,
    map: &Map,
) {
    let xx = MULTIPLIERS[0][octant];
    let xy = MULTIPLIERS[1][octant];
    let yx = MULTIPLIERS[2][octant];
    let yy = MULTIPLIERS[3][octant];
    let r2 = (range * range) as i32;
    let mut new_start = start;
    let mut prev_blocked = false;

    for i in row..=range as i32 {
        let (mut dx, dy) = (-i - 1, -i);

        while dx < 0 {
            dx += 1;

            // l_slope / r_slope are the left / right extremities of the current cell
            let l_slope = (dx as f32 - 0.5) / (dy as f32 + 0.5);
            let r_slope = (dx as f32 + 0.5) / (dy as f32 - 0.5);

            if start < r_slope {
                continue;
            } else if end > l_slope {
                break;
            }

            // map dx, dy coords to map coords
            let x = from.x + dx * xx + dy * xy;
            let y = from.y + dx * yx + dy * yy;
            if x < 0 || y < 0 || x >= map.w as i32 || y >= map.h as i32 {
                continue;
            }

            let pos = Pos::new(x, y);
            let idx = map.pos_idx(pos);

            if dx * dx + dy * dy < r2 {
                fov.insert(pos);
            }

            let cur_blocked = map.tile_defs[map.tiles[idx]].block_sight;
            if prev_blocked {
                if cur_blocked {
                    new_start = r_slope;
                } else {
                    prev_blocked = false;
                    start = new_start;
                }
            } else if cur_blocked && i < range as i32 {
                if start >= end {
                    cast_light(from, i + 1, start, l_slope, range, octant, fov, map);
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
