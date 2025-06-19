//! See:
//!   https://www.roguebasin.com/index.php/FOV_using_recursive_shadowcasting
//!   https://www.roguebasin.com/index.php/Line_of_Sight_-_Tobias_Downer
//!   https://www.roguebasin.com/index.php/Computing_LOS_for_Large_Areas
//!   https://www.roguebasin.com/index.php?title=Discussion:Field_of_Vision
//!   https://www.roguebasin.com/index.php/Restrictive_Precise_Angle_Shadowcasting
use crate::{Pos, map::Map};
use sdl2::pixels::Color;
use std::collections::{HashMap, HashSet};

/// Scaling factor for inverse-square falloff
const DIST_SCALE: f32 = 0.15;
/// Exponent to correct with when r^2 drops below 1.0
const EXP_FALLOFF: f32 = 0.11;
/// % of original color to use when blending light levels
const BLEND_PERC: f32 = 0.6;

#[derive(Debug, Clone, Copy)]
pub struct FovRange(pub u32);

pub struct Fov {
    pub points: HashSet<Pos>,
}

impl Fov {
    pub fn new(map: &Map, from: Pos, FovRange(range): FovRange) -> Self {
        let points: HashSet<Pos> = RPACaster::new(from, range as i32, 0.33, |pos| {
            map.try_cell_at(pos).map(|idx| map.tile_defs[*idx].opacity)
        })
        .filter_map(|(pos, opacity)| if opacity < 1.0 { Some(pos) } else { None })
        .collect();

        Fov { points }
    }
}

/// TODO: the falloff / power should be part of this
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
        let points: HashMap<Pos, Color> = RPACaster::new(from, range as i32, 0.33, |pos| {
            map.try_cell_at(pos).map(|idx| map.tile_defs[*idx].opacity)
        })
        .filter(|(p, opacity)| fov.points.contains(p) && *opacity < 1.0)
        .map(|(pos, opacity)| {
            let d = from.fdist(pos);
            let mut falloff = DIST_SCALE * d.powi(2);
            if falloff < 1.0 {
                falloff = falloff.powf(EXP_FALLOFF);
            }

            let transparency = 1.0 - opacity;
            let cell_color = Color::RGB(
                (color.r as f32 * transparency / falloff) as u8,
                (color.g as f32 * transparency / falloff) as u8,
                (color.b as f32 * transparency / falloff) as u8,
            );

            (pos, cell_color)
        })
        .collect();

        LightMap { points }
    }

    pub fn apply_light_level(&self, p: Pos, color: Color) -> Option<Color> {
        let light_color = self.points.get(&p)?;

        Some(blend(color, *light_color, BLEND_PERC))
    }
}

/// Restrictive Precise Angle Shadowcasting
/// https://www.roguebasin.com/index.php/Restrictive_Precise_Angle_Shadowcasting
struct RPACaster<F>
where
    F: Fn(Pos) -> Option<f32>,
{
    /// Mapping from position to opacity
    get_opacity: F,
    /// location of obstructions encountered so far in the current octant and their opacity
    obstructions: Vec<(Angle, f32)>,
    /// Iterator of all candidate cells for the current octant
    cells: OctantCells,
    /// The octant currently being iterated over
    octant: usize,
    /// Radius to include cells up to
    radius: i32,
    /// Smoothed cutoff point for the given radius
    r_cutoff: f32,
    /// Origin point we are starting at
    from: Pos,
}

impl<F> RPACaster<F>
where
    F: Fn(Pos) -> Option<f32>,
{
    fn new(from: Pos, radius: i32, smoothing: f32, get_opacity: F) -> Self {
        let r_cutoff = radius as f32 + smoothing;

        Self {
            get_opacity,
            obstructions: Vec::new(),
            cells: OctantCells::new(radius, r_cutoff, 0),
            octant: 0,
            radius,
            r_cutoff,
            from,
        }
    }

    #[inline]
    fn next_cell(&mut self) -> Option<(Pos, Angle, f32)> {
        loop {
            let (pos, angle) = match self.cells.next() {
                Some(elem) => elem,
                None => {
                    if self.octant == 7 {
                        return None; // all octants scanned
                    }

                    self.octant += 1;
                    self.obstructions.clear();
                    self.cells = OctantCells::new(self.radius, self.r_cutoff, self.octant);

                    return self.next_cell();
                }
            };

            // skip out of bounds cells
            let pos = self.from + pos;
            match (self.get_opacity)(pos) {
                Some(opacity) => return Some((pos, angle, opacity)),
                None => continue,
            }
        }
    }
}

