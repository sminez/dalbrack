//! See:
//!   https://www.roguebasin.com/index.php/FOV_using_recursive_shadowcasting
//!   https://www.roguebasin.com/index.php/Line_of_Sight_-_Tobias_Downer
//!   https://www.roguebasin.com/index.php/Computing_LOS_for_Large_Areas
//!   https://www.roguebasin.com/index.php?title=Discussion:Field_of_Vision
//!   https://www.roguebasin.com/index.php/Restrictive_Precise_Angle_Shadowcasting
use crate::{Pos, map::Map, ui::blend};
use sdl2::pixels::Color;
use std::collections::{HashMap, HashSet};

/// Scaling factor for inverse-square falloff
const DIST_SCALE: f32 = 0.15;
/// Exponent to correct with when r^2 drops below 1.0
const EXP_FALLOFF: f32 = 0.11;
/// % of original color to use when blending light levels
const BLEND_PERC: f32 = 0.5;
/// Smoothing factor added to radius to make the edge points "nicer"
const R_SMOOTHING: f32 = 0.33;

#[derive(Debug, Clone, Copy)]
pub struct FovRange(pub u32);

pub struct Fov {
    pub points: HashSet<Pos>,
    pub from: Pos,
    pub r_cutoff: f32,
}

impl Fov {
    pub fn new(map: &Map, from: Pos, FovRange(range): FovRange) -> Self {
        let r_cutoff = range as f32 + R_SMOOTHING;
        let points: HashSet<Pos> =
            RPACaster::new(from, range as i32, r_cutoff, Vis::CenterPlus, |pos| {
                map.try_cell_at(pos).map(|idx| map.tile_defs[*idx].opacity)
            })
            .filter_map(|(pos, opacity)| if opacity < 1.0 { Some(pos) } else { None })
            .collect();

        Fov {
            points,
            from,
            r_cutoff,
        }
    }

    /// Whether or not there is a possible intersection between a light source and the full FOV
    /// radius. This is only used as a heuristic to avoid computing light map contributions from
    /// sources that are definitely out of range.
    fn could_contain(&self, p: Pos, r: f32) -> bool {
        self.from.fdist(p) <= self.r_cutoff + r
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
            if !fov.could_contain(*from, source.range as f32 + R_SMOOTHING) {
                continue;
            }

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
        let r_cutoff = range as f32 + R_SMOOTHING;
        let points: HashMap<Pos, Color> =
            RPACaster::new(from, range as i32, r_cutoff, Vis::CenterPlus, |pos| {
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

#[allow(dead_code)]
enum Vis {
    Any,
    CenterPlus,
    All,
}

impl Vis {
    fn is_visible(&self, v_near: bool, v_center: bool, v_far: bool) -> bool {
        match self {
            Self::Any => v_near || v_center || v_far,
            Self::CenterPlus => v_center && (v_near || v_far),
            Self::All => v_near && v_center && v_far,
        }
    }
}

/// Restrictive Precise Angle Shadowcasting
/// https://www.roguebasin.com/index.php/Restrictive_Precise_Angle_Shadowcasting
struct RPACaster<F>
where
    F: Fn(Pos) -> Option<f32>,
{
    /// How restrictive to be when classifying a tile as visible
    vis: Vis,
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
    fn new(from: Pos, radius: i32, r_cutoff: f32, restrictiveness: Vis, get_opacity: F) -> Self {
        Self {
            vis: restrictiveness,
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

            if !self.vis.is_visible(v_near, v_center, v_far) {
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

#[derive(Default, Debug, Clone, Copy)]
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
    // x coordinate transform for the current quadrant
    quad_x: i32,
    // y coordinate transform for the current quadrant
    quad_y: i32,
    // whether or not this is the upper octant in the current quadrant
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
