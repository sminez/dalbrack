//! See:
//!   https://www.roguebasin.com/index.php/FOV_using_recursive_shadowcasting
//!   https://www.roguebasin.com/index.php/Line_of_Sight_-_Tobias_Downer
//!   https://www.roguebasin.com/index.php/Computing_LOS_for_Large_Areas
use crate::{Pos, map::Map};
use sdl2::pixels::Color;
use std::collections::{HashMap, HashSet};

/// Multipliers for each octant to map to the correct coordinate system
const MULTIPLIERS: [[i32; 8]; 4] = [
    [1, 0, 0, -1, -1, 0, 0, 1],
    [0, 1, -1, 0, 0, -1, 1, 0],
    [0, 1, 1, 0, 0, -1, -1, 0],
    [1, 0, 0, 1, -1, 0, 0, -1],
];
/// Scaling factor for inverse-square falloff
const DIST_SCALE: f32 = 0.15;
/// Exponent to correct with when r^2 drops below 1.0
const EXP_FALLOFF: f32 = 0.11;
/// % of original color to use when blending light levels
const BLEND_PERC: f32 = 0.5;

#[derive(Debug, Clone, Copy)]
pub struct FovRange(pub u32);

pub struct Fov {
    pub points: HashSet<Pos>,
}

impl Fov {
    pub fn new(map: &Map, from: Pos, FovRange(range): FovRange) -> Self {
        let mut points = HashSet::with_capacity(4 * (range * range) as usize);
        points.insert(from);

        for octant in 0..8 {
            let builder = FovBuilder {
                points: &mut points,
            };
            Caster::new(from, range, octant, builder, map).cast(1, 1.0, 0.0, false);
        }

        Fov { points }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LightSource {
    pub range: u32,
    pub color: Color,
}

pub struct LightMap {
    pub points: HashMap<Pos, Color>,
}

impl LightMap {
    pub fn from_sources<'a>(
        map: &Map,
        fov: &Fov,
        sources: impl Iterator<Item = (&'a Pos, &'a LightSource)>,
    ) -> Self {
        let mut points = HashMap::with_capacity(fov.points.len());

        for (from, source) in sources {
            let lm = Self::new(map, *from, fov, *source);
            for (p, color) in lm.points.into_iter() {
                points
                    .entry(p)
                    .and_modify(|current| {
                        *current = blend(*current, color, 0.5);
                    })
                    .or_insert(color);
            }
        }

        LightMap { points }
    }

    pub fn new(map: &Map, from: Pos, fov: &Fov, LightSource { range, color }: LightSource) -> Self {
        let mut points = HashMap::with_capacity(4 * (range * range) as usize);
        points.insert(from, color);

        for octant in 0..8 {
            let builder = LightBuilder {
                points: &mut points,
                color,
                fov,
            };
            Caster::new(from, range, octant, builder, map).cast(1, 1.0, 0.0, false);
        }

        LightMap { points }
    }

    pub fn apply_light_level(&self, p: Pos, color: Color) -> Option<Color> {
        let light_color = self.points.get(&p)?;

        Some(blend(color, *light_color, BLEND_PERC))
    }
}

trait Builder {
    fn in_fov(&self, pos: Pos) -> bool;
    fn push(&mut self, pos: Pos, from: Pos);
}

struct FovBuilder<'a> {
    points: &'a mut HashSet<Pos>,
}

impl<'a> Builder for FovBuilder<'a> {
    fn in_fov(&self, _pos: Pos) -> bool {
        true
    }

    fn push(&mut self, pos: Pos, _from: Pos) {
        self.points.insert(pos);
    }
}

struct LightBuilder<'a> {
    points: &'a mut HashMap<Pos, Color>,
    color: Color,
    fov: &'a Fov,
}

impl<'a> Builder for LightBuilder<'a> {
    fn in_fov(&self, pos: Pos) -> bool {
        self.fov.points.contains(&pos)
    }

    fn push(&mut self, pos: Pos, from: Pos) {
        let d = from.fdist(pos);
        let mut falloff = (d * DIST_SCALE).powi(2);
        if falloff < 1.0 {
            falloff = falloff.powf(EXP_FALLOFF);
        }

        let color = Color::RGB(
            (self.color.r as f32 / falloff) as u8,
            (self.color.g as f32 / falloff) as u8,
            (self.color.b as f32 / falloff) as u8,
        );

        self.points.insert(pos, color);
    }
}

struct Caster<'a, B>
where
    B: Builder,
{
    from: Pos,
    range: u32,
    xx: i32,
    xy: i32,
    yx: i32,
    yy: i32,
    builder: B,
    map: &'a Map,
}

impl<'a, B> Caster<'a, B>
where
    B: Builder,
{
    fn new(from: Pos, range: u32, octant: usize, builder: B, map: &'a Map) -> Self {
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
            builder,
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
                let pos = Pos::new(x, y);

                // If we're out of bounds then skip this cell
                if x < 0
                    || y < 0
                    || x >= self.map.w as i32
                    || y >= self.map.h as i32
                    || !self.builder.in_fov(pos)
                {
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

                let idx = self.map.pos_idx(pos);

                if dx * dx + dy * dy <= r2 {
                    self.builder.push(pos, self.from);
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

fn blend(color1: Color, color2: Color, perc: f32) -> Color {
    let (c1, m1, y1, k1) = to_cmyk(color1);
    let (c2, m2, y2, k2) = to_cmyk(color2);

    from_cmyk(
        c1 * perc + c2 * (1.0 - perc),
        m1 * perc + m2 * (1.0 - perc),
        y1 * perc + y2 * (1.0 - perc),
        k1 * perc + k2 * (1.0 - perc),
    )
}

fn to_cmyk(color: Color) -> (f32, f32, f32, f32) {
    let mut c = 1.0 - (color.r as f32 / 255.0);
    let mut m = 1.0 - (color.g as f32 / 255.0);
    let mut y = 1.0 - (color.b as f32 / 255.0);

    let mut k = if c < m { c } else { m };
    k = if k < y { k } else { y };

    c = (c - k) / (1.0 - k);
    m = (m - k) / (1.0 - k);
    y = (y - k) / (1.0 - k);

    (c, m, y, k)
}

fn from_cmyk(c: f32, m: f32, y: f32, k: f32) -> Color {
    let mut r = c * (1.0 - k) + k;
    let mut g = m * (1.0 - k) + k;
    let mut b = y * (1.0 - k) + k;

    r = (1.0 - r) * 255.0 + 0.5;
    g = (1.0 - g) * 255.0 + 0.5;
    b = (1.0 - b) * 255.0 + 0.5;

    Color::RGB(r as u8, g as u8, b as u8)
}
