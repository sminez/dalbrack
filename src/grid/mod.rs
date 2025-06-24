use rand::{Rng, rng};
use std::{
    iter::from_fn,
    ops::{Add, AddAssign, Index, IndexMut},
};

mod astar;
mod dijkstra_map;

pub use astar::a_star;
pub use dijkstra_map::dijkstra_map;

const NEIGHBOURS: [(i32, i32); 8] = [
    (-1, 0),
    (1, 0),
    (0, -1),
    (0, 1),
    (-1, -1),
    (-1, 1),
    (1, -1),
    (1, 1),
];

/// A cell position within a [Grid]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

impl Pos {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn dist(&self, other: Pos) -> u32 {
        self.fdist(other).ceil() as u32
    }

    pub fn fdist(&self, other: Pos) -> f32 {
        (((self.x - other.x).pow(2) + (self.y - other.y).pow(2)) as f32).sqrt()
    }

    pub fn random_offset(&self) -> Pos {
        let mut r = rng();
        let mut pos = *self;

        // 012
        // 3 4
        // 567
        let dir = r.random_range(0..8);

        if [0, 3, 5].contains(&dir) {
            pos.x -= 1;
        } else if [2, 4, 7].contains(&dir) {
            pos.x += 1;
        }

        if [0, 1, 2].contains(&dir) {
            pos.y -= 1;
        } else if [5, 6, 7].contains(&dir) {
            pos.y += 1;
        }

        pos
    }
}

impl Add<Pos> for Pos {
    type Output = Pos;

    fn add(self, rhs: Pos) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl AddAssign<Pos> for Pos {
    fn add_assign(&mut self, rhs: Pos) {
        *self = *self + rhs
    }
}

/// A generic 2D grid container
#[derive(Default, Debug, Clone)]
pub struct Grid<T> {
    pub cells: Vec<T>,
    pub w: usize,
    pub h: usize,
}

impl<T> Grid<T> {
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    #[inline]
    pub fn idx(&self, x: usize, y: usize) -> usize {
        y * self.w + x
    }

    #[inline]
    pub fn pos_idx(&self, Pos { x, y }: Pos) -> usize {
        y as usize * self.w + x as usize
    }

    pub fn cell_at(&self, pos: Pos) -> &T {
        let idx = self.pos_idx(pos);
        &self.cells[idx]
    }

    pub fn contains_pos(&self, pos: Pos) -> bool {
        pos.x >= 0 && pos.y >= 0 && (pos.x as usize) < self.w && (pos.y as usize) < self.h
    }

    pub fn try_cell_at(&self, pos: Pos) -> Option<&T> {
        if !self.contains_pos(pos) {
            return None;
        }

        let idx = self.pos_idx(pos);
        if idx >= self.cells.len() {
            None
        } else {
            Some(&self.cells[idx])
        }
    }

    pub fn neighbouring_tiles(&self, pos: Pos) -> impl Iterator<Item = Pos> {
        let mut i = 0;

        from_fn(move || {
            loop {
                if i == 8 {
                    break;
                }

                let (dx, dy) = NEIGHBOURS[i];
                i += 1;
                let p = Pos::new(pos.x + dx, pos.y + dy);
                if self.contains_pos(p) {
                    return Some(p);
                }
            }

            None
        })
    }

    pub fn line_between(&self, from: Pos, to: Pos) -> Vec<Pos> {
        if !(self.contains_pos(from) && self.contains_pos(to)) {
            return Vec::new();
        }

        let dy = to.y - from.y;
        let dx = to.x - from.x;

        let (y_longer, mut d_long, d_short) = if dy.abs() > dx.abs() {
            (true, dy, dx)
        } else {
            (false, dx, dy)
        };

        let k = if d_long == 0 {
            d_short as f32
        } else {
            d_short as f32 / d_long as f32
        };

        let inc = if d_long < 0 { -1 } else { 1 };
        d_long *= inc;

        (0..=d_long)
            .map(|i| {
                let d1 = i * inc;
                let d2 = (d1 as f32 * k) as i32;
                if y_longer {
                    from + Pos::new(d2, d1)
                } else {
                    from + Pos::new(d1, d2)
                }
            })
            .collect()
    }
}

impl<T> Grid<T>
where
    T: Clone,
{
    pub fn new(w: usize, h: usize, fill_val: T) -> Self {
        Self {
            cells: vec![fill_val; w * h],
            w,
            h,
        }
    }
}

impl<T> Index<usize> for Grid<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.cells[index]
    }
}

impl<T> IndexMut<usize> for Grid<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.cells[index]
    }
}

impl<T> Index<Pos> for Grid<T> {
    type Output = T;

    fn index(&self, pos: Pos) -> &Self::Output {
        &self.cells[self.pos_idx(pos)]
    }
}

impl<T> IndexMut<Pos> for Grid<T> {
    fn index_mut(&mut self, pos: Pos) -> &mut Self::Output {
        let idx = self.pos_idx(pos);
        &mut self.cells[idx]
    }
}