impl<F> Iterator for RPACaster<F>
where
    F: Fn(Pos) -> Option<f32>,
{
    type Item = (Pos, f32);

    fn next(&mut self) -> Option<(Pos, f32)> {
        let (pos, angle, mut cell_opacity) = self.next_cell()?;

        let mut opacity: f32 = 0.0;
        let mut v_near = true;
        let mut v_center = true;
        let mut v_far = true;

        for &(o_angle, o_opacity) in self.obstructions.iter() {
            v_near = v_near && !o_angle.contains(angle.near);
            v_center = v_center && !o_angle.contains(angle.center);
            v_far = v_far && !o_angle.contains(angle.far);

            if !v_center {
                opacity = opacity.max(0.5 * o_opacity);
            }

            // This is tunable
            let visible = v_center && (v_near || v_far);
            if !visible {
                opacity = opacity.max(o_opacity);
            }

            if opacity >= 1.0 {
                break;
            }
        }

        cell_opacity += opacity;
        if cell_opacity > 0.0 {
            self.obstructions.push((angle, cell_opacity));
        }

        Some((pos, opacity))
    }
}

#[derive(Default, Clone, Copy)]
struct Angle {
    near: f32,
    center: f32,
    far: f32,
}

impl Angle {
    // using <= is more restrictive
    fn contains(&self, angle: f32) -> bool {
        self.near < angle && angle < self.far
    }
}

const OCTANTS: [(i32, i32, bool); 8] = [
    (1, 1, true),
    (1, 1, false),
    (1, -1, true),
    (1, -1, false),
    (-1, -1, true),
    (-1, -1, false),
    (-1, 1, true),
    (-1, 1, false),
];

/// Iterator over the canidate cells offsets for FOV for a single octant centered at (0, 0).
struct OctantCells {
    quad_x: i32,
    quad_y: i32,
    is_vert: bool,
    /// Radius to include cells up to
    radius: i32,
    /// Smoothed cutoff point for the given radius
    r_cutoff: f32,
    /// Current radial offset
    dr: i32,
    /// Current transverse offset
    dx: i32,
}

impl OctantCells {
    fn new(radius: i32, r_cutoff: f32, octant: usize) -> Self {
        let (quad_x, quad_y, is_vert) = OCTANTS[octant];

        Self {
            quad_x,
            quad_y,
            is_vert,
            radius,
            r_cutoff,
            dr: 0,
            dx: 0,
        }
    }
}

impl Iterator for OctantCells {
    type Item = (Pos, Angle);

    fn next(&mut self) -> Option<(Pos, Angle)> {
        if self.dr == 0 {
            self.dr += 1;
            return Some((Pos::new(0, 0), Angle::default()));
        }

        if self.dx > self.dr {
            // end of row
            self.dx = 0;
            self.dr += 1;
        }

        if self.dr > self.radius {
            return None; // end of octant
        }

        let (a, b) = if self.is_vert {
            (self.dx * self.quad_x, self.dr * self.quad_y)
        } else {
            (self.dr * self.quad_x, self.dx * self.quad_y)
        };

        if (a as f32).hypot(b as f32) >= self.r_cutoff {
            // at cutoff so try next cell
            self.dx += 1;
            return self.next();
        }

        let n_cells_in_row = self.dr + 1;
        let angle_allocation = 1.0 / n_cells_in_row as f32;
        let near = self.dx as f32 * angle_allocation;
        let angle = Angle {
            near,
            center: near + 0.5 * angle_allocation,
            far: near + angle_allocation,
        };

        self.dx += 1;

        Some((Pos::new(a, b), angle))
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
